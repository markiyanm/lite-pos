import { select, execute } from "$lib/db/index.js";
import type { Customer } from "$lib/types/index.js";

export async function getCustomers(search?: string): Promise<Customer[]> {
	let query = "SELECT * FROM customers WHERE deleted_at IS NULL";
	const params: unknown[] = [];

	if (search) {
		query += " AND (first_name LIKE ? OR last_name LIKE ? OR email LIKE ? OR phone LIKE ?)";
		const term = `%${search}%`;
		params.push(term, term, term, term);
	}

	query += " ORDER BY last_name, first_name";
	return select<Customer>(query, params);
}

export async function getCustomer(id: number): Promise<Customer | null> {
	const rows = await select<Customer>("SELECT * FROM customers WHERE id = ? AND deleted_at IS NULL", [id]);
	return rows[0] ?? null;
}

export async function createCustomer(customer: Omit<Customer, "id" | "uuid" | "created_at" | "updated_at" | "deleted_at">): Promise<{ lastInsertId: number }> {
	const uuid = crypto.randomUUID();
	return execute(
		`INSERT INTO customers (uuid, first_name, last_name, email, phone, billing_address_line1, billing_address_line2, billing_city, billing_state, billing_zip, shipping_address_line1, shipping_address_line2, shipping_city, shipping_state, shipping_zip, notes)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		[uuid, customer.first_name, customer.last_name, customer.email, customer.phone, customer.billing_address_line1, customer.billing_address_line2, customer.billing_city, customer.billing_state, customer.billing_zip, customer.shipping_address_line1, customer.shipping_address_line2, customer.shipping_city, customer.shipping_state, customer.shipping_zip, customer.notes]
	);
}

export async function updateCustomer(id: number, updates: Partial<Customer>): Promise<void> {
	const sets: string[] = [];
	const values: unknown[] = [];

	const fields: (keyof Customer)[] = [
		"first_name", "last_name", "email", "phone",
		"billing_address_line1", "billing_address_line2", "billing_city", "billing_state", "billing_zip",
		"shipping_address_line1", "shipping_address_line2", "shipping_city", "shipping_state", "shipping_zip",
		"notes"
	];

	for (const field of fields) {
		if (updates[field] !== undefined) {
			sets.push(`${field} = ?`);
			values.push(updates[field]);
		}
	}

	if (sets.length === 0) return;
	values.push(id);
	await execute(`UPDATE customers SET ${sets.join(", ")} WHERE id = ?`, values);
}

export async function deleteCustomer(id: number): Promise<void> {
	await execute("UPDATE customers SET deleted_at = datetime('now') WHERE id = ?", [id]);
}
