import { select } from "$lib/db/index.js";
import type { User } from "$lib/types/index.js";

export async function login(pin: string): Promise<User | null> {
	const users = await select<User>(
		"SELECT * FROM users WHERE pin_hash = ? AND is_active = 1 AND deleted_at IS NULL LIMIT 1",
		[pin]
	);
	return users[0] ?? null;
}

export async function getUsers(): Promise<User[]> {
	return select<User>(
		"SELECT * FROM users WHERE deleted_at IS NULL ORDER BY name"
	);
}

export async function createUser(user: { name: string; email?: string; pin: string; role: "admin" | "cashier" }): Promise<{ lastInsertId: number }> {
	const { execute } = await import("$lib/db/index.js");
	const uuid = crypto.randomUUID();
	return execute(
		"INSERT INTO users (uuid, name, email, pin_hash, role) VALUES (?, ?, ?, ?, ?)",
		[uuid, user.name, user.email ?? null, user.pin, user.role]
	);
}

export async function updateUser(id: number, user: { name?: string; email?: string; pin?: string; role?: string; is_active?: boolean }): Promise<void> {
	const { execute } = await import("$lib/db/index.js");
	const sets: string[] = [];
	const values: unknown[] = [];

	if (user.name !== undefined) { sets.push("name = ?"); values.push(user.name); }
	if (user.email !== undefined) { sets.push("email = ?"); values.push(user.email); }
	if (user.pin !== undefined) { sets.push("pin_hash = ?"); values.push(user.pin); }
	if (user.role !== undefined) { sets.push("role = ?"); values.push(user.role); }
	if (user.is_active !== undefined) { sets.push("is_active = ?"); values.push(user.is_active ? 1 : 0); }

	if (sets.length === 0) return;
	values.push(id);
	await execute(`UPDATE users SET ${sets.join(", ")} WHERE id = ?`, values);
}
