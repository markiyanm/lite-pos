import { invoke } from "@tauri-apps/api/core";

export interface LogEntry {
	timestamp: string;
	level: string;
	source: string;
	category: string;
	message: string;
	raw: string;
}

export interface LogFilters {
	date?: string;
	level?: string;
	category?: string;
	limit?: number;
}

/**
 * Write a log entry via the Rust backend.
 * Source is always "frontend" for calls from the SvelteKit layer.
 */
export async function logEvent(
	level: string,
	category: string,
	message: string,
	logLevelThreshold?: string,
	enabled?: boolean
): Promise<void> {
	return invoke("log_event", {
		level,
		source: "frontend",
		category,
		message,
		logLevelThreshold: logLevelThreshold ?? undefined,
		enabled: enabled ?? undefined,
	});
}

/**
 * Retrieve filtered log entries for a given date.
 */
export async function getLogEntries(filters?: LogFilters): Promise<LogEntry[]> {
	return invoke<LogEntry[]>("get_log_entries", {
		date: filters?.date ?? undefined,
		level: filters?.level ?? undefined,
		category: filters?.category ?? undefined,
		limit: filters?.limit ?? undefined,
	});
}

/**
 * List all dates for which log files exist (most recent first).
 */
export async function getLogDates(): Promise<string[]> {
	return invoke<string[]>("get_log_dates");
}

/**
 * Export the full log content for a given date as a string.
 */
export async function exportLog(date?: string): Promise<string> {
	return invoke<string>("export_log", { date: date ?? undefined });
}

/**
 * Delete log files older than the retention period. Returns bytes freed.
 */
export async function purgeOldLogs(retentionDays?: number): Promise<number> {
	return invoke<number>("purge_old_logs", {
		retentionDays: retentionDays ?? undefined,
	});
}
