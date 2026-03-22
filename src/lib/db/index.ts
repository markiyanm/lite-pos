import Database from "@tauri-apps/plugin-sql";

let db: Database | null = null;

export async function getDb(): Promise<Database> {
	if (!db) {
		db = await Database.load("sqlite:lite-pos.db");
		// WAL mode allows concurrent readers during writes. Set per-connection
		// since PRAGMA in migrations may not persist across connections.
		await db.execute("PRAGMA journal_mode = WAL", []);
		// Busy timeout: wait up to 5s for a lock instead of failing immediately
		// with SQLITE_BUSY. Needed because background sync and logging write
		// concurrently with user operations.
		await db.execute("PRAGMA busy_timeout = 5000", []);
	}
	return db;
}

export async function execute(query: string, bindValues?: unknown[]): Promise<{ rowsAffected: number; lastInsertId: number }> {
	const database = await getDb();
	const result = await database.execute(query, bindValues);
	return { rowsAffected: result.rowsAffected, lastInsertId: result.lastInsertId ?? 0 };
}

export async function select<T>(query: string, bindValues?: unknown[]): Promise<T[]> {
	const database = await getDb();
	return database.select(query, bindValues);
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
		} catch {
			// Rollback failed — connection may be in a bad state, but don't mask the original error
		}
		throw err;
	}
}
