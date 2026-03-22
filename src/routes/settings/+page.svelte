<script lang="ts">
	import { Settings, Save, Loader2, Sun, Moon, Monitor, ImagePlus, X, Eye, EyeOff, CreditCard, Wifi, Pencil, Check, Trash2, Plus, Printer, FileText, RefreshCw } from "lucide-svelte";
	import { Tabs, TabsContent, TabsList, TabsTrigger } from "$lib/components/ui/tabs/index.js";
	import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import {
		AlertDialog,
		AlertDialogAction,
		AlertDialogCancel,
		AlertDialogContent,
		AlertDialogDescription,
		AlertDialogFooter,
		AlertDialogHeader,
		AlertDialogTitle,
	} from "$lib/components/ui/alert-dialog/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import { Select, SelectContent, SelectItem, SelectTrigger } from "$lib/components/ui/select/index.js";
	import SettingField from "$lib/components/settings/SettingField.svelte";
	import SettingToggle from "$lib/components/settings/SettingToggle.svelte";
	import EmployeeManagement from "$lib/components/settings/EmployeeManagement.svelte";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import { updateSetting, getAllSettings } from "$lib/commands/settings.js";
	import { encryptValue, decryptValue } from "$lib/commands/crypto.js";
	import { getSystemPrinters, type PrinterInfo } from "$lib/commands/printing.js";
	import { syncCustomersWithGateway, acquireSyncLock, releaseSyncLock, getDecryptedApiKey } from "$lib/commands/customer-sync.js";
	import { session } from "$lib/stores/session.svelte.js";
	import { mode, setMode } from "mode-watcher";
	import { toast } from "svelte-sonner";
	import { readFileAsDataUrl } from "$lib/utils.js";
	import type { SolaDevice, SolaDeviceListResponse } from "$lib/types/index.js";
	import { getAllThemes, type Theme } from "$lib/themes.js";
	import { themeStore } from "$lib/stores/theme.svelte.js";
	import PrintReceiptDialog from "$lib/components/receipts/PrintReceiptDialog.svelte";
	import LogViewerDialog from "$lib/components/settings/LogViewerDialog.svelte";
	import { log } from "$lib/utils/logger.js";
	import type { Order, OrderItem, Payment, Customer } from "$lib/types/index.js";

	// Local copies of settings for editing
	let storeName = $state(settingsStore.get("store_name"));
	let storeAddress = $state(settingsStore.get("store_address"));
	let storePhone = $state(settingsStore.get("store_phone"));
	let storeEmail = $state(settingsStore.get("store_email"));
	let storeLogo = $state(settingsStore.get("store_logo"));
	let logoFileInput = $state<HTMLInputElement>(undefined!);
	let currencySymbol = $state(settingsStore.get("currency_symbol"));
	let currencyCode = $state(settingsStore.get("currency_code"));

	let defaultTaxRateBps = $state(settingsStore.get("default_tax_rate_bps"));
	let taxIncludedInPrice = $state(settingsStore.getBoolean("tax_included_in_price"));

	let paymentMethodsEnabled = $state<string[]>(
		settingsStore.getJson<string[]>("payment_methods_enabled") ?? ["cash", "check", "credit_card"]
	);

	let receiptHeader = $state(settingsStore.get("receipt_header"));
	let receiptFooter = $state(settingsStore.get("receipt_footer"));
	let receiptShowStoreInfo = $state(settingsStore.getBoolean("receipt_show_store_info"));

	// Printer settings
	let receiptMode = $state(settingsStore.get("receipt_mode") || "off");
	let printerName = $state(settingsStore.get("printer_name"));
	let printerType = $state(settingsStore.get("printer_type"));
	let paperWidthMm = $state(settingsStore.get("paper_width_mm"));
	// Seed with saved printer name so the <select> has a matching option before refresh
	let availablePrinters = $state.raw<PrinterInfo[]>(
		printerName ? [{ name: printerName, is_default: false }] : []
	);
	let printersLoading = $state(false);

	let lowStockAlertEnabled = $state(settingsStore.getBoolean("low_stock_alert_enabled"));
	let lowStockDefaultThreshold = $state(settingsStore.get("low_stock_default_threshold"));

	let orderNumberPrefix = $state(settingsStore.get("order_number_prefix"));
	let requireCustomerOnOrder = $state(settingsStore.getBoolean("require_customer_on_order"));

	// Diagnostics / Logging settings
	let enableLogging = $state(settingsStore.get("enable_logging") !== "false");
	let logLevel = $state(settingsStore.get("log_level") || "info");
	let logRetentionDays = $state(settingsStore.get("log_retention_days") || "30");
	let logViewerOpen = $state(false);

	// Theme settings
	let selectedTheme = $state(settingsStore.get("color_theme") || "blue");
	const availableThemes = getAllThemes();

	async function handleThemeChange(themeId: string) {
		selectedTheme = themeId;
		themeStore.setTheme(themeId);

		// Auto-save theme immediately
		try {
			await updateSetting("color_theme", themeId);
			settingsStore.update("color_theme", themeId);

			// Show success toast
			const { toast } = await import("svelte-sonner");
			toast.success("Theme updated");
		} catch {
			// Show error toast
			const { toast } = await import("svelte-sonner");
			toast.error("Failed to save theme");
		}
	}

	async function handleBrightnessChange(newMode: "light" | "dark" | "system") {
		setMode(newMode);

		// Show success toast
		const { toast } = await import("svelte-sonner");
		const modeLabel = newMode === "light" ? "Light" : newMode === "dark" ? "Dark" : "System";
		toast.success(`Brightness set to ${modeLabel}`);
	}

	// Card on File setting
	let enableCardOnFile = $state(settingsStore.getBoolean("enable_card_on_file"));

	// Sola Gateway settings
	let solaCardPresent = $state(settingsStore.getBoolean("sola_gateway_card_present"));
	let solaCardNotPresent = $state(settingsStore.getBoolean("sola_gateway_card_not_present"));
	let solaApiKey = $state("");
	let solaApiKeyVisible = $state(false);

	let ifieldsKey = $state("");
	let ifieldsKeyVisible = $state(false);

	// Decrypt the stored API key on load
	const encryptedKey = settingsStore.get("sola_gateway_api_key");
	if (encryptedKey) {
		decryptValue(encryptedKey).then((v) => (solaApiKey = v)).catch(() => {});
	}

	// Decrypt the stored iFields key on load
	const encryptedIfieldsKey = settingsStore.get("ifields_key");
	if (encryptedIfieldsKey) {
		decryptValue(encryptedIfieldsKey).then((v) => (ifieldsKey = v)).catch(() => {});
	}

	// Device selection state
	let solaDefaultDeviceId = $state(settingsStore.get("sola_gateway_default_device_id"));
	let solaDevices = $state<SolaDevice[]>([]);
	let solaDevicesLoading = $state(false);
	let solaDevicesError = $state("");
	let editingDeviceId = $state<string | null>(null);
	let editingDeviceName = $state("");
	let deleteDialogOpen = $state(false);
	let deviceToDelete = $state<SolaDevice | null>(null);
	let addDeviceDialogOpen = $state(false);
	let newDeviceSerialNumber = $state("");
	let newDeviceMake = $state("pax");
	let newDeviceName = $state("");
	let addingDevice = $state(false);

	// Customer Sync settings
	let gatewaySyncEnabled = $state(settingsStore.getBoolean("gateway_customer_sync_enabled"));
	let gatewaySyncInterval = $state(settingsStore.get("gateway_sync_interval_minutes") || "15");
	let customerSyncing = $state(false);
	let lastSyncResult = $state("");

	let activeTab = $state("store");
	let saving = $state(false);
	let saveMessage = $state("");

	// Test print dialog
	let testPrintOpen = $state(false);

	// Generate test receipt data
	const testOrder = $derived<Order>({
		id: 1,
		uuid: "test-uuid",
		order_number: "TEST-001",
		status: "completed",
		customer_id: null,
		user_id: 1,
		subtotal_cents: 4500,
		discount_cents: 0,
		tax_total_cents: 394,
		total_cents: 4894,
		notes: null,
		completed_at: new Date().toISOString(),
		created_at: new Date().toISOString(),
		updated_at: new Date().toISOString(),
		deleted_at: null
	});

	const testItems = $derived<OrderItem[]>([
		{
			id: 1,
			uuid: "item-1",
			order_id: 1,
			product_id: 1,
			product_name: "Sample Product",
			product_sku: "SKU-001",
			quantity: 2,
			unit_price_cents: 1500,
			tax_rate_bps: 875,
			line_subtotal_cents: 3000,
			line_tax_cents: 263,
			line_total_cents: 3263,
			notes: null,
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString()
		},
		{
			id: 2,
			uuid: "item-2",
			order_id: 1,
			product_id: 2,
			product_name: "Another Item",
			product_sku: null,
			quantity: 1,
			unit_price_cents: 1500,
			tax_rate_bps: 875,
			line_subtotal_cents: 1500,
			line_tax_cents: 131,
			line_total_cents: 1631,
			notes: null,
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString()
		}
	]);

	const testPayments = $derived<Payment[]>([
		{
			id: 1,
			uuid: "payment-1",
			order_id: 1,
			method: "cash",
			amount_cents: 5000,
			change_cents: 106,
			reference_number: null,
			card_auth_code: null,
			card_last_four: null,
			card_type: null,
			card_entry_mode: null,
			gateway_ref_num: null,
			gateway_response: null,
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString()
		}
	]);

	async function handleLogoSelect(e: Event) {
		const input = e.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		try {
			storeLogo = await readFileAsDataUrl(file, 128);
		} catch {
			// silent
		}
		input.value = "";
	}

	function togglePaymentMethod(method: string) {
		if (paymentMethodsEnabled.includes(method)) {
			if (paymentMethodsEnabled.length > 1) {
				paymentMethodsEnabled = paymentMethodsEnabled.filter((m) => m !== method);
			}
		} else {
			paymentMethodsEnabled = [...paymentMethodsEnabled, method];
		}
	}

	// Displayed tax rate (convert bps to percentage for display)
	const taxRateDisplay = $derived.by(() => {
		const bps = parseInt(defaultTaxRateBps || "0", 10);
		return (bps / 100).toFixed(2);
	});

	function taxRateFromPercent(percent: string): string {
		const val = parseFloat(percent) || 0;
		return String(Math.round(val * 100));
	}

	// Auto-load printers only when the Printing tab is active and printing is enabled
	$effect(() => {
		if (activeTab === "printing" && receiptMode !== "off" && availablePrinters.length <= 1 && !printersLoading) {
			loadPrinters();
		}
	});

	// Auto-load devices when conditions are met
	$effect(() => {
		const shouldAutoLoad =
			paymentMethodsEnabled.includes("credit_card") &&
			solaCardPresent &&
			solaApiKey.trim() !== "";

		if (shouldAutoLoad && solaDevices.length === 0 && !solaDevicesLoading) {
			fetchSolaDevices();
		}
	});

	async function updateDeviceName(deviceId: string, newName: string) {
		if (!newName.trim()) {
			solaDevicesError = "Device name cannot be empty";
			return;
		}

		try {
			const { fetch } = await import("@tauri-apps/plugin-http");

			const response = await fetch(`https://device.cardknox.com/v1/Device/${deviceId}`, {
				method: "PUT",
				headers: {
					"Authorization": solaApiKey.trim(),
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					xDeviceFriendlyName: newName.trim()
				})
			});

			const data = await response.json() as { xRefnum: string; xResult: "S" | "E" };

			if (!response.ok || data.xResult === "E") {
				throw new Error("Failed to update device name");
			}

			// Refresh the device list to get the updated name from the API
			await fetchSolaDevices();
			editingDeviceId = null;
			editingDeviceName = "";
		} catch (err) {
			solaDevicesError = err instanceof Error ? err.message : "Failed to update device name";
		}
	}

	function maskApiKey(key: string): string {
		if (key.length <= 10) return "***";
		return key.substring(0, 5) + "..." + key.substring(key.length - 5);
	}

	function maskDeviceId(id: string): string {
		if (id.length <= 10) return id;
		return id.substring(0, 5) + "..." + id.substring(id.length - 5);
	}

	async function deleteDevice(deviceId: string) {
		try {
			const { fetch } = await import("@tauri-apps/plugin-http");

			const response = await fetch(`https://device.cardknox.com/v1/Device/${deviceId}`, {
				method: "DELETE",
				headers: {
					"Authorization": solaApiKey.trim()
				}
			});

			const data = await response.json() as { xRefnum: string; xResult: "S" | "E" };

			if (!response.ok || data.xResult === "E") {
				throw new Error("Failed to delete device");
			}

			// Clear default device ID if we deleted the default device
			if (solaDefaultDeviceId === deviceId) {
				solaDefaultDeviceId = "";
				await updateSetting("sola_gateway_default_device_id", "");
			}

			// Refresh the device list
			await fetchSolaDevices();
			deleteDialogOpen = false;
			deviceToDelete = null;
		} catch (err) {
			solaDevicesError = err instanceof Error ? err.message : "Failed to delete device";
			deleteDialogOpen = false;
			deviceToDelete = null;
		}
	}

	async function createDevice() {
		// Validate inputs
		if (!newDeviceSerialNumber.trim() || !newDeviceName.trim()) {
			solaDevicesError = "Serial number and device name are required";
			return;
		}

		// Validate serial number format (8-16 digits)
		if (!/^\d{8,16}$/.test(newDeviceSerialNumber.trim())) {
			solaDevicesError = "Serial number must be 8-16 digits";
			return;
		}

		addingDevice = true;
		solaDevicesError = "";

		try {
			const { fetch } = await import("@tauri-apps/plugin-http");

			const response = await fetch("https://device.cardknox.com/v1/Device", {
				method: "POST",
				headers: {
					"Authorization": solaApiKey.trim(),
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					xDeviceSerialNumber: newDeviceSerialNumber.trim(),
					xDeviceMake: newDeviceMake,
					xDeviceFriendlyName: newDeviceName.trim()
				})
			});

			const data = await response.json() as { xRefnum: string; xResult: "S" | "E"; xDeviceId?: string };

			if (!response.ok || data.xResult === "E") {
				throw new Error("Failed to create device. Please check the serial number and try again.");
			}

			// Refresh the device list
			await fetchSolaDevices();

			// Close dialog and reset form
			addDeviceDialogOpen = false;
			newDeviceSerialNumber = "";
			newDeviceMake = "pax";
			newDeviceName = "";
		} catch (err) {
			solaDevicesError = err instanceof Error ? err.message : "Failed to create device";
		} finally {
			addingDevice = false;
		}
	}

	async function fetchSolaDevices() {
		if (!solaApiKey.trim()) {
			solaDevicesError = "API Key required";
			solaDevices = [];
			return;
		}

		solaDevicesLoading = true;
		solaDevicesError = "";

		try {
			const { fetch } = await import("@tauri-apps/plugin-http");

			const url = "https://device.cardknox.com/v1/Device";
			console.log("[Sola] Fetching from URL:", url);
			console.log("[Sola] API Key (first 10 chars):", solaApiKey.substring(0, 10) + "...");

			const response = await fetch(url, {
				method: "GET",
				headers: {
					"Authorization": solaApiKey.trim()
				}
			});

			console.log("[Sola] Response status:", response.status);
			console.log("[Sola] Response statusText:", response.statusText);
			console.log("[Sola] Response headers:", Object.fromEntries(response.headers.entries()));

			const responseText = await response.text();
			console.log("[Sola] Response body:", responseText);

			if (!response.ok) {
				if (response.status === 401) {
					throw new Error("Invalid API key");
				}
				throw new Error(`HTTP ${response.status}: ${responseText || response.statusText}`);
			}

			const data = JSON.parse(responseText) as SolaDeviceListResponse;
			console.log("[Sola] Parsed response:", data);

			if (data.xResult === "E") {
				throw new Error("API returned error. Check your API key and try again.");
			}

			const devices = data.xDevices || [];
			console.log("[Sola] Devices:", devices);
			solaDevices = devices;

			// Auto-select if only one device
			if (devices.length === 1 && !solaDefaultDeviceId) {
				solaDefaultDeviceId = devices[0].xDeviceId;
			}

			// Clear selection if previously selected device is not in the list
			if (solaDefaultDeviceId && !devices.find(d => d.xDeviceId === solaDefaultDeviceId)) {
				solaDefaultDeviceId = "";
			}
		} catch (err) {
			console.error("[Sola] Error:", err);
			solaDevicesError = err instanceof Error ? err.message : "Failed to fetch devices";
			solaDevices = [];
		} finally {
			solaDevicesLoading = false;
		}
	}

	async function loadPrinters() {
		printersLoading = true;
		console.log("[Settings] Starting printer load...");

		try {
			// Add timeout to prevent infinite hanging
			const timeoutPromise = new Promise((_, reject) =>
				setTimeout(() => reject(new Error("Timeout loading printers")), 10000)
			);

			const printersPromise = getSystemPrinters();
			console.log("[Settings] Called getSystemPrinters()...");

			const printers = await Promise.race([printersPromise, timeoutPromise]) as PrinterInfo[];
			console.log("[Settings] Loaded printers:", printers);
			console.log("[Settings] Printers count:", printers.length);

			availablePrinters = printers;
			console.log("[Settings] availablePrinters after assignment:", availablePrinters);
		} catch (err) {
			console.error("Failed to load printers:", err);
			toast.error(`Failed to load printers: ${err instanceof Error ? err.message : String(err)}`);
			// Keep the seeded printer so saved selection is still visible
			availablePrinters = printerName ? [{ name: printerName, is_default: false }] : [];
		} finally {
			printersLoading = false;
			console.log("[Settings] Printer loading finished");
		}
	}

	async function saveSettings() {
		saving = true;
		saveMessage = "";

		try {
			// Encrypt the API key and iFields key before saving
			const encryptedApiKey = await encryptValue(solaApiKey);
			const encryptedIfieldsKey = await encryptValue(ifieldsKey);

			await Promise.all([
				updateSetting("store_logo", storeLogo),
				updateSetting("store_name", storeName),
				updateSetting("store_address", storeAddress),
				updateSetting("store_phone", storePhone),
				updateSetting("store_email", storeEmail),
				updateSetting("currency_symbol", currencySymbol),
				updateSetting("currency_code", currencyCode),
				updateSetting("default_tax_rate_bps", defaultTaxRateBps),
				updateSetting("tax_included_in_price", String(taxIncludedInPrice)),
				updateSetting("payment_methods_enabled", JSON.stringify(paymentMethodsEnabled)),
				updateSetting("receipt_header", receiptHeader),
				updateSetting("receipt_footer", receiptFooter),
				updateSetting("receipt_show_store_info", String(receiptShowStoreInfo)),
				updateSetting("low_stock_alert_enabled", String(lowStockAlertEnabled)),
				updateSetting("low_stock_default_threshold", lowStockDefaultThreshold),
				updateSetting("order_number_prefix", orderNumberPrefix),
				updateSetting("require_customer_on_order", String(requireCustomerOnOrder)),
				updateSetting("sola_gateway_card_present", String(solaCardPresent)),
				updateSetting("sola_gateway_card_not_present", String(solaCardNotPresent)),
				updateSetting("sola_gateway_api_key", encryptedApiKey),
				updateSetting("sola_gateway_default_device_id", solaDefaultDeviceId),
				updateSetting("enable_card_on_file", String(enableCardOnFile)),
				updateSetting("ifields_key", encryptedIfieldsKey),
				updateSetting("receipt_mode", receiptMode),
				updateSetting("printer_name", printerName),
				updateSetting("printer_type", printerType),
				updateSetting("paper_width_mm", paperWidthMm),
				updateSetting("enable_logging", String(enableLogging)),
				updateSetting("log_level", logLevel),
				updateSetting("log_retention_days", logRetentionDays),
				updateSetting("gateway_customer_sync_enabled", String(gatewaySyncEnabled)),
				updateSetting("gateway_sync_interval_minutes", gatewaySyncInterval)
				// Note: color_theme is auto-saved on change, not part of main save
			]);

			// Reload settings into store
			const all = await getAllSettings();
			settingsStore.load(all);

			log.info("settings", "Settings saved");
			saveMessage = "Settings saved successfully.";
			setTimeout(() => (saveMessage = ""), 3000);
		} catch {
			saveMessage = "Failed to save settings.";
		} finally {
			saving = false;
		}
	}

	const allPaymentMethods = [
		{ value: "cash", label: "Cash" },
		{ value: "check", label: "Check" },
		{ value: "credit_card", label: "Credit Card" }
	];

	async function handleSyncNow() {
		customerSyncing = true;
		lastSyncResult = "";
		try {
			const locked = await acquireSyncLock();
			if (!locked) {
				lastSyncResult = "Sync already in progress.";
				return;
			}

			const apiKey = await getDecryptedApiKey();
			if (!apiKey) {
				lastSyncResult = "API key not configured.";
				await releaseSyncLock();
				return;
			}

			const result = await syncCustomersWithGateway(apiKey);
			await releaseSyncLock();

			if (result.success) {
				lastSyncResult = `Done: ${result.created} created, ${result.updated} updated, ${result.pulled} pulled${result.errors > 0 ? `, ${result.errors} errors` : ""}`;
				if (result.errors > 0) {
					toast.warning(`Customer sync completed with ${result.errors} errors`);
				} else {
					toast.success("Customer sync completed successfully");
				}
			} else {
				lastSyncResult = result.errorMessage ?? "Sync failed";
				toast.error(result.errorMessage ?? "Customer sync failed");
			}
		} catch {
			lastSyncResult = "Sync failed unexpectedly";
			toast.error("Customer sync failed");
			try { await releaseSyncLock(); } catch {}
		} finally {
			customerSyncing = false;
		}
	}
