import { select } from "$lib/db/index.js";

export interface SalesByPeriod {
	period: string;
	order_count: number;
	total_cents: number;
	tax_cents: number;
	avg_order_cents: number;
}

export interface ProductMetric {
	product_id: number;
	product_name: string;
	product_sku: string | null;
	total_quantity: number;
	total_revenue_cents: number;
	total_tax_cents: number;
	order_count: number;
}

export interface InventorySummary {
	id: number;
	name: string;
	sku: string | null;
	stock_quantity: number;
	low_stock_threshold: number;
	cost_price_cents: number;
	sale_price_cents: number;
	category_name: string | null;
}

export type GroupBy = "day" | "week" | "month";

function dateFormat(groupBy: GroupBy): string {
	switch (groupBy) {
		case "day":
			return "%Y-%m-%d";
		case "week":
			return "%Y-W%W";
		case "month":
			return "%Y-%m";
	}
}

export async function getSalesByPeriod(opts: {
	groupBy: GroupBy;
	dateFrom?: string;
	dateTo?: string;
}): Promise<SalesByPeriod[]> {
	const fmt = dateFormat(opts.groupBy);
	let query = `
		SELECT
			strftime('${fmt}', completed_at) AS period,
			COUNT(*) AS order_count,
			SUM(total_cents) AS total_cents,
			SUM(tax_total_cents) AS tax_cents,
			CAST(ROUND(AVG(total_cents)) AS INTEGER) AS avg_order_cents
		FROM orders
		WHERE status = 'completed' AND deleted_at IS NULL AND completed_at IS NOT NULL
	`;
	const params: unknown[] = [];

	if (opts.dateFrom) {
		query += " AND completed_at >= ?";
		params.push(opts.dateFrom);
	}
	if (opts.dateTo) {
		query += " AND completed_at <= ?";
		params.push(opts.dateTo + "T23:59:59");
	}

	query += ` GROUP BY period ORDER BY period`;
	return select<SalesByPeriod>(query, params);
}

export async function getProductMetrics(opts?: {
	dateFrom?: string;
	dateTo?: string;
	limit?: number;
}): Promise<ProductMetric[]> {
	let query = `
		SELECT
			oi.product_id,
			oi.product_name,
			oi.product_sku,
			SUM(oi.quantity) AS total_quantity,
			SUM(oi.line_subtotal_cents) AS total_revenue_cents,
			SUM(oi.line_tax_cents) AS total_tax_cents,
			COUNT(DISTINCT oi.order_id) AS order_count
		FROM order_items oi
		JOIN orders o ON o.id = oi.order_id
		WHERE o.status = 'completed' AND o.deleted_at IS NULL
	`;
	const params: unknown[] = [];

	if (opts?.dateFrom) {
		query += " AND o.completed_at >= ?";
		params.push(opts.dateFrom);
	}
	if (opts?.dateTo) {
		query += " AND o.completed_at <= ?";
		params.push(opts.dateTo + "T23:59:59");
	}

	query += " GROUP BY oi.product_id, oi.product_name";
	query += " ORDER BY total_revenue_cents DESC";

	if (opts?.limit) {
		query += " LIMIT ?";
		params.push(opts.limit);
	}

	return select<ProductMetric>(query, params);
}

export async function getInventorySummary(): Promise<InventorySummary[]> {
	return select<InventorySummary>(
		`SELECT
			p.id, p.name, p.sku, p.stock_quantity, p.low_stock_threshold,
			p.cost_price_cents, p.sale_price_cents,
			c.name AS category_name
		FROM products p
		LEFT JOIN categories c ON c.id = p.category_id
		WHERE p.deleted_at IS NULL AND p.is_active = 1
		ORDER BY p.stock_quantity ASC`
	);
}
