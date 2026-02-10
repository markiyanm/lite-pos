import { select, execute } from "$lib/db/index.js";
import type { Order, OrderItem, Payment } from "$lib/types/index.js";

export async function getOrders(opts?: {
	status?: string;
	customerId?: number;
	userId?: number;
	dateFrom?: string;
	dateTo?: string;
}): Promise<Order[]> {
	let query = "SELECT * FROM orders WHERE deleted_at IS NULL";
	const params: unknown[] = [];

	if (opts?.status) {
		query += " AND status = ?";
		params.push(opts.status);
	}
	if (opts?.customerId) {
		query += " AND customer_id = ?";
		params.push(opts.customerId);
	}
	if (opts?.userId) {
		query += " AND user_id = ?";
		params.push(opts.userId);
	}
	if (opts?.dateFrom) {
		query += " AND created_at >= ?";
		params.push(opts.dateFrom);
	}
	if (opts?.dateTo) {
		query += " AND created_at <= ?";
		params.push(opts.dateTo);
	}

	query += " ORDER BY created_at DESC";
	return select<Order>(query, params);
}

export async function getOrder(id: number): Promise<Order | null> {
	const rows = await select<Order>("SELECT * FROM orders WHERE id = ? AND deleted_at IS NULL", [id]);
	return rows[0] ?? null;
}

export async function getOrderItems(orderId: number): Promise<OrderItem[]> {
	return select<OrderItem>("SELECT * FROM order_items WHERE order_id = ?", [orderId]);
}

export async function getOrderPayments(orderId: number): Promise<Payment[]> {
	return select<Payment>("SELECT * FROM payments WHERE order_id = ?", [orderId]);
}

export async function getNextOrderNumber(): Promise<string> {
	const rows = await select<{ value: string }>(
		"SELECT value FROM settings WHERE key = 'order_number_prefix'"
	);
	const prefixRows = rows;
	const seqRows = await select<{ value: string }>(
		"SELECT value FROM settings WHERE key = 'next_order_number'"
	);

	const prefix = prefixRows[0]?.value ?? "ORD-";
	const seq = parseInt(seqRows[0]?.value ?? "1", 10);

	await execute(
		"UPDATE settings SET value = ? WHERE key = 'next_order_number'",
		[String(seq + 1)]
	);

	return `${prefix}${String(seq).padStart(5, "0")}`;
}

export async function createOrder(order: {
	orderNumber: string;
	userId: number;
	customerId?: number;
	subtotalCents: number;
	discountCents: number;
	taxTotalCents: number;
	totalCents: number;
	notes?: string;
	status?: string;
}): Promise<{ lastInsertId: number }> {
	const uuid = crypto.randomUUID();
	return execute(
		`INSERT INTO orders (uuid, order_number, status, customer_id, user_id, subtotal_cents, discount_cents, tax_total_cents, total_cents, notes)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		[uuid, order.orderNumber, order.status ?? "draft", order.customerId ?? null, order.userId, order.subtotalCents, order.discountCents, order.taxTotalCents, order.totalCents, order.notes ?? null]
	);
}

export async function addOrderItem(item: {
	orderId: number;
	productId: number;
	productName: string;
	productSku?: string;
	quantity: number;
	unitPriceCents: number;
	taxRateBps: number;
	lineSubtotalCents: number;
	lineTaxCents: number;
	lineTotalCents: number;
	notes?: string;
}): Promise<{ lastInsertId: number }> {
	const uuid = crypto.randomUUID();
	return execute(
		`INSERT INTO order_items (uuid, order_id, product_id, product_name, product_sku, quantity, unit_price_cents, tax_rate_bps, line_subtotal_cents, line_tax_cents, line_total_cents, notes)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		[uuid, item.orderId, item.productId, item.productName, item.productSku ?? null, item.quantity, item.unitPriceCents, item.taxRateBps, item.lineSubtotalCents, item.lineTaxCents, item.lineTotalCents, item.notes ?? null]
	);
}

export async function completeOrder(orderId: number): Promise<void> {
	await execute(
		"UPDATE orders SET status = 'completed', completed_at = datetime('now') WHERE id = ?",
		[orderId]
	);
}

export async function voidOrder(orderId: number): Promise<void> {
	await execute("UPDATE orders SET status = 'void' WHERE id = ?", [orderId]);
}

export async function updateOrderCustomer(orderId: number, customerId: number | null): Promise<void> {
	await execute("UPDATE orders SET customer_id = ? WHERE id = ?", [customerId, orderId]);
}
