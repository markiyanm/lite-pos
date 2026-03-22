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
	try {
		return await invoke<string>("hash_pin", { pin });
	} catch {
		// Fallback for browser-only mode (no Tauri backend).
		// Uses Web Crypto API to match the Rust implementation: SHA-256("vira-pin:" + pin)
		const encoder = new TextEncoder();
		const data = encoder.encode(`vira-pin:${pin}`);
		const hashBuffer = await crypto.subtle.digest("SHA-256", data);
		const hashArray = Array.from(new Uint8Array(hashBuffer));
		return hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");
	}
}
