import { select, execute } from "$lib/db/index.js";
import type { Category } from "$lib/types/index.js";

export async function getCategories(): Promise<Category[]> {
	return select<Category>(
		"SELECT * FROM categories WHERE deleted_at IS NULL ORDER BY sort_order, name"
	);
}

export async function getCategory(id: number): Promise<Category | null> {
	const rows = await select<Category>("SELECT * FROM categories WHERE id = ? AND deleted_at IS NULL", [id]);
	return rows[0] ?? null;
}

export async function createCategory(category: { name: string; description?: string; color?: string; icon?: string; sort_order?: number }): Promise<{ lastInsertId: number }> {
	const uuid = crypto.randomUUID();
	return execute(
		"INSERT INTO categories (uuid, name, description, color, icon, sort_order) VALUES (?, ?, ?, ?, ?, ?)",
		[uuid, category.name, category.description ?? null, category.color ?? "#6366f1", category.icon ?? null, category.sort_order ?? 0]
	);
}

export async function updateCategory(id: number, updates: Partial<Category>): Promise<void> {
	const sets: string[] = [];
	const values: unknown[] = [];

	const fields: (keyof Category)[] = ["name", "description", "color", "icon", "sort_order"];
	for (const field of fields) {
		if (updates[field] !== undefined) {
			sets.push(`${field} = ?`);
			values.push(updates[field]);
		}
	}

	if (sets.length === 0) return;
	values.push(id);
	await execute(`UPDATE categories SET ${sets.join(", ")} WHERE id = ?`, values);
}

export async function deleteCategory(id: number): Promise<void> {
	await execute("UPDATE categories SET deleted_at = datetime('now') WHERE id = ?", [id]);
}
