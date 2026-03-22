import { select, execute } from "$lib/db/index.js";
import { encryptValue } from "$lib/commands/crypto.js";
import type { CustomerPaymentToken } from "$lib/types/index.js";

/**
 * Get all non-deleted payment tokens for a customer.
 * Token values remain encrypted — the frontend should never decrypt them.
 */
export async function getCustomerTokens(customerId: number): Promise<CustomerPaymentToken[]> {
	return select<CustomerPaymentToken>(
		`SELECT * FROM customer_payment_tokens
		 WHERE customer_id = ? AND deleted_at IS NULL
		 ORDER BY is_default DESC, created_at DESC`,
		[customerId]
	);
}

/**
 * Get the default payment token for a customer, or null if none.
 */
export async function getDefaultToken(customerId: number): Promise<CustomerPaymentToken | null> {
	const rows = await select<CustomerPaymentToken>(
		`SELECT * FROM customer_payment_tokens
		 WHERE customer_id = ? AND is_default = 1 AND deleted_at IS NULL
		 LIMIT 1`,
		[customerId]
	);
	return rows[0] ?? null;
}

/**
 * Create a new payment token record.
 * The token value is encrypted before storage.
 */
export async function createPaymentToken(data: {
	customerId: number;
	token: string; // plaintext xToken from gateway — will be encrypted
	cardLastFour: string;
	cardBrand: string | null;
	expMonth: number | null;
	expYear: number | null;
	isDefault?: boolean;
	gatewayPaymentMethodId?: string;
	gatewayRevision?: number;
}): Promise<{ lastInsertId: number }> {
	const uuid = crypto.randomUUID();

	// Encrypt the token before storage
	const encryptedToken = await encryptValue(data.token);

	// If setting as default, unset any existing default first
	if (data.isDefault) {
		await execute(
			`UPDATE customer_payment_tokens SET is_default = 0
			 WHERE customer_id = ? AND deleted_at IS NULL`,
			[data.customerId]
		);
	}

	return execute(
		`INSERT INTO customer_payment_tokens (
			uuid, customer_id, gateway_payment_method_id, token,
			card_last_four, card_brand, exp_month, exp_year,
			is_default, gateway_revision
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		[
			uuid,
			data.customerId,
			data.gatewayPaymentMethodId ?? null,
			encryptedToken,
			data.cardLastFour,
			data.cardBrand ?? null,
			data.expMonth ?? null,
			data.expYear ?? null,
			data.isDefault ? 1 : 0,
			data.gatewayRevision ?? null
		]
	);
}

/**
 * Soft-delete a payment token.
 * If it was the default, promotes the next most recent card.
 */
export async function deletePaymentToken(id: number): Promise<void> {
	// Check if this is the default
	const rows = await select<CustomerPaymentToken>(
		`SELECT * FROM customer_payment_tokens WHERE id = ? AND deleted_at IS NULL`,
		[id]
	);
	const token = rows[0];
	if (!token) return;

	// Soft delete
	await execute(
		`UPDATE customer_payment_tokens SET deleted_at = datetime('now') WHERE id = ?`,
		[id]
	);

	// If it was the default, promote the next most recent card
	if (token.is_default) {
		const nextDefault = await select<CustomerPaymentToken>(
			`SELECT id FROM customer_payment_tokens
			 WHERE customer_id = ? AND deleted_at IS NULL
			 ORDER BY created_at DESC LIMIT 1`,
			[token.customer_id]
		);
		if (nextDefault[0]) {
			await execute(
				`UPDATE customer_payment_tokens SET is_default = 1 WHERE id = ?`,
				[nextDefault[0].id]
			);
		}
	}
}

/**
 * Set a specific token as the default for a customer.
 * Unsets any other defaults for that customer.
 */
export async function setDefaultToken(customerId: number, tokenId: number): Promise<void> {
	// Unset all defaults for this customer
	await execute(
		`UPDATE customer_payment_tokens SET is_default = 0
		 WHERE customer_id = ? AND deleted_at IS NULL`,
		[customerId]
	);

	// Set the new default
	await execute(
		`UPDATE customer_payment_tokens SET is_default = 1 WHERE id = ? AND customer_id = ?`,
		[tokenId, customerId]
	);
}
