import { invoke } from "@tauri-apps/api/core";
import type {
	SolaTransactionResponse,
	SolaRequestInfo,
	SolaTransactionResult,
	SolaCnpTransactionResult,
	SolaSaveCardResult
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

export async function processSolaCnpTransaction(params: {
	apiKey: string;
	cardToken: string;
	cvvToken: string;
	exp: string;
	amountCents: number;
	invoiceNumber?: string;
	name?: string;
	zip?: string;
}): Promise<SolaCnpTransactionResult> {
	return invoke<SolaCnpTransactionResult>("process_sola_cnp_transaction", {
		apiKey: params.apiKey,
		cardToken: params.cardToken,
		cvvToken: params.cvvToken,
		exp: params.exp,
		amountCents: params.amountCents,
		invoiceNumber: params.invoiceNumber,
		name: params.name,
		zip: params.zip
	});
}

export async function solaSaveCard(params: {
	apiKey: string;
	cardToken: string;
	cvvToken: string;
	exp: string;
	name?: string;
}): Promise<SolaSaveCardResult> {
	return invoke<SolaSaveCardResult>("sola_save_card", {
		apiKey: params.apiKey,
		cardToken: params.cardToken,
		cvvToken: params.cvvToken,
		exp: params.exp,
		name: params.name
	});
}

export async function processSolaTokenTransaction(params: {
	apiKey: string;
	token: string;
	amountCents: number;
	invoiceNumber?: string;
	cvv?: string;
}): Promise<SolaCnpTransactionResult> {
	return invoke<SolaCnpTransactionResult>("process_sola_token_transaction", {
		apiKey: params.apiKey,
		token: params.token,
		amountCents: params.amountCents,
		invoiceNumber: params.invoiceNumber,
		cvv: params.cvv
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
