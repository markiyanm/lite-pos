import Database from "@tauri-apps/plugin-sql";

let db: Database | null = null;

export async function getDb(): Promise<Database> {
	if (!db) {
		db = await Database.load("sqlite:lite-pos.db");
		// WAL mode persists in the DB file — only needs to be set once ever
		try {
			await db.execute("PRAGMA journal_mode = WAL", []);
		} catch {
			// Non-fatal
		}
	}
	return db;
}

/**
 * Check if an error is a SQLite BUSY/LOCKED error.
 * Code 5 = SQLITE_BUSY, Code 517 = SQLITE_BUSY_SNAPSHOT
 */
function isBusyError(err: unknown): boolean {
	const msg = String(err);
	return msg.includes("database is locked") || msg.includes("(code: 5)") || msg.includes("(code: 517)");
}

/**
 * Sleep for a given number of milliseconds.
 */
function sleep(ms: number): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Retry a database operation with exponential backoff on SQLITE_BUSY.
 *
 * tauri-plugin-sql uses sqlx::SqlitePool with multiple connections.
 * PRAGMA busy_timeout is per-connection, and we can't guarantee which
 * pool connection executes our query. Instead of fighting the pool,
 * we retry with increasing delays (50ms, 150ms, 350ms, 750ms, 1550ms)
 * giving the lock holder time to finish.
 */
const MAX_RETRIES = 5;
const BASE_DELAY_MS = 50;

async function withRetry<T>(operation: () => Promise<T>): Promise<T> {
	let lastErr: unknown;
	for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
		try {
			return await operation();
		} catch (err) {
			if (isBusyError(err) && attempt < MAX_RETRIES) {
				lastErr = err;
				const delay = BASE_DELAY_MS * Math.pow(2, attempt); // 50, 100, 200, 400, 800
				await sleep(delay);
				// Also try setting busy_timeout on whatever connection we get next
				try {
					const database = await getDb();
					await database.execute("PRAGMA busy_timeout = 5000", []);
				} catch {
					// Non-fatal
				}
				continue;
			}
			throw err;
		}
	}
	throw lastErr;
}

export async function execute(query: string, bindValues?: unknown[]): Promise<{ rowsAffected: number; lastInsertId: number }> {
	return withRetry(async () => {
		const database = await getDb();
		const result = await database.execute(query, bindValues);
		return { rowsAffected: result.rowsAffected, lastInsertId: result.lastInsertId ?? 0 };
	});
}

export async function select<T>(query: string, bindValues?: unknown[]): Promise<T[]> {
	return withRetry(async () => {
		const database = await getDb();
		return database.select(query, bindValues);
	});
}

/**
 * Execute a callback inside a SQLite savepoint (nestable transaction).
 * Uses SAVEPOINT/RELEASE/ROLLBACK TO instead of BEGIN/COMMIT/ROLLBACK
 * because SQLite does not support nested BEGIN TRANSACTION, and the
 * tauri-plugin-sql connection may have implicit transactions active.
 *
 * The SAVEPOINT creation itself is retried on BUSY. Operations inside
 * the callback use execute/select which have their own retry logic.
 */
let _savepointCounter = 0;

export async function withTransaction<T>(callback: () => Promise<T>): Promise<T> {
	const database = await getDb();
	const name = `sp_${++_savepointCounter}`;

	// Retry the SAVEPOINT creation itself on BUSY
	await withRetry(async () => {
		await database.execute(`SAVEPOINT ${name}`, []);
	});

	try {
		const result = await callback();
		await database.execute(`RELEASE SAVEPOINT ${name}`, []);
		return result;
	} catch (err) {
		try {
			await database.execute(`ROLLBACK TO SAVEPOINT ${name}`, []);
			await database.execute(`RELEASE SAVEPOINT ${name}`, []);
		} catch (rollbackErr) {
			console.error(`[DB] Rollback of ${name} failed:`, rollbackErr);
		}
		throw err;
	}
}
