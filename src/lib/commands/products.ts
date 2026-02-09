import { select, execute } from "$lib/db/index.js";
import type { Product } from "$lib/types/index.js";

export async function getProducts(opts?: { categoryId?: number; search?: string; activeOnly?: boolean }): Promise<Product[]> {
	let query = "SELECT * FROM products WHERE deleted_at IS NULL";
	const params: unknown[] = [];

	if (opts?.activeOnly !== false) {
		query += " AND is_active = 1";
	}
	if (opts?.categoryId) {
		query += " AND category_id = ?";
		params.push(opts.categoryId);
	}
	if (opts?.search) {
		query += " AND (name LIKE ? OR sku LIKE ? OR barcode LIKE ?)";
		const term = `%${opts.search}%`;
		params.push(term, term, term);
	}

	query += " ORDER BY sort_order, name";
	return select<Product>(query, params);
}

export async function getProduct(id: number): Promise<Product | null> {
	const rows = await select<Product>("SELECT * FROM products WHERE id = ? AND deleted_at IS NULL", [id]);
	return rows[0] ?? null;
}

export async function createProduct(product: Omit<Product, "id" | "uuid" | "created_at" | "updated_at" | "deleted_at">): Promise<{ lastInsertId: number }> {
	const uuid = crypto.randomUUID();
	return execute(
		`INSERT INTO products (uuid, name, description, sku, barcode, category_id, cost_price_cents, sale_price_cents, tax_rate_bps, stock_quantity, low_stock_threshold, image_path, is_active, sort_order)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		[uuid, product.name, product.description, product.sku, product.barcode, product.category_id, product.cost_price_cents, product.sale_price_cents, product.tax_rate_bps, product.stock_quantity, product.low_stock_threshold, product.image_path, product.is_active ? 1 : 0, product.sort_order]
	);
}

export async function updateProduct(id: number, updates: Partial<Product>): Promise<void> {
	const sets: string[] = [];
	const values: unknown[] = [];

	const fields: (keyof Product)[] = ["name", "description", "sku", "barcode", "category_id", "cost_price_cents", "sale_price_cents", "tax_rate_bps", "stock_quantity", "low_stock_threshold", "image_path", "sort_order"];
	for (const field of fields) {
		if (updates[field] !== undefined) {
			sets.push(`${field} = ?`);
			values.push(updates[field]);
		}
	}
	if (updates.is_active !== undefined) {
		sets.push("is_active = ?");
		values.push(updates.is_active ? 1 : 0);
	}

	if (sets.length === 0) return;
	values.push(id);
	await execute(`UPDATE products SET ${sets.join(", ")} WHERE id = ?`, values);
}

export async function deleteProduct(id: number): Promise<void> {
	await execute("UPDATE products SET deleted_at = datetime('now') WHERE id = ?", [id]);
}

export async function adjustStock(id: number, delta: number): Promise<void> {
	await execute("UPDATE products SET stock_quantity = stock_quantity + ? WHERE id = ?", [delta, id]);
}
