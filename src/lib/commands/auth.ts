import { select, execute } from "$lib/db/index.js";
import type { User } from "$lib/types/index.js";
import { hashPin } from "$lib/commands/crypto.js";
import { log } from "$lib/utils/logger.js";

export async function login(pin: string): Promise<User | null> {
	const hashed = await hashPin(pin);

	// Try matching the hashed PIN first (migrated users)
	const hashedMatch = await select<User>(
		"SELECT * FROM users WHERE pin_hash = ? AND is_active = 1 AND deleted_at IS NULL LIMIT 1",
		[hashed]
	);
	if (hashedMatch[0]) {
		log.info("auth", `Login successful: user="${hashedMatch[0].name}" role=${hashedMatch[0].role}`);
		return hashedMatch[0];
	}

	// Backward compat: try matching the raw PIN (pre-migration users)
	const rawMatch = await select<User>(
		"SELECT * FROM users WHERE pin_hash = ? AND is_active = 1 AND deleted_at IS NULL LIMIT 1",
		[pin]
	);
	if (rawMatch[0]) {
		// Migrate the user's PIN to hashed on successful login
		await execute("UPDATE users SET pin_hash = ? WHERE id = ?", [hashed, rawMatch[0].id]);
		log.info("auth", `Login successful (PIN migrated): user="${rawMatch[0].name}" role=${rawMatch[0].role}`);
		return rawMatch[0];
	}

	log.warn("auth", "Login failed: invalid PIN");
	return null;
}

export async function getUsers(): Promise<User[]> {
	return select<User>(
		"SELECT * FROM users WHERE deleted_at IS NULL ORDER BY name"
	);
}

export async function createUser(user: { name: string; email?: string; pin: string; role: "admin" | "cashier" }): Promise<{ lastInsertId: number }> {
	const uuid = crypto.randomUUID();
	const hashed = await hashPin(user.pin);
	return execute(
		"INSERT INTO users (uuid, name, email, pin_hash, role) VALUES (?, ?, ?, ?, ?)",
		[uuid, user.name, user.email ?? null, hashed, user.role]
	);
}

export async function updateUser(id: number, user: { name?: string; email?: string; pin?: string; role?: string; is_active?: boolean }): Promise<void> {
	const sets: string[] = [];
	const values: unknown[] = [];

	if (user.name !== undefined) { sets.push("name = ?"); values.push(user.name); }
	if (user.email !== undefined) { sets.push("email = ?"); values.push(user.email); }
	if (user.pin !== undefined) {
		const hashed = await hashPin(user.pin);
		sets.push("pin_hash = ?");
		values.push(hashed);
	}
	if (user.role !== undefined) { sets.push("role = ?"); values.push(user.role); }
	if (user.is_active !== undefined) { sets.push("is_active = ?"); values.push(user.is_active ? 1 : 0); }

	if (sets.length === 0) return;
	values.push(id);
	await execute(`UPDATE users SET ${sets.join(", ")} WHERE id = ?`, values);
}
