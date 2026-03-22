import { invoke } from "@tauri-apps/api/core";
import type {
	SolaTransactionResponse,
	SolaRequestInfo,
	SolaTransactionResult
} from "$lib/types/index.js";

export async function processSolaTransaction(params: {
	apiKey: string;
	deviceId: string;
	amountCents: number;
	invoiceNumber?: string;
}): Promise<SolaTransactionResult> {
	return invoke<SolaTransactionResult>("process_sola_transaction", {
		apiKey: params.apiKey,
		deviceId: params.deviceId,
		amountCents: params.amountCents,
		invoiceNumber: params.invoiceNumber
	});
}

export async function cancelSolaTransaction(params: {
	apiKey: string;
	deviceId: string;
}): Promise<SolaTransactionResult> {
	return invoke<SolaTransactionResult>("cancel_sola_transaction", {
		apiKey: params.apiKey,
		deviceId: params.deviceId
	});
}

export async function buildSolaRequestInfo(params: {
	apiKey: string;
	deviceId: string;
	amountCents: number;
	invoiceNumber?: string;
	command?: string;
}): Promise<SolaRequestInfo> {
	return invoke<SolaRequestInfo>("build_sola_request_info", {
		apiKey: params.apiKey,
		deviceId: params.deviceId,
		amountCents: params.amountCents,
		invoiceNumber: params.invoiceNumber,
		command: params.command
	});
}
