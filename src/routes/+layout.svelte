<script lang="ts">
	import "../app.css";
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import AppSidebar from "$lib/components/app/AppSidebar.svelte";
	import UpdateChecker from "$lib/components/app/UpdateChecker.svelte";
	import { Toaster } from "$lib/components/ui/sonner/index.js";
	import { ModeWatcher } from "mode-watcher";
	import { toast } from "svelte-sonner";
	import { session } from "$lib/stores/session.svelte.js";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import { themeStore } from "$lib/stores/theme.svelte.js";
	import { untrack, onMount } from "svelte";
	import { log } from "$lib/utils/logger.js";
	import {
		syncCustomersWithGateway,
		acquireSyncLock,
		releaseSyncLock,
		getDecryptedApiKey
	} from "$lib/commands/customer-sync.js";

	let { children } = $props();

	const isLoginPage = $derived(page.url.pathname === "/login");

	// Routes that require admin role
	const adminOnlyPrefixes = ["/settings", "/categories", "/reports"];

	function isAdminRoute(pathname: string): boolean {
		if (adminOnlyPrefixes.some((p) => pathname.startsWith(p))) return true;
		// /products/new and /products/[id] edit pages, but NOT the /products list
		if (pathname.startsWith("/products/")) return true;
		return false;
	}

	// Log app startup
	onMount(() => {
		log.info("startup", `App started: platform=${navigator.platform}`);
	});

	// Load theme once when settings are loaded (use untrack to prevent reactive loops)
	let hasLoadedTheme = false;
	$effect(() => {
		if (settingsStore.loaded && !hasLoadedTheme) {
			hasLoadedTheme = true;
			untrack(() => {
				themeStore.loadFromSettings();
			});
		}
	});

	// Route guard: redirect to login if not authenticated
	$effect(() => {
		if (!isLoginPage && !session.isAuthenticated) {
			goto("/login");
		}
	});

	// Admin guard: redirect non-admin users away from admin-only routes
	$effect(() => {
		if (session.isAuthenticated && !session.isAdmin && isAdminRoute(page.url.pathname)) {
			goto("/pos");
			toast.error("Admin access required");
		}
	});

	// Background customer sync
	let syncIntervalId: ReturnType<typeof setInterval> | null = null;

	async function runBackgroundSync() {
		const syncEnabled = settingsStore.getBoolean("gateway_customer_sync_enabled");
		const apiKeyConfigured = !!settingsStore.get("sola_gateway_api_key");

		if (!syncEnabled || !apiKeyConfigured) return;

		const locked = await acquireSyncLock();
		if (!locked) return; // already running

		try {
			const apiKey = await getDecryptedApiKey();
			if (!apiKey) {
				await releaseSyncLock();
				return;
			}

			const result = await syncCustomersWithGateway(apiKey);
			if (result.errors > 0) {
				toast.warning(`Customer sync completed with ${result.errors} errors`);
			}
		} catch {
			// Silently fail for background sync
		} finally {
			try { await releaseSyncLock(); } catch {}
		}
	}

	function startSyncSchedule() {
		if (syncIntervalId) {
			clearInterval(syncIntervalId);
			syncIntervalId = null;
		}

		const intervalMinutes = settingsStore.getNumber("gateway_sync_interval_minutes") || 15;
		const intervalMs = Math.max(5, intervalMinutes) * 60 * 1000;

		syncIntervalId = setInterval(runBackgroundSync, intervalMs);
	}

	// Start background sync after settings load, with 5-second delay per FR-12
	let hasSyncStarted = false;
	$effect(() => {
		if (settingsStore.loaded && session.isAuthenticated && !hasSyncStarted) {
			hasSyncStarted = true;
			untrack(() => {
				setTimeout(() => {
					runBackgroundSync();
					startSyncSchedule();
				}, 5000);
			});
		}
	});

	function handleGlobalKeydown(e: KeyboardEvent) {
		if (isLoginPage || !session.isAuthenticated) return;

		const isModKey = e.ctrlKey || e.metaKey;

		// Ctrl/Cmd+K — focus search input on current page
		if (isModKey && e.key === "k") {
			e.preventDefault();
			const searchInput = document.querySelector<HTMLInputElement>(
				'input[placeholder*="Search"]'
			);
			if (searchInput) searchInput.focus();
		}

		// F2 — go to POS
		if (e.key === "F2") {
			e.preventDefault();
			goto("/pos");
		}

		// F3 — go to Orders
		if (e.key === "F3") {
			e.preventDefault();
			goto("/orders");
		}

		// F4 — go to Products
		if (e.key === "F4") {
			e.preventDefault();
			goto("/products");
		}
	}
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<svelte:head>
	<title>Lite POS</title>
</svelte:head>

{#if isLoginPage}
	{@render children()}
{:else if session.isAuthenticated}
	<div class="flex h-screen overflow-hidden">
		<AppSidebar />
		<main class="flex-1 overflow-y-auto bg-background">
			{@render children()}
		</main>
	</div>
{/if}

<ModeWatcher />
<Toaster richColors position="bottom-right" />
<UpdateChecker />
