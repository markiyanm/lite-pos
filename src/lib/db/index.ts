import Database from "@tauri-apps/plugin-sql";

let db: Database | null = null;
let initialized = false;

export async function getDb(): Promise<Database> {
	if (!db) {
		db = await Database.load("sqlite:lite-pos.db?mode=rwc&busy_timeout=5000");
	}
	// Ensure busy_timeout and WAL mode are set on every connection we get.
	// tauri-plugin-sql uses sqlx::SqlitePool which may hand us different connections,
	// and PRAGMAs are per-connection. We run this once after load.
	if (!initialized) {
		initialized = true;
		try {
			await db.execute("PRAGMA busy_timeout = 5000", []);
			await db.execute("PRAGMA journal_mode = WAL", []);
		} catch {
			// Non-fatal — some PRAGMAs may fail during migrations
		}
	}
	return db;
}

export async function execute(query: string, bindValues?: unknown[]): Promise<{ rowsAffected: number; lastInsertId: number }> {
	const database = await getDb();
	try {
		const result = await database.execute(query, bindValues);
		return { rowsAffected: result.rowsAffected, lastInsertId: result.lastInsertId ?? 0 };
	} catch (err: unknown) {
		const msg = String(err);
		if (msg.includes("database is locked") || msg.includes("517")) {
			// Retry once after explicitly setting busy_timeout on this connection
			await database.execute("PRAGMA busy_timeout = 5000", []);
			const result = await database.execute(query, bindValues);
			return { rowsAffected: result.rowsAffected, lastInsertId: result.lastInsertId ?? 0 };
		}
		throw err;
	}
}

export async function select<T>(query: string, bindValues?: unknown[]): Promise<T[]> {
	const database = await getDb();
	try {
		return await database.select(query, bindValues);
	} catch (err: unknown) {
		const msg = String(err);
		if (msg.includes("database is locked") || msg.includes("517")) {
			await database.execute("PRAGMA busy_timeout = 5000", []);
			return database.select(query, bindValues);
		}
		throw err;
	}
}

/**
 * Execute a callback inside a SQLite savepoint (nestable transaction).
 * Uses SAVEPOINT/RELEASE/ROLLBACK TO instead of BEGIN/COMMIT/ROLLBACK
 * because SQLite does not support nested BEGIN TRANSACTION, and the
 * tauri-plugin-sql connection may have implicit transactions active.
 * If the callback throws, the savepoint is rolled back and the error re-thrown.
 */
let _savepointCounter = 0;

export async function withTransaction<T>(callback: () => Promise<T>): Promise<T> {
	const database = await getDb();
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
