import Database from "@tauri-apps/plugin-sql";

let db: Database | null = null;

export async function getDb(): Promise<Database> {
	if (!db) {
		db = await Database.load("sqlite:lite-pos.db");
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
 * Execute a callback inside a SQLite transaction (BEGIN / COMMIT / ROLLBACK).
 * If the callback throws, the transaction is rolled back and the error re-thrown.
 */
export async function withTransaction<T>(callback: () => Promise<T>): Promise<T> {
	const database = await getDb();
	await database.execute("BEGIN TRANSACTION", []);
	try {
		const result = await callback();
		await database.execute("COMMIT", []);
		return result;
	} catch (err) {
		await database.execute("ROLLBACK", []);
		throw err;
	}
}
