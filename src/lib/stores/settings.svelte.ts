import type { Setting } from "$lib/types/index.js";

let settings = $state<Map<string, Setting>>(new Map());
let loaded = $state(false);

export function getSettingsStore() {
	return {
		get loaded() { return loaded; },

		get(key: string): string {
			return settings.get(key)?.value ?? "";
		},

		getNumber(key: string): number {
			return parseInt(settings.get(key)?.value ?? "0", 10);
		},

		getBoolean(key: string): boolean {
			return settings.get(key)?.value === "true";
		},

		getJson<T>(key: string): T | null {
			const val = settings.get(key)?.value;
			if (!val) return null;
			try {
				return JSON.parse(val) as T;
			} catch {
				return null;
			}
		},

		load(settingsList: Setting[]) {
			const map = new Map<string, Setting>();
			for (const s of settingsList) {
				map.set(s.key, s);
			}
			settings = map;
			loaded = true;
		},

		update(key: string, value: string) {
			const existing = settings.get(key);
			if (existing) {
				settings.set(key, { ...existing, value });
				settings = new Map(settings);
			}
		}
	};
}

export const settingsStore = getSettingsStore();
