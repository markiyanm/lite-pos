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

// ─── Write Serializer (Async Mutex) ─────────────────────────────────────────
//
// SQLite allows only one writer at a time. tauri-plugin-sql uses sqlx::SqlitePool
// with multiple connections, and PRAGMA busy_timeout is per-connection — we can't
// control which connection the pool hands us, so busy_timeout is unreliable.
//
// Instead, we serialize ALL write operations through a single-lane queue in JS.
// This guarantees no two writes contend for SQLite's write lock, eliminating
// SQLITE_BUSY entirely. Reads (SELECT) bypass the queue since WAL mode allows
// concurrent reads alongside a single writer.
//
// For a single-user desktop app, this adds negligible latency (writes are fast,
// they just can't overlap) and provides 100% reliability.
// ─────────────────────────────────────────────────────────────────────────────

let writeQueue: Promise<unknown> = Promise.resolve();
let insideTransaction = false;

function serializeWrite<T>(operation: () => Promise<T>): Promise<T> {
	// If we're already inside a withTransaction callback, don't re-queue
	// (the transaction already holds the queue slot)
	if (insideTransaction) {
		return operation();
	}

	const next = writeQueue.then(
		() => operation(),
		() => operation()  // Run even if previous write failed
	);
	// Update the queue tail — subsequent writes wait for this one
	writeQueue = next.catch(() => {}); // Prevent unhandled rejection on queue chain
	return next;
}

export async function execute(query: string, bindValues?: unknown[]): Promise<{ rowsAffected: number; lastInsertId: number }> {
	return serializeWrite(async () => {
		const database = await getDb();
		const result = await database.execute(query, bindValues);
		return { rowsAffected: result.rowsAffected, lastInsertId: result.lastInsertId ?? 0 };
	});
}

export async function select<T>(query: string, bindValues?: unknown[]): Promise<T[]> {
	// Reads bypass the write queue — WAL mode allows concurrent reads
	const database = await getDb();
	return database.select(query, bindValues);
}

/**
 * Execute a callback inside a serialized write session.
 * The ENTIRE transaction (SAVEPOINT + all writes + RELEASE) runs as a single
 * queued unit, so no other write can interleave or cause lock contention.
 *
 * Uses SAVEPOINT/RELEASE/ROLLBACK TO instead of BEGIN/COMMIT/ROLLBACK
 * because SQLite does not support nested BEGIN TRANSACTION.
 *
 * All execute() calls inside the callback detect they're inside a transaction
 * and skip the queue (they already hold the queue slot).
 */
let _savepointCounter = 0;

export async function withTransaction<T>(callback: () => Promise<T>): Promise<T> {
	return serializeWrite(async () => {
		const database = await getDb();
		const name = `sp_${++_savepointCounter}`;
		await database.execute(`SAVEPOINT ${name}`, []);
		insideTransaction = true;
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
		} finally {
			insideTransaction = false;
		}
	});
}
