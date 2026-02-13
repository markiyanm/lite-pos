import { invoke } from "@tauri-apps/api/core";
import type { SolaTransactionResponse } from "$lib/types/index.js";

export async function processSolaTransaction(params: {
	apiKey: string;
	deviceId: string;
	amountCents: number;
	invoiceNumber?: string;
}): Promise<SolaTransactionResponse> {
	return invoke<SolaTransactionResponse>("process_sola_transaction", {
		apiKey: params.apiKey,
		deviceId: params.deviceId,
		amountCents: params.amountCents,
		invoiceNumber: params.invoiceNumber
	});
}
