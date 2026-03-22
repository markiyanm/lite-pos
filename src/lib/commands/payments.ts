import { execute } from "$lib/db/index.js";
import type { PaymentMethod } from "$lib/types/index.js";
import { log } from "$lib/utils/logger.js";

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
	const lastFourDisplay = payment.cardLastFour ? ` card=****${payment.cardLastFour}` : "";
	log.info("payment", `Payment processed: order=${payment.orderId} method=${payment.method} amount=$${(payment.amountCents / 100).toFixed(2)}${lastFourDisplay}`);
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
