import { invoke } from "@tauri-apps/api/core";

export interface PrinterInfo {
	name: string;
	is_default: boolean;
}

export async function getSystemPrinters(): Promise<PrinterInfo[]> {
	return invoke<PrinterInfo[]>("get_system_printers");
}
