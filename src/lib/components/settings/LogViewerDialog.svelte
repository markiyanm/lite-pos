<script lang="ts">
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "$lib/components/ui/dialog/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Select, SelectContent, SelectItem, SelectTrigger } from "$lib/components/ui/select/index.js";
	import { Loader2, Download, Trash2 } from "lucide-svelte";
	import { getLogEntries, getLogDates, exportLog, purgeOldLogs, type LogEntry } from "$lib/commands/logging.js";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import { toast } from "svelte-sonner";

	interface Props {
		open: boolean;
		onClose: () => void;
	}

	let { open = $bindable(), onClose }: Props = $props();

	let dates = $state<string[]>([]);
	let selectedDate = $state("");
	let entries = $state<LogEntry[]>([]);
	let loading = $state(false);
	let datesLoading = $state(false);

	// Level filters — all enabled by default
	let showError = $state(true);
	let showWarn = $state(true);
	let showInfo = $state(true);
	let showDebug = $state(true);

	// Category filter
	let selectedCategory = $state("");
	let categories = $derived.by(() => {
		const cats = new Set<string>();
		for (const entry of entries) {
			if (entry.category) cats.add(entry.category);
		}
		return Array.from(cats).sort();
	});

	// Filtered entries based on level toggles and category
	let filteredEntries = $derived.by(() => {
		return entries.filter((e) => {
			const lvl = e.level.toLowerCase();
			if (lvl === "error" && !showError) return false;
			if (lvl === "warn" && !showWarn) return false;
			if (lvl === "info" && !showInfo) return false;
			if (lvl === "debug" && !showDebug) return false;
			if (selectedCategory && e.category.toLowerCase() !== selectedCategory.toLowerCase()) return false;
			return true;
		});
	});

	let scrollContainer = $state<HTMLDivElement>(undefined!);

	// Load dates when dialog opens
	$effect(() => {
		if (open) {
			loadDates();
		}
	});

	// Load entries when date changes
	$effect(() => {
		if (selectedDate) {
			loadEntries();
		}
	});

	async function loadDates() {
		datesLoading = true;
		try {
			dates = await getLogDates();
			if (dates.length > 0 && !selectedDate) {
				selectedDate = dates[0]; // most recent
			}
		} catch {
			toast.error("Failed to load log dates");
		} finally {
			datesLoading = false;
		}
	}

	async function loadEntries() {
		if (!selectedDate) return;
		loading = true;
		try {
			entries = await getLogEntries({ date: selectedDate, limit: 1000 });
			// Auto-scroll to bottom after entries load
			requestAnimationFrame(() => {
				if (scrollContainer) {
					scrollContainer.scrollTop = scrollContainer.scrollHeight;
				}
			});
		} catch {
			toast.error("Failed to load log entries");
			entries = [];
		} finally {
			loading = false;
		}
	}

	async function handleExport() {
		try {
			const content = await exportLog(selectedDate || undefined);
			if (!content) {
				toast.info("No log content to export");
				return;
			}
			const date = selectedDate || new Date().toISOString().slice(0, 10);
			const blob = new Blob([content], { type: "text/plain" });
			const url = URL.createObjectURL(blob);
			const a = document.createElement("a");
			a.href = url;
			a.download = `vira-logs-${date}.txt`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);
			toast.success("Log file exported");
		} catch {
			toast.error("Failed to export log");
		}
	}

	async function handlePurge() {
		try {
			const retentionDays = settingsStore.getNumber("log_retention_days") || 30;
			const freed = await purgeOldLogs(retentionDays);
			const freedMB = (freed / (1024 * 1024)).toFixed(2);
			toast.success(`Cleared old logs (${freedMB} MB freed)`);
			// Reload dates after purge
			await loadDates();
			if (dates.length > 0) {
				selectedDate = dates[0];
			} else {
				selectedDate = "";
				entries = [];
			}
		} catch {
			toast.error("Failed to clear old logs");
		}
	}

	function levelColor(level: string): string {
		switch (level.toLowerCase()) {
			case "error":
				return "text-red-500";
			case "warn":
				return "text-orange-500";
			case "debug":
				return "text-muted-foreground";
			default:
				return "text-foreground";
		}
	}
</script>

