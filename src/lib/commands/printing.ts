import { invoke } from "@tauri-apps/api/core";

export interface PrinterInfo {
	name: string;
	is_default: boolean;
}

export interface ReceiptItem {
	name: string;
	quantity: number;
	unit_price: string;
	line_total: string;
}

export interface ReceiptPayment {
	method: string;
	amount: string;
	change?: string;
	reference?: string;
}

export interface ReceiptData {
	store_name: string;
	store_address?: string;
	store_phone?: string;
	header?: string;
	order_number: string;
	date: string;
	customer_name?: string;
	items: ReceiptItem[];
	subtotal: string;
	discount?: string;
	tax_label: string;
	tax: string;
	total: string;
	payments: ReceiptPayment[];
	footer?: string;
}

export async function getSystemPrinters(): Promise<PrinterInfo[]> {
	return invoke<PrinterInfo[]>("get_system_printers");
}

export async function printReceipt(receipt: ReceiptData, printerName: string): Promise<void> {
	return invoke<void>("print_receipt", { receipt, printerName });
}

export async function printHtml(html: string, printerName: string): Promise<void> {
	return invoke<void>("print_html", { html, printerName });
}
