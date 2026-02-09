import { select, execute } from "$lib/db/index.js";
import type { Refund, RefundItem } from "$lib/types/index.js";

export async function getRefundsByOrder(orderId: number): Promise<Refund[]> {
	return select<Refund>("SELECT * FROM refunds WHERE order_id = ? ORDER BY created_at DESC", [orderId]);
}

export async function getRefundItems(refundId: number): Promise<RefundItem[]> {
	return select<RefundItem>("SELECT * FROM refund_items WHERE refund_id = ?", [refundId]);
}

export async function createRefund(refund: {
	orderId: number;
	userId: number;
	totalRefundCents: number;
	reason?: string;
}): Promise<{ lastInsertId: number }> {
	const uuid = crypto.randomUUID();
	return execute(
		`INSERT INTO refunds (uuid, order_id, user_id, total_refund_cents, reason)
		 VALUES (?, ?, ?, ?, ?)`,
		[uuid, refund.orderId, refund.userId, refund.totalRefundCents, refund.reason ?? null]
	);
}

export async function addRefundItem(item: {
	refundId: number;
	orderItemId: number;
	quantity: number;
	refundAmountCents: number;
	restock: boolean;
}): Promise<{ lastInsertId: number }> {
	const uuid = crypto.randomUUID();
	return execute(
		`INSERT INTO refund_items (uuid, refund_id, order_item_id, quantity, refund_amount_cents, restock)
		 VALUES (?, ?, ?, ?, ?, ?)`,
		[uuid, item.refundId, item.orderItemId, item.quantity, item.refundAmountCents, item.restock ? 1 : 0]
	);
}

export async function setOrderRefunded(orderId: number): Promise<void> {
	await execute("UPDATE orders SET status = 'refunded' WHERE id = ?", [orderId]);
}