<Dialog bind:open onOpenChange={(v) => { if (!v) onClose(); }}>
	<DialogContent class="max-w-4xl max-h-[90vh] flex flex-col">
		<DialogHeader>
			<DialogTitle>Application Logs</DialogTitle>
			<DialogDescription>View and export application log entries.</DialogDescription>
		</DialogHeader>

		<!-- Filter Bar -->
		<div class="flex flex-wrap items-center gap-3 border-b pb-3">
			<!-- Date Selector -->
			<div class="flex items-center gap-2">
				<span class="text-sm font-medium">Date:</span>
				{#if datesLoading}
					<Loader2 class="h-4 w-4 animate-spin" />
				{:else}
					<select
						class="h-9 rounded-md border border-input bg-background px-3 text-sm"
						bind:value={selectedDate}
					>
						{#each dates as date}
							<option value={date}>{date}</option>
						{/each}
						{#if dates.length === 0}
							<option value="">No logs available</option>
						{/if}
					</select>
				{/if}
			</div>

			<!-- Level Toggles -->
			<div class="flex items-center gap-1">
				<button
					type="button"
					class="rounded-md px-2 py-1 text-xs font-medium transition-colors {showError ? 'bg-red-500/20 text-red-500 ring-1 ring-red-500/30' : 'bg-muted text-muted-foreground'}"
					onclick={() => (showError = !showError)}
				>
					Error
				</button>
				<button
					type="button"
					class="rounded-md px-2 py-1 text-xs font-medium transition-colors {showWarn ? 'bg-orange-500/20 text-orange-500 ring-1 ring-orange-500/30' : 'bg-muted text-muted-foreground'}"
					onclick={() => (showWarn = !showWarn)}
				>
					Warn
				</button>
				<button
					type="button"
					class="rounded-md px-2 py-1 text-xs font-medium transition-colors {showInfo ? 'bg-blue-500/20 text-blue-500 ring-1 ring-blue-500/30' : 'bg-muted text-muted-foreground'}"
					onclick={() => (showInfo = !showInfo)}
				>
					Info
				</button>
				<button
					type="button"
					class="rounded-md px-2 py-1 text-xs font-medium transition-colors {showDebug ? 'bg-gray-500/20 text-gray-500 ring-1 ring-gray-500/30' : 'bg-muted text-muted-foreground'}"
					onclick={() => (showDebug = !showDebug)}
				>
					Debug
				</button>
			</div>

			<!-- Category Filter -->
			{#if categories.length > 0}
				<div class="flex items-center gap-2">
					<span class="text-sm font-medium">Category:</span>
					<select
						class="h-9 rounded-md border border-input bg-background px-3 text-sm"
						bind:value={selectedCategory}
					>
						<option value="">All</option>
						{#each categories as cat}
							<option value={cat}>{cat}</option>
						{/each}
					</select>
				</div>
			{/if}
		</div>

		<!-- Log Display -->
		<div
			bind:this={scrollContainer}
			class="flex-1 min-h-0 max-h-[55vh] overflow-y-auto rounded-md border bg-muted/30 p-3"
		>
			{#if loading}
				<div class="flex items-center justify-center py-12">
					<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
				</div>
			{:else if filteredEntries.length === 0}
				<div class="flex items-center justify-center py-12 text-muted-foreground text-sm">
					No log entries found
				</div>
			{:else}
				<div class="space-y-0.5 font-mono text-xs leading-relaxed">
					{#each filteredEntries as entry}
						<div class={levelColor(entry.level)}>
							<span class="text-muted-foreground">[{entry.timestamp}]</span>
							<span class="font-semibold">[{entry.level}]</span>
							<span class="text-muted-foreground">[{entry.source}]</span>
							<span class="text-muted-foreground">[{entry.category}]</span>
							{entry.message}
						</div>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Footer -->
		<DialogFooter class="flex items-center justify-between sm:justify-between">
			<span class="text-sm text-muted-foreground">
				{filteredEntries.length} of {entries.length} entries
			</span>
			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" onclick={handlePurge}>
					<Trash2 class="mr-2 h-4 w-4" />
					Clear Old Logs
				</Button>
				<Button variant="outline" size="sm" onclick={handleExport} disabled={!selectedDate}>
					<Download class="mr-2 h-4 w-4" />
					Export
				</Button>
				<Button variant="default" size="sm" onclick={onClose}>
					Close
				</Button>
			</div>
		</DialogFooter>
	</DialogContent>
</Dialog>
