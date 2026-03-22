import { select, execute } from "$lib/db/index.js";
import type { Setting } from "$lib/types/index.js";

export async function getAllSettings(): Promise<Setting[]> {
	return select<Setting>("SELECT * FROM settings ORDER BY group_name, key");
}

export async function updateSetting(key: string, value: string): Promise<void> {
	await execute("UPDATE settings SET value = ? WHERE key = ?", [value, key]);
}
