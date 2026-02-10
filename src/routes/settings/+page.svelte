<script lang="ts">
	import { Settings, Save, Loader2, Sun, Moon, Monitor, ImagePlus, X } from "lucide-svelte";
	import { Tabs, TabsContent, TabsList, TabsTrigger } from "$lib/components/ui/tabs/index.js";
	import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import SettingField from "$lib/components/settings/SettingField.svelte";
	import SettingToggle from "$lib/components/settings/SettingToggle.svelte";
	import EmployeeManagement from "$lib/components/settings/EmployeeManagement.svelte";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import { updateSetting, getAllSettings } from "$lib/commands/settings.js";
	import { session } from "$lib/stores/session.svelte.js";
	import { mode, setMode } from "mode-watcher";
	import { readFileAsDataUrl } from "$lib/utils.js";

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

	let lowStockAlertEnabled = $state(settingsStore.getBoolean("low_stock_alert_enabled"));
	let lowStockDefaultThreshold = $state(settingsStore.get("low_stock_default_threshold"));

	let orderNumberPrefix = $state(settingsStore.get("order_number_prefix"));
	let requireCustomerOnOrder = $state(settingsStore.getBoolean("require_customer_on_order"));

	let saving = $state(false);
	let saveMessage = $state("");

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
	const taxRateDisplay = $derived(() => {
		const bps = parseInt(defaultTaxRateBps || "0", 10);
		return (bps / 100).toFixed(2);
	});

	function taxRateFromPercent(percent: string): string {
		const val = parseFloat(percent) || 0;
		return String(Math.round(val * 100));
	}

	async function saveSettings() {
		saving = true;
		saveMessage = "";

		try {
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
				updateSetting("require_customer_on_order", String(requireCustomerOnOrder))
			]);

			// Reload settings into store
			const all = await getAllSettings();
			settingsStore.load(all);

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
			<Button onclick={saveSettings} disabled={saving}>
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

	<Tabs value="store" class="w-full">
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
			{/if}
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
						value={taxRateDisplay()}
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
							onchange={() => togglePaymentMethod(method.value)}
						/>
					{/each}
				</CardContent>
			</Card>
		</TabsContent>

		<!-- Printing -->
		<TabsContent value="printing">
			<Card>
				<CardHeader>
					<CardTitle>Receipt Settings</CardTitle>
					<CardDescription>Customize what appears on printed receipts.</CardDescription>
				</CardHeader>
				<CardContent class="space-y-4">
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
			<Card>
				<CardHeader>
					<CardTitle>Theme</CardTitle>
					<CardDescription>Choose how the app looks. Select a theme preference below.</CardDescription>
				</CardHeader>
				<CardContent>
					<div class="flex gap-3">
						<button
							type="button"
							class="flex flex-1 flex-col items-center gap-2 rounded-lg border-2 p-4 transition-colors {mode.current === 'light' ? 'border-primary bg-primary/5' : 'border-muted hover:border-muted-foreground/30'}"
							onclick={() => setMode("light")}
						>
							<Sun class="h-6 w-6" />
							<span class="text-sm font-medium">Light</span>
						</button>
						<button
							type="button"
							class="flex flex-1 flex-col items-center gap-2 rounded-lg border-2 p-4 transition-colors {mode.current === 'dark' ? 'border-primary bg-primary/5' : 'border-muted hover:border-muted-foreground/30'}"
							onclick={() => setMode("dark")}
						>
							<Moon class="h-6 w-6" />
							<span class="text-sm font-medium">Dark</span>
						</button>
						<button
							type="button"
							class="flex flex-1 flex-col items-center gap-2 rounded-lg border-2 p-4 transition-colors {mode.current !== 'light' && mode.current !== 'dark' ? 'border-primary bg-primary/5' : 'border-muted hover:border-muted-foreground/30'}"
							onclick={() => setMode("system")}
						>
							<Monitor class="h-6 w-6" />
							<span class="text-sm font-medium">System</span>
						</button>
					</div>
				</CardContent>
			</Card>
		</TabsContent>

		<!-- Employees -->
		<TabsContent value="employees">
			<EmployeeManagement />
		</TabsContent>
	</Tabs>
</div>