</script>

<div class="p-6">
	<div class="mb-6 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<Settings class="h-6 w-6" />
			<h1 class="text-2xl font-semibold">Settings</h1>
		</div>
		<div class="flex items-center gap-3">
			{#if saveMessage}
				<span class="text-sm text-muted-foreground">{saveMessage}</span>
			{/if}
			<Button onclick={saveSettings} disabled={saving || !session.isAdmin}>
				{#if saving}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
					Saving...
				{:else}
					<Save class="mr-2 h-4 w-4" />
					Save Changes
				{/if}
			</Button>
		</div>
	</div>

	<Tabs bind:value={activeTab} class="w-full">
		<TabsList class="mb-4">
			<TabsTrigger value="store">Store Info</TabsTrigger>
			<TabsTrigger value="tax">Tax</TabsTrigger>
			<TabsTrigger value="payments">Payments</TabsTrigger>
			<TabsTrigger value="printing">Printing</TabsTrigger>
			<TabsTrigger value="inventory">Inventory</TabsTrigger>
			<TabsTrigger value="orders">Orders</TabsTrigger>
			<TabsTrigger value="appearance">Appearance</TabsTrigger>
			{#if session.isAdmin}
				<TabsTrigger value="employees">Employees</TabsTrigger>
				<TabsTrigger value="diagnostics">Diagnostics</TabsTrigger>
			{/if}
			<TabsTrigger value="about">About</TabsTrigger>
		</TabsList>

		<!-- Store Info -->
		<TabsContent value="store">
			<Card>
				<CardHeader>
					<CardTitle>Store Information</CardTitle>
					<CardDescription>Your business details displayed on receipts and throughout the app.</CardDescription>
				</CardHeader>
				<CardContent class="space-y-4">
					<div class="space-y-2">
						<p class="text-sm font-medium">Store Logo</p>
						<div class="flex items-center gap-4">
							{#if storeLogo}
								<div class="relative">
									<img
										src={storeLogo}
										alt="Store logo"
										class="h-16 w-16 rounded-lg border object-contain"
									/>
									<button
										type="button"
										class="absolute -right-1 -top-1 flex h-5 w-5 items-center justify-center rounded-full bg-destructive text-destructive-foreground"
										onclick={() => (storeLogo = "")}
									>
										<X class="h-3 w-3" />
									</button>
								</div>
							{:else}
								<div class="flex h-16 w-16 items-center justify-center rounded-lg border border-dashed bg-muted/50">
									<ImagePlus class="h-6 w-6 text-muted-foreground/50" />
								</div>
							{/if}
							<div>
								<input
									bind:this={logoFileInput}
									type="file"
									accept="image/*"
									class="hidden"
									onchange={handleLogoSelect}
								/>
								<Button variant="outline" size="sm" onclick={() => logoFileInput.click()}>
									<ImagePlus class="mr-2 h-4 w-4" />
									{storeLogo ? "Change Logo" : "Upload Logo"}
								</Button>
								<p class="mt-1 text-xs text-muted-foreground">Shown in the sidebar and on receipts.</p>
							</div>
						</div>
					</div>
					<Separator />
					<SettingField
						label="Store Name"
						value={storeName}
						placeholder="My Store"
						onchange={(v) => (storeName = v)}
					/>
					<SettingField
						label="Address"
						value={storeAddress}
						type="textarea"
						placeholder="123 Main St, City, ST 12345"
						onchange={(v) => (storeAddress = v)}
					/>
					<div class="grid grid-cols-2 gap-4">
						<SettingField
							label="Phone"
							value={storePhone}
							placeholder="(555) 123-4567"
							onchange={(v) => (storePhone = v)}
						/>
						<SettingField
							label="Email"
							value={storeEmail}
							placeholder="store@example.com"
							onchange={(v) => (storeEmail = v)}
						/>
					</div>
					<Separator />
					<div class="grid grid-cols-2 gap-4">
						<SettingField
							label="Currency Symbol"
							value={currencySymbol}
							placeholder="$"
							onchange={(v) => (currencySymbol = v)}
						/>
						<SettingField
							label="Currency Code"
							value={currencyCode}
							placeholder="USD"
							onchange={(v) => (currencyCode = v)}
						/>
					</div>
				</CardContent>
			</Card>
		</TabsContent>

		<!-- Tax -->
		<TabsContent value="tax">
			<Card>
				<CardHeader>
					<CardTitle>Tax Configuration</CardTitle>
					<CardDescription>Set up default tax rates for your products.</CardDescription>
				</CardHeader>
				<CardContent class="space-y-4">
					<SettingField
						label="Default Tax Rate (%)"
						description="Applied to new products. Enter as a percentage (e.g., 8.75 for 8.75%)."
						value={taxRateDisplay}
						type="number"
						placeholder="0.00"
						onchange={(v) => (defaultTaxRateBps = taxRateFromPercent(v))}
					/>
					<SettingToggle
						label="Tax Included in Price"
						description="When enabled, product prices already include tax."
						checked={taxIncludedInPrice}
						onchange={(v) => (taxIncludedInPrice = v)}
					/>
				</CardContent>
			</Card>
		</TabsContent>

		<!-- Payments -->
		<TabsContent value="payments">
			<div class="space-y-6">
				<Card>
					<CardHeader>
						<CardTitle>Payment Methods</CardTitle>
						<CardDescription>Choose which payment methods are available at checkout.</CardDescription>
					</CardHeader>
					<CardContent class="space-y-3">
						{#each allPaymentMethods as method}
							<SettingToggle
								label={method.label}
								checked={paymentMethodsEnabled.includes(method.value)}
								disabled={!session.isAdmin}
								onchange={() => togglePaymentMethod(method.value)}
							/>
						{/each}
					</CardContent>
				</Card>

				{#if paymentMethodsEnabled.includes("credit_card")}
					<Card>
						<CardHeader>
							<div class="flex items-center gap-2">
								<CreditCard class="h-5 w-5" />
								<CardTitle>Sola Gateway</CardTitle>
							</div>
							<CardDescription>Configure credit card processing through Sola Gateway.</CardDescription>
						</CardHeader>
						<CardContent class="space-y-4">
							<!-- SECTION 1: API Key (moved to top) -->
							<div class="grid gap-2">
								<Label>API Key</Label>
								<div class="flex gap-2">
									<Input
										type={solaApiKeyVisible ? "text" : "password"}
										value={session.isAdmin ? solaApiKey : maskApiKey(solaApiKey)}
										placeholder="Enter your Sola Gateway API key"
										oninput={(e) => (solaApiKey = e.currentTarget.value)}
										disabled={!session.isAdmin}
										class="font-mono"
									/>
									{#if session.isAdmin}
										<Button
											variant="outline"
											size="icon"
											onclick={() => (solaApiKeyVisible = !solaApiKeyVisible)}
											type="button"
										>
											{#if solaApiKeyVisible}
												<EyeOff class="h-4 w-4" />
											{:else}
												<Eye class="h-4 w-4" />
											{/if}
										</Button>
									{/if}
								</div>
								<p class="text-sm text-muted-foreground">
									Your API key is encrypted before being stored locally.
								</p>
							</div>

							<Separator />

							<!-- SECTION 2: Card Not Present (moved up) -->
							<SettingToggle
								label="Card Not Present"
								description="Accept card-not-present transactions by keying in the card number."
								checked={solaCardNotPresent}
								disabled={!session.isAdmin}
								onchange={(v) => (solaCardNotPresent = v)}
							/>

							{#if solaCardNotPresent}
								<div class="ml-6 space-y-2 rounded-lg border bg-muted/30 p-4">
									<div class="grid gap-2">
										<Label>iFields Key</Label>
										<div class="flex gap-2">
											<Input
												type={ifieldsKeyVisible ? "text" : "password"}
												value={session.isAdmin ? ifieldsKey : maskApiKey(ifieldsKey)}
												placeholder="Enter your iFields public key"
												oninput={(e) => (ifieldsKey = e.currentTarget.value)}
												disabled={!session.isAdmin}
												class="font-mono"
											/>
											{#if session.isAdmin}
												<Button
													variant="outline"
													size="icon"
													onclick={() => (ifieldsKeyVisible = !ifieldsKeyVisible)}
													type="button"
												>
													{#if ifieldsKeyVisible}
														<EyeOff class="h-4 w-4" />
													{:else}
														<Eye class="h-4 w-4" />
													{/if}
												</Button>
											{/if}
										</div>
										<p class="text-sm text-muted-foreground">
											Your iFields public key from the Sola Portal (Settings &gt; API Keys &gt; iFields). Encrypted before storage.
										</p>
									</div>
								</div>
							{/if}

							<!-- SECTION 3: Card Present (moved to bottom) -->
							<SettingToggle
								label="Card Present"
								description="Accept card-present transactions using a credit card terminal."
								checked={solaCardPresent}
								disabled={!session.isAdmin}
								onchange={(v) => (solaCardPresent = v)}
							/>

							<Separator />

							<!-- Card on File -->
							<SettingToggle
								label="Card on File"
								description="Save and reuse customer card tokens for faster repeat transactions."
								checked={enableCardOnFile}
								disabled={!session.isAdmin || !solaApiKey.trim()}
								onchange={(v) => (enableCardOnFile = v)}
							/>
							{#if enableCardOnFile && !solaApiKey.trim()}
								<p class="ml-6 text-sm text-destructive">
									Requires API Key to be configured.
								</p>
							{/if}

							<!-- SECTION 4: Terminal Selection (NEW - only visible when Card Present enabled) -->
							{#if solaCardPresent}
								<div class="ml-6 space-y-3 rounded-lg border bg-muted/30 p-4">
									<div class="flex items-center justify-between">
										<Label class="text-sm font-medium">Payment Terminal</Label>
										{#if !solaApiKey.trim()}
											<span class="text-xs text-muted-foreground">API Key required</span>
										{:else}
											<div class="flex gap-2">
												{#if session.isAdmin}
													<Button
														variant="outline"
														size="sm"
														onclick={() => (addDeviceDialogOpen = true)}
														disabled={solaDevicesLoading}
													>
														<Plus class="mr-2 h-3 w-3" />
														Add Device
													</Button>
												{/if}
												<Button
													variant="outline"
													size="sm"
													onclick={fetchSolaDevices}
													disabled={solaDevicesLoading || !solaApiKey.trim() || !session.isAdmin}
												>
													{#if solaDevicesLoading}
														<Loader2 class="mr-2 h-3 w-3 animate-spin" />
														Loading...
													{:else}
														Refresh Devices
													{/if}
												</Button>
											</div>
										{/if}
									</div>

									{#if !solaApiKey.trim()}
										<p class="text-sm text-muted-foreground">
											Enter an API key above to view available devices.
										</p>
									{:else if solaDevicesError}
										<div class="rounded-md bg-destructive/10 p-2 text-sm text-destructive">
											{solaDevicesError}
										</div>
									{:else if solaDevices.length > 0}
										<div class="grid gap-2">
											<div class="relative rounded-md border">
												{#if solaDevicesLoading}
													<div class="absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center rounded-md z-10">
														<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
													</div>
												{/if}
												<table class="w-full text-sm">
													<thead>
														<tr class="border-b bg-muted/50">
															<th class="p-2 text-left font-medium">Default</th>
															<th class="p-2 text-left font-medium">Status</th>
															<th class="p-2 text-left font-medium">Device Name</th>
															<th class="p-2 text-left font-medium">Serial Number</th>
															<th class="p-2 text-left font-medium">Device ID</th>
															<th class="p-2 text-left font-medium">Actions</th>
														</tr>
													</thead>
													<tbody>
														{#each solaDevices as device}
															<tr class="border-b last:border-0 hover:bg-muted/50 transition-colors">
																<td class="p-2">
																	<input
																		type="radio"
																		name="default-device"
																		checked={solaDefaultDeviceId === device.xDeviceId}
																		onchange={() => (solaDefaultDeviceId = device.xDeviceId)}
																		class="cursor-pointer"
																	/>
																</td>
																<td class="p-2">
																	<div class="flex items-center gap-1.5">
																		<Wifi
																			class="h-4 w-4 {device.xDeviceStatus === 'CONNECTED'
																				? 'text-green-500'
																				: 'text-muted-foreground'}"
																		/>
																		<span class="text-xs {device.xDeviceStatus === 'CONNECTED' ? 'text-green-500' : 'text-muted-foreground'}">
																			{device.xDeviceStatus}
																		</span>
																	</div>
																</td>
																<td class="p-2">
																	{#if editingDeviceId === device.xDeviceId}
																		<Input
																			type="text"
																			value={editingDeviceName}
																			oninput={(e) => (editingDeviceName = e.currentTarget.value)}
																			onkeydown={(e) => {
																				if (e.key === "Enter") {
																					updateDeviceName(device.xDeviceId, editingDeviceName);
																				} else if (e.key === "Escape") {
																					editingDeviceId = null;
																					editingDeviceName = "";
																				}
																			}}
																			onblur={() => {
																				if (editingDeviceName.trim() && editingDeviceName !== device.xDeviceFriendlyName) {
																					updateDeviceName(device.xDeviceId, editingDeviceName);
																				} else {
																					editingDeviceId = null;
																					editingDeviceName = "";
																				}
																			}}
																			class="h-8"
																			autofocus
																		/>
																	{:else}
																		<span class="font-medium">{device.xDeviceFriendlyName}</span>
																	{/if}
																</td>
																<td class="p-2 font-mono text-xs">{device.xDeviceSerialNumber}</td>
																<td class="p-2 text-muted-foreground font-mono text-xs">
																	{session.isAdmin ? device.xDeviceId : maskDeviceId(device.xDeviceId)}
																</td>
																<td class="p-2">
																	{#if session.isAdmin}
																		<div class="flex items-center gap-1">
																			{#if editingDeviceId === device.xDeviceId}
																				<Button
																					variant="ghost"
																					size="icon"
																					class="h-7 w-7"
																					disabled={solaDevicesLoading}
																					onclick={() => updateDeviceName(device.xDeviceId, editingDeviceName)}
																				>
																					<Check class="h-3.5 w-3.5" />
																				</Button>
																			{:else}
																				<Button
																					variant="ghost"
																					size="icon"
																					class="h-7 w-7"
																					disabled={solaDevicesLoading}
																					onclick={() => {
																						editingDeviceId = device.xDeviceId;
																						editingDeviceName = device.xDeviceFriendlyName;
																					}}
																				>
																					<Pencil class="h-3.5 w-3.5" />
																				</Button>
																				<Button
																					variant="ghost"
																					size="icon"
																					class="h-7 w-7 text-destructive hover:text-destructive"
																					disabled={solaDevicesLoading}
																					onclick={() => {
																						deviceToDelete = device;
																						deleteDialogOpen = true;
																					}}
																				>
																					<Trash2 class="h-3.5 w-3.5" />
																				</Button>
																			{/if}
																		</div>
																	{:else}
																		<span class="text-xs text-muted-foreground">—</span>
																	{/if}
																</td>
															</tr>
														{/each}
													</tbody>
												</table>
											</div>
											<p class="text-xs text-muted-foreground">
												{solaDevices.length} device{solaDevices.length !== 1 ? 's' : ''} found
												{#if solaDefaultDeviceId}
													• Default: {solaDevices.find(d => d.xDeviceId === solaDefaultDeviceId)?.xDeviceFriendlyName}
												{/if}
											</p>
										</div>
									{:else if !solaDevicesLoading}
										<p class="text-sm text-muted-foreground">
											Click "Refresh Devices" to scan for available terminals.
										</p>
									{/if}
								</div>
							{/if}
						</CardContent>
					</Card>
				{/if}

				<!-- Delete Device Confirmation Dialog -->
				<AlertDialog bind:open={deleteDialogOpen}>
					<AlertDialogContent>
						<AlertDialogHeader>
							<AlertDialogTitle>Delete Device</AlertDialogTitle>
							<AlertDialogDescription>
								Are you sure you want to delete <strong>{deviceToDelete?.xDeviceFriendlyName}</strong>?
								This action cannot be undone.
							</AlertDialogDescription>
						</AlertDialogHeader>
						<AlertDialogFooter>
							<AlertDialogCancel onclick={() => {
								deleteDialogOpen = false;
								deviceToDelete = null;
							}}>
								Cancel
							</AlertDialogCancel>
							<AlertDialogAction
								onclick={() => {
									if (deviceToDelete) {
										deleteDevice(deviceToDelete.xDeviceId);
									}
								}}
								class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
							>
								Delete
							</AlertDialogAction>
						</AlertDialogFooter>
					</AlertDialogContent>
				</AlertDialog>

				<!-- Add Device Dialog -->
				<AlertDialog bind:open={addDeviceDialogOpen}>
					<AlertDialogContent>
						<AlertDialogHeader>
							<AlertDialogTitle>Add New Device</AlertDialogTitle>
							<AlertDialogDescription>
								Enter the device details to register a new terminal. The serial number can be found on the physical device.
							</AlertDialogDescription>
						</AlertDialogHeader>
						<div class="grid gap-4 py-4">
							<div class="grid gap-2">
								<Label for="deviceSerial">Serial Number</Label>
								<Input
									id="deviceSerial"
									type="text"
									placeholder="8-16 digits (e.g., 12345678)"
									value={newDeviceSerialNumber}
									oninput={(e) => (newDeviceSerialNumber = e.currentTarget.value)}
									disabled={addingDevice}
								/>
								<p class="text-xs text-muted-foreground">Enter the device serial number (8-16 digits)</p>
							</div>
							<div class="grid gap-2">
								<Label for="deviceMake">Device Make</Label>
								<select
									id="deviceMake"
									class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
									bind:value={newDeviceMake}
									disabled={addingDevice}
								>
									<option value="pax">PAX</option>
									<option value="ingenico">Ingenico</option>
									<option value="verifone">Verifone</option>
								</select>
								<p class="text-xs text-muted-foreground">Select the device manufacturer</p>
							</div>
							<div class="grid gap-2">
								<Label for="deviceName">Device Name</Label>
								<Input
									id="deviceName"
									type="text"
									placeholder="e.g., Front Counter Terminal"
									value={newDeviceName}
									oninput={(e) => (newDeviceName = e.currentTarget.value)}
									disabled={addingDevice}
								/>
								<p class="text-xs text-muted-foreground">Choose a friendly name for this device</p>
							</div>
						</div>
						<AlertDialogFooter>
							<AlertDialogCancel
								onclick={() => {
									addDeviceDialogOpen = false;
									newDeviceSerialNumber = "";
									newDeviceMake = "pax";
									newDeviceName = "";
									solaDevicesError = "";
								}}
								disabled={addingDevice}
							>
								Cancel
							</AlertDialogCancel>
							<AlertDialogAction
								onclick={createDevice}
								disabled={addingDevice || !newDeviceSerialNumber.trim() || !newDeviceName.trim()}
							>
								{#if addingDevice}
									<Loader2 class="mr-2 h-4 w-4 animate-spin" />
									Adding...
								{:else}
									Add Device
								{/if}
							</AlertDialogAction>
						</AlertDialogFooter>
					</AlertDialogContent>
				</AlertDialog>
			</div>

			<!-- Customer Sync (only visible when API key is configured) -->
			{#if solaApiKey.trim()}
				<Card class="mt-6">
					<CardHeader>
						<div class="flex items-center gap-2">
							<CardTitle>Customer Sync</CardTitle>
						</div>
						<CardDescription>Sync customer records with the Sola gateway for recurring billing.</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						<SettingToggle
							label="Auto-sync Customers"
							description="Automatically sync customer records with the gateway in the background."
							checked={gatewaySyncEnabled}
							disabled={!session.isAdmin}
							onchange={(v) => (gatewaySyncEnabled = v)}
						/>

						{#if gatewaySyncEnabled}
							<div class="grid gap-2">
								<Label>Sync Interval (minutes)</Label>
								<Input
									type="number"
									min={5}
									max={1440}
									value={gatewaySyncInterval}
									oninput={(e) => (gatewaySyncInterval = e.currentTarget.value)}
									disabled={!session.isAdmin}
									class="max-w-32"
								/>
								<p class="text-xs text-muted-foreground">How often to sync (5-1440 minutes)</p>
							</div>
						{/if}

						<Separator />

						<div class="flex items-center gap-3">
							<Button
								variant="outline"
								onclick={handleSyncNow}
								disabled={customerSyncing || !session.isAdmin}
							>
								{#if customerSyncing}
									<Loader2 class="mr-2 h-4 w-4 animate-spin" />
									Syncing...
								{:else}
									Sync Now
								{/if}
							</Button>
							{#if lastSyncResult}
								<p class="text-sm text-muted-foreground">
									{lastSyncResult}
								</p>
							{/if}
						</div>
					</CardContent>
				</Card>
			{/if}
		</TabsContent>

		<!-- Printing -->
		<TabsContent value="printing">
			<!-- Receipt Settings -->
			<Card class="mb-4">
				<CardHeader>
					<CardTitle>Receipt Settings</CardTitle>
					<CardDescription>Configure receipt printing behavior and content.</CardDescription>
				</CardHeader>
				<CardContent class="space-y-4">
					<!-- Receipt Mode: three-way selector -->
					<div class="space-y-2">
						<Label>Receipt Printing</Label>
						<div class="grid grid-cols-3 gap-2">
							<button
								type="button"
								class="flex flex-col items-center gap-1.5 rounded-lg border-2 p-3 transition-colors {receiptMode === 'off' ? 'border-primary bg-primary/5' : 'border-muted hover:border-muted-foreground/30'}"
								onclick={() => (receiptMode = "off")}
							>
								<span class="text-sm font-medium">Off</span>
								<span class="text-xs text-muted-foreground text-center">No receipts printed</span>
							</button>
							<button
								type="button"
								class="flex flex-col items-center gap-1.5 rounded-lg border-2 p-3 transition-colors {receiptMode === 'prompt' ? 'border-primary bg-primary/5' : 'border-muted hover:border-muted-foreground/30'}"
								onclick={() => (receiptMode = "prompt")}
							>
								<span class="text-sm font-medium">Preview First</span>
								<span class="text-xs text-muted-foreground text-center">Show receipt before printing</span>
							</button>
							<button
								type="button"
								class="flex flex-col items-center gap-1.5 rounded-lg border-2 p-3 transition-colors {receiptMode === 'auto' ? 'border-primary bg-primary/5' : 'border-muted hover:border-muted-foreground/30'}"
								onclick={() => (receiptMode = "auto")}
							>
								<span class="text-sm font-medium">Automatic</span>
								<span class="text-xs text-muted-foreground text-center">Print after every sale</span>
							</button>
						</div>
						<p class="text-xs text-muted-foreground">
							Choose how receipts are handled after completing a sale.
						</p>
					</div>

					<Separator />

					<SettingToggle
						label="Show Store Info"
						description="Display store name, address, and contact info on receipts."
						checked={receiptShowStoreInfo}
						onchange={(v) => (receiptShowStoreInfo = v)}
					/>
					<SettingField
						label="Receipt Header"
						description="Custom text shown at the top of receipts."
						value={receiptHeader}
						type="textarea"
						placeholder="Welcome to our store!"
						onchange={(v) => (receiptHeader = v)}
					/>
					<SettingField
						label="Receipt Footer"
						description="Custom text shown at the bottom of receipts."
						value={receiptFooter}
						type="textarea"
						placeholder="Thank you for your purchase!"
						onchange={(v) => (receiptFooter = v)}
					/>
				</CardContent>
			</Card>

			<!-- Printer Hardware (only shown when printing is enabled) -->
			{#if receiptMode !== "off"}
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Printer class="h-5 w-5" />
							Printer Hardware
						</CardTitle>
						<CardDescription>Select and configure your receipt printer.</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						<!-- Printer Selection -->
						<div class="space-y-2">
							<Label for="printer-name">Select Printer</Label>
							<div class="flex gap-2">
								<select
									id="printer-name"
									bind:value={printerName}
									class="flex-1 h-10 rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
								>
									<option value="">Select printer...</option>
									{#each availablePrinters as printer, i (i)}
										<option value={printer.name}>
											{printer.name}{#if printer.is_default} (Default){/if}
										</option>
									{/each}
								</select>
								<Button
									variant="outline"
									onclick={loadPrinters}
									disabled={printersLoading}
								>
									{#if printersLoading}
										<Loader2 class="h-4 w-4 animate-spin" />
									{:else}
										Refresh
									{/if}
								</Button>
							</div>
							<p class="text-xs text-muted-foreground">
								Select your receipt printer from the list of installed printers.
							</p>
						</div>

						<!-- Printer Type -->
						<div class="space-y-2">
							<Label for="printer-type">Printer Type</Label>
							<select
								id="printer-type"
								bind:value={printerType}
								class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
							>
								<option value="standard">Standard Printer</option>
								<option value="thermal">Thermal Receipt Printer (Epson, Star)</option>
							</select>
							<p class="text-xs text-muted-foreground">
								Choose "Thermal" for dedicated receipt printers like Epson TM-T20 or Star TSP100.
							</p>
						</div>

						<!-- Paper Width -->
						<div class="space-y-2">
							<Label for="paper-width">Paper Width (mm)</Label>
							<select
								id="paper-width"
								bind:value={paperWidthMm}
								class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
							>
								<option value="58">58mm (2 inch)</option>
								<option value="80">80mm (3 inch)</option>
							</select>
							<p class="text-xs text-muted-foreground">
								Standard thermal receipt printers use 80mm paper.
							</p>
						</div>

						<!-- Test Print Button -->
						<div class="pt-2">
							<Button
								variant="outline"
								onclick={() => (testPrintOpen = true)}
								disabled={!printerName}
							>
								<Printer class="h-4 w-4 mr-2" />
								Test Print
							</Button>
						</div>
					</CardContent>
				</Card>
			{/if}
		</TabsContent>

		<!-- Inventory -->
		<TabsContent value="inventory">
			<Card>
				<CardHeader>
					<CardTitle>Inventory Settings</CardTitle>
					<CardDescription>Configure stock tracking and alerts.</CardDescription>
				</CardHeader>
				<CardContent class="space-y-4">
					<SettingToggle
						label="Low Stock Alerts"
						description="Show alerts when product stock falls below the threshold."
						checked={lowStockAlertEnabled}
						onchange={(v) => (lowStockAlertEnabled = v)}
					/>
					<SettingField
						label="Default Low Stock Threshold"
						description="Default minimum stock level before triggering an alert."
						value={lowStockDefaultThreshold}
						type="number"
						placeholder="5"
						onchange={(v) => (lowStockDefaultThreshold = v)}
					/>
				</CardContent>
			</Card>
		</TabsContent>

		<!-- Orders -->
		<TabsContent value="orders">
			<Card>
				<CardHeader>
					<CardTitle>Order Settings</CardTitle>
					<CardDescription>Configure order numbering and requirements.</CardDescription>
				</CardHeader>
				<CardContent class="space-y-4">
					<SettingField
						label="Order Number Prefix"
						description="Prefix for generated order numbers (e.g., ORD-00001)."
						value={orderNumberPrefix}
						placeholder="ORD-"
						onchange={(v) => (orderNumberPrefix = v)}
					/>
					<SettingToggle
						label="Require Customer on Orders"
						description="When enabled, a customer must be selected before completing an order."
						checked={requireCustomerOnOrder}
						onchange={(v) => (requireCustomerOnOrder = v)}
					/>
				</CardContent>
			</Card>
		</TabsContent>

		<!-- Appearance -->
		<TabsContent value="appearance">
			<div class="space-y-6">
				<Card>
					<CardHeader>
						<CardTitle>Brightness</CardTitle>
						<CardDescription>Choose light or dark mode, or sync with your system settings.</CardDescription>
					</CardHeader>
					<CardContent>
						<div class="flex gap-3">
							<button
								type="button"
								class="flex flex-1 flex-col items-center gap-2 rounded-lg border-2 p-4 transition-colors {mode.current === 'light' ? 'border-primary bg-primary/5' : 'border-muted hover:border-muted-foreground/30'}"
								onclick={() => handleBrightnessChange("light")}
							>
								<Sun class="h-6 w-6" />
								<span class="text-sm font-medium">Light</span>
							</button>
							<button
								type="button"
								class="flex flex-1 flex-col items-center gap-2 rounded-lg border-2 p-4 transition-colors {mode.current === 'dark' ? 'border-primary bg-primary/5' : 'border-muted hover:border-muted-foreground/30'}"
								onclick={() => handleBrightnessChange("dark")}
							>
								<Moon class="h-6 w-6" />
								<span class="text-sm font-medium">Dark</span>
							</button>
							<button
								type="button"
								class="flex flex-1 flex-col items-center gap-2 rounded-lg border-2 p-4 transition-colors {mode.current !== 'light' && mode.current !== 'dark' ? 'border-primary bg-primary/5' : 'border-muted hover:border-muted-foreground/30'}"
								onclick={() => handleBrightnessChange("system")}
							>
								<Monitor class="h-6 w-6" />
								<span class="text-sm font-medium">System</span>
							</button>
						</div>
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<CardTitle>Color Scheme</CardTitle>
						<CardDescription>Choose your preferred color theme. Each theme adapts to light and dark mode.</CardDescription>
					</CardHeader>
					<CardContent>
						<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
							{#each availableThemes as theme}
								<button
									type="button"
									class="flex flex-col gap-2 rounded-lg border-2 p-4 text-left transition-colors {selectedTheme === theme.id ? 'border-primary bg-primary/5' : 'border-muted hover:border-muted-foreground/30'}"
									onclick={() => handleThemeChange(theme.id)}
								>
									<div class="flex items-center justify-between">
										<span class="font-medium">{theme.name}</span>
										{#if selectedTheme === theme.id}
											<div class="h-2 w-2 rounded-full bg-primary"></div>
										{/if}
									</div>
									<p class="text-xs text-muted-foreground">{theme.description}</p>
									<div class="mt-1 flex gap-1.5">
										<!-- Color preview dots -->
										<div class="h-4 w-4 rounded-full border" style="background-color: {theme.light.primary}"></div>
										<div class="h-4 w-4 rounded-full border" style="background-color: {theme.light.secondary}"></div>
										<div class="h-4 w-4 rounded-full border" style="background-color: {theme.light.accent}"></div>
									</div>
								</button>
							{/each}
						</div>
					</CardContent>
				</Card>
			</div>
		</TabsContent>

		<!-- Employees -->
		<TabsContent value="employees">
			<EmployeeManagement />
		</TabsContent>

		<!-- Diagnostics -->
		<TabsContent value="diagnostics">
			<Card>
				<CardHeader>
					<CardTitle>Diagnostics</CardTitle>
					<CardDescription>Configure application logging and view system logs.</CardDescription>
				</CardHeader>
				<CardContent class="space-y-4">
					<SettingToggle
						label="Enable Logging"
						description="Record application events for troubleshooting and diagnostics."
						checked={enableLogging}
						onchange={(v) => (enableLogging = v)}
					/>

					<div class="space-y-2">
						<Label>Log Level</Label>
						<p class="text-sm text-muted-foreground">Minimum severity level to record. Lower levels include all higher severity events.</p>
						<select
							class="h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
							bind:value={logLevel}
						>
							<option value="error">Error only</option>
							<option value="warn">Warning and above</option>
							<option value="info">Info and above</option>
							<option value="debug">Debug (all events)</option>
						</select>
					</div>

					<div class="space-y-2">
						<Label>Log Retention (days)</Label>
						<p class="text-sm text-muted-foreground">Log files older than this will be automatically deleted. Range: 7-90 days.</p>
						<Input
							type="number"
							min="7"
							max="90"
							bind:value={logRetentionDays}
						/>
					</div>

					<Separator />

					<div class="flex items-center gap-3">
						<Button onclick={() => (logViewerOpen = true)}>
							<FileText class="mr-2 h-4 w-4" />
							View Logs
						</Button>
					</div>
				</CardContent>
			</Card>
		</TabsContent>

		<!-- About -->
		<TabsContent value="about">
			<div class="space-y-6">
				<Card>
					<CardHeader>
						<CardTitle>About MM Lite POS</CardTitle>
						<CardDescription>Application information and version details</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid gap-3">
							<div class="flex justify-between items-center py-2">
								<span class="text-sm font-medium">Application Name</span>
								<span class="text-sm text-muted-foreground">MM Lite POS</span>
							</div>
							<Separator />
							<div class="flex justify-between items-center py-2">
								<span class="text-sm font-medium">Version</span>
								<span class="text-sm text-muted-foreground font-mono">0.1.0</span>
							</div>
							<Separator />
							<div class="flex justify-between items-center py-2">
								<span class="text-sm font-medium">Build Date</span>
								<span class="text-sm text-muted-foreground">{new Date().toLocaleDateString()}</span>
							</div>
						</div>
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<CardTitle>System Information</CardTitle>
						<CardDescription>Details about your installation</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid gap-3">
							<div class="flex justify-between items-center py-2">
								<span class="text-sm font-medium">Platform</span>
								<span class="text-sm text-muted-foreground">{navigator.platform}</span>
							</div>
							<Separator />
							<div class="flex justify-between items-center py-2">
								<span class="text-sm font-medium">User Agent</span>
								<span class="text-sm text-muted-foreground text-right max-w-md truncate">{navigator.userAgent}</span>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>
		</TabsContent>
	</Tabs>
</div>

<!-- Test Print Receipt Dialog -->
<PrintReceiptDialog
	bind:open={testPrintOpen}
	order={testOrder}
	items={testItems}
	payments={testPayments}
	customer={null}
	storeName={storeName}
	storeAddress={storeAddress}
	storePhone={storePhone}
	taxLabel="Tax"
	currencySymbol={currencySymbol}
	printerType={printerType as "standard" | "thermal"}
	{printerName}
	onClose={() => (testPrintOpen = false)}
/>

<!-- Log Viewer Dialog -->
<LogViewerDialog bind:open={logViewerOpen} onClose={() => (logViewerOpen = false)} />
