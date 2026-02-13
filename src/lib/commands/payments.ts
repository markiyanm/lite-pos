import { execute } from "$lib/db/index.js";
import type { PaymentMethod } from "$lib/types/index.js";

export async function addPayment(payment: {
	orderId: number;
	method: PaymentMethod;
	amountCents: number;
	changeCents: number;
	referenceNumber?: string;
	cardAuthCode?: string;
	cardLastFour?: string;
	cardType?: string;
	cardEntryMode?: string;
	gatewayRefNum?: string;
	gatewayResponse?: string;
}): Promise<{ lastInsertId: number }> {
	const uuid = crypto.randomUUID();
	return execute(
		`INSERT INTO payments (
			uuid, order_id, method, amount_cents, change_cents, reference_number,
			card_auth_code, card_last_four, card_type, card_entry_mode,
			gateway_ref_num, gateway_response
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		[
			uuid,
			payment.orderId,
			payment.method,
			payment.amountCents,
			payment.changeCents,
			payment.referenceNumber ?? null,
			payment.cardAuthCode ?? null,
			payment.cardLastFour ?? null,
			payment.cardType ?? null,
			payment.cardEntryMode ?? null,
			payment.gatewayRefNum ?? null,
			payment.gatewayResponse ?? null
		]
	);
}
