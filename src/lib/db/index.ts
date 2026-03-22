import Database from "@tauri-apps/plugin-sql";

let db: Database | null = null;

export async function getDb(): Promise<Database> {
	if (!db) {
		db = await Database.load("sqlite:lite-pos.db");
		// Set WAL mode once (persists across connections in the same DB file)
		try {
			await db.execute("PRAGMA journal_mode = WAL", []);
		} catch {
			// Non-fatal
		}
	}
	return db;
}

/**
 * Ensure busy_timeout is set on the current pool connection before any operation.
 * tauri-plugin-sql uses sqlx::SqlitePool — each call may get a different connection,
 * and PRAGMAs are per-connection. We must set busy_timeout every time to guarantee
 * the connection we're using will wait (up to 5s) for locks instead of failing immediately.
 */
async function ensureBusyTimeout(database: Database): Promise<void> {
	await database.execute("PRAGMA busy_timeout = 5000", []);
}

export async function execute(query: string, bindValues?: unknown[]): Promise<{ rowsAffected: number; lastInsertId: number }> {
	const database = await getDb();
	await ensureBusyTimeout(database);
	const result = await database.execute(query, bindValues);
	return { rowsAffected: result.rowsAffected, lastInsertId: result.lastInsertId ?? 0 };
}

export async function select<T>(query: string, bindValues?: unknown[]): Promise<T[]> {
	const database = await getDb();
	await ensureBusyTimeout(database);
	return database.select(query, bindValues);
}

/**
 * Execute a callback inside a SQLite savepoint (nestable transaction).
 * Uses SAVEPOINT/RELEASE/ROLLBACK TO instead of BEGIN/COMMIT/ROLLBACK
 * because SQLite does not support nested BEGIN TRANSACTION, and the
 * tauri-plugin-sql connection may have implicit transactions active.
 * If the callback throws, the savepoint is rolled back and the error re-thrown.
 *
 * Note: busy_timeout is set before the SAVEPOINT starts. All operations inside
 * the callback use execute/select which also set busy_timeout, so every pool
 * connection involved in this transaction will wait for locks.
 */
let _savepointCounter = 0;

export async function withTransaction<T>(callback: () => Promise<T>): Promise<T> {
	const database = await getDb();
	await ensureBusyTimeout(database);
	const name = `sp_${++_savepointCounter}`;
	await database.execute(`SAVEPOINT ${name}`, []);
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
