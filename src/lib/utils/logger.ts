import { logEvent } from "$lib/commands/logging.js";
import { settingsStore } from "$lib/stores/settings.svelte.js";

/**
 * Convenience logger that wraps the Tauri log_event command.
 * All methods silently swallow errors so logging never crashes the app.
 *
 * Usage:
 *   log.info('auth', 'User logged in successfully')
 *   log.error('payment', 'Payment failed: timeout')
 */
function createLogger() {
	function getSettings() {
		const enabled = settingsStore.get("enable_logging") !== "false";
		const level = settingsStore.get("log_level") || "info";
		return { enabled, level };
	}

	async function write(level: string, category: string, message: string) {
		try {
			const { enabled, level: threshold } = getSettings();
			await logEvent(level, category, message, threshold, enabled);
		} catch {
			// Logging failures must never crash the app
		}
	}

	return {
		info(category: string, message: string) {
			write("info", category, message);
		},
		warn(category: string, message: string) {
			write("warn", category, message);
		},
		error(category: string, message: string) {
			write("error", category, message);
		},
		debug(category: string, message: string) {
			write("debug", category, message);
		},
	};
}

export const log = createLogger();
