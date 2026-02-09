import { execute } from "$lib/db/index.js";
import type { PaymentMethod } from "$lib/types/index.js";

export async function addPayment(payment: {
	orderId: number;
	method: PaymentMethod;
	amountCents: number;
	changeCents: number;
	referenceNumber?: string;
}): Promise<{ lastInsertId: number }> {
	const uuid = crypto.randomUUID();
	return execute(
		`INSERT INTO payments (uuid, order_id, method, amount_cents, change_cents, reference_number)
		 VALUES (?, ?, ?, ?, ?, ?)`,
		[uuid, payment.orderId, payment.method, payment.amountCents, payment.changeCents, payment.referenceNumber ?? null]
	);
}
