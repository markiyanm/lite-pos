import { invoke } from "@tauri-apps/api/core";

export async function encryptValue(plaintext: string): Promise<string> {
	if (!plaintext) return "";
	return invoke<string>("encrypt_value", { plaintext });
}

export async function decryptValue(encrypted: string): Promise<string> {
	if (!encrypted) return "";
	return invoke<string>("decrypt_value", { encrypted });
}

export async function hashPin(pin: string): Promise<string> {
	return invoke<string>("hash_pin", { pin });
}
