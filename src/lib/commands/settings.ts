import { select, execute } from "$lib/db/index.js";
import type { Setting } from "$lib/types/index.js";

export async function getAllSettings(): Promise<Setting[]> {
	return select<Setting>("SELECT * FROM settings ORDER BY group_name, key");
}

export async function getSetting(key: string): Promise<Setting | null> {
	const rows = await select<Setting>("SELECT * FROM settings WHERE key = ?", [key]);
	return rows[0] ?? null;
}

export async function updateSetting(key: string, value: string): Promise<void> {
	await execute("UPDATE settings SET value = ? WHERE key = ?", [value, key]);
}

export async function getSettingsByGroup(group: string): Promise<Setting[]> {
	return select<Setting>("SELECT * FROM settings WHERE group_name = ? ORDER BY key", [group]);
}
