<script lang="ts">
	import { Banknote, CreditCard, FileCheck, Delete, Loader2, RefreshCw } from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle
	} from "$lib/components/ui/dialog/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { formatCurrency } from "$lib/utils.js";
	import { toast } from "svelte-sonner";
	import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import { decryptValue } from "$lib/commands/crypto.js";
	import { processSolaTransaction } from "$lib/commands/sola.js";
	import DeviceSelector from "./DeviceSelector.svelte";
	import TransactionResultDialog from "./TransactionResultDialog.svelte";
	import type {
		PaymentMethod,
		PartialPayment,
		SolaDevice,
		SolaDeviceListResponse,
		SolaTransactionResponse
	} from "$lib/types/index.js";

	interface Props {
		open: boolean;
		totalCents: number;
		currencySymbol: string;
		invoiceNumber: string;
		onComplete: (payments: PartialPayment[]) => void;
		onCancel: () => void;
	}

	let { open = $bindable(), totalCents, currencySymbol, invoiceNumber, onComplete, onCancel }: Props = $props();

	// Payment method state
	let selectedMethod = $state<PaymentMethod>("cash");
	let cardPaymentType = $state<"card_present" | "card_not_present" | null>(null);

	// Cash payment state
	let amountInput = $state("");

	// Card present state
	let selectedDeviceId = $state("");
	let devices = $state<SolaDevice[]>([]);
	let loadingDevices = $state(false);
	let processing = $state(false);

	// Transaction result state
	let transactionResult = $state<SolaTransactionResponse | null>(null);
	let transactionResultOpen = $state(false);

	// Partial payments tracking
	let partialPayments = $state<PartialPayment[]>([]);

	// Derived values
	const amountCents = $derived(() => {
		if (!amountInput) return 0;
		const val = parseFloat(amountInput);
		return isNaN(val) ? 0 : Math.round(val * 100);
	});

	const changeCents = $derived(() => {
		const paid = amountCents();
		return Math.max(0, paid - remainingCents());
	});

	const totalPaidCents = $derived(() => {
		return partialPayments.reduce((sum, p) => sum + p.amountCents, 0);
	});

	const remainingCents = $derived(() => {
		return totalCents - totalPaidCents();
	});

	const canCompleteCash = $derived(() => {
		return amountCents() >= remainingCents();
	});

	const canSendToDevice = $derived(() => {
		return (
			cardPaymentType === "card_present" &&
			selectedDeviceId &&
			amountCents() > 0 &&
			amountCents() <= remainingCents() &&
			!processing
		);
	});

	// Quick amount suggestions based on remaining balance
	const quickAmounts = $derived(() => {
		const remainingDollars = remainingCents() / 100;
		const suggestions: number[] = [];

		// Round up to nearest 5, 10, 20, 50
		const roundups = [5, 10, 20, 50, 100];
		for (const r of roundups) {
			const rounded = Math.ceil(remainingDollars / r) * r;
			if (rounded >= remainingDollars && !suggestions.includes(rounded)) {
				suggestions.push(rounded);
			}
			if (suggestions.length >= 4) break;
		}

		return suggestions;
	});

	// Cash numpad functions
	function appendDigit(digit: string) {
		if (digit === "." && amountInput.includes(".")) return;
		if (digit === "." && amountInput === "") {
			amountInput = "0.";
			return;
		}
		// Limit to 2 decimal places
		const parts = amountInput.split(".");
		if (parts[1] && parts[1].length >= 2) return;
		amountInput += digit;
	}

	function deleteDigit() {
		amountInput = amountInput.slice(0, -1);
	}

	function setExact() {
		amountInput = (remainingCents() / 100).toFixed(2);
	}

	function setQuickAmount(dollars: number) {
		amountInput = dollars.toFixed(2);
	}

	// Method selection
	function selectMethod(method: PaymentMethod) {
		selectedMethod = method;

		if (method === "credit_card") {
			// Default to card present
			const cardPresentEnabled = settingsStore.getBoolean("sola_gateway_card_present");
			if (cardPresentEnabled) {
				cardPaymentType = "card_present";
				setExact();
				if (devices.length === 0) {
					loadDevices();
				}
			} else {
				// Fall back to card not present if card present not enabled
				cardPaymentType = "card_not_present";
				setExact();
			}
		} else {
			cardPaymentType = null;
			if (method !== "cash") {
				// Auto-set exact amount for check
				setExact();
			} else {
				amountInput = "";
			}
		}
	}

	function selectCardType(type: "card_present" | "card_not_present") {
		cardPaymentType = type;
		setExact();

		if (type === "card_present" && devices.length === 0) {
			loadDevices();
		}
	}

	// Load devices from Sola Gateway
	async function loadDevices() {
		loadingDevices = true;
		try {
			const apiKey = settingsStore.get("sola_gateway_api_key");
			if (!apiKey) {
				toast.error("Sola Gateway API key not configured");
				return;
			}

			const decryptedKey = await decryptValue(apiKey);
			if (!decryptedKey) {
				toast.error("Failed to decrypt API key");
				return;
			}

			const response = await tauriFetch("https://device.cardknox.com/v1/Device", {
				method: "GET",
				headers: {
					Authorization: decryptedKey.trim()
				}
			});

			if (!response.ok) {
				throw new Error(`HTTP error ${response.status}`);
			}

			const data: SolaDeviceListResponse = await response.json();

			if (data.xResult === "E") {
				throw new Error("Failed to fetch devices from Sola Gateway");
			}

			devices = data.xDevices || [];

			// Auto-select default device
			const defaultDeviceId = settingsStore.get("sola_gateway_default_device_id");
			if (defaultDeviceId && devices.some((d) => d.xDeviceId === defaultDeviceId)) {
				selectedDeviceId = defaultDeviceId;
			} else if (devices.length === 1) {
				selectedDeviceId = devices[0].xDeviceId;
			}
		} catch (err) {
			console.error("Failed to load devices:", err);
			toast.error("Failed to load devices");
		} finally {
			loadingDevices = false;
		}
	}

	// Send transaction to device
	async function handleSendToDevice() {
		processing = true;
		try {
			const apiKey = settingsStore.get("sola_gateway_api_key");
			if (!apiKey) {
				toast.error("Sola Gateway API key not configured");
				return;
			}

			const decryptedKey = await decryptValue(apiKey);
			if (!decryptedKey) {
				toast.error("Failed to decrypt API key");
				return;
			}

			const result = await processSolaTransaction({
				apiKey: decryptedKey,
				deviceId: selectedDeviceId,
				amountCents: amountCents(),
				invoiceNumber
			});

			transactionResult = result;
			transactionResultOpen = true;

			if (result.xResult === "A") {
				// Approved - add to partial payments
				const lastFour = result.xMaskedCardNumber?.slice(-4) || "";

				partialPayments.push({
					method: "credit_card",
					amountCents: amountCents(),
					changeCents: 0,
					cardDetails: {
						authCode: result.xAuthCode || "",
						lastFour,
						cardType: result.xCardType || "",
						entryMode: "card_present",
						gatewayRefNum: result.xRefnum,
						gatewayResponse: JSON.stringify(result)
					}
				});

				// Check if fully paid
				if (remainingCents() === 0) {
					// Close result dialog and complete order
					transactionResultOpen = false;
					onComplete(partialPayments);
				} else {
					// Reset for next payment
					amountInput = "";
					selectedMethod = "cash";
					cardPaymentType = null;
					toast.success(
						`Collected ${formatCurrency(amountCents(), currencySymbol)}. ${formatCurrency(remainingCents(), currencySymbol)} remaining.`
					);
				}
			}
			// If declined or error, result dialog will show the error
		} catch (err) {
			console.error("Transaction error:", err);
			toast.error(err instanceof Error ? err.message : "Transaction failed");
		} finally {
			processing = false;
		}
	}

	// Complete cash/check payment
	async function handleComplete() {
		processing = true;
		try {
			const amount = selectedMethod === "cash" ? amountCents() : remainingCents();
			const change = selectedMethod === "cash" ? changeCents() : 0;

			partialPayments.push({
				method: selectedMethod,
				amountCents: amount,
				changeCents: change
			});

			// Complete the order with all payments
			onComplete(partialPayments);
		} finally {
			processing = false;
		}
	}

	// Close transaction result dialog
	function handleResultClose() {
		transactionResultOpen = false;
		if (transactionResult?.xResult === "A" && remainingCents() === 0) {
			// Order was completed
			return;
		}
		// Reset for retry or different payment method
		transactionResult = null;
	}

	// Keyboard support
	function handleKeydown(e: KeyboardEvent) {
		if (!open) return;
		if (selectedMethod === "cash" && cardPaymentType === null) {
			if (e.key >= "0" && e.key <= "9") {
				appendDigit(e.key);
			} else if (e.key === ".") {
				appendDigit(".");
			} else if (e.key === "Backspace") {
				deleteDigit();
			}
		}
		if (e.key === "Escape") {
			onCancel();
		} else if (e.key === "Enter") {
			if (selectedMethod === "cash" && canCompleteCash()) {
				handleComplete();
			}
		}
	}

	// Reset state when opening
	$effect(() => {
		if (open) {
			selectedMethod = "cash";
			cardPaymentType = null;
			amountInput = "";
			processing = false;
			partialPayments = [];
			selectedDeviceId = "";
			devices = [];
		}
	});

	const methods: { value: PaymentMethod; label: string; icon: typeof Banknote }[] = [
		{ value: "cash", label: "Cash", icon: Banknote },
		{ value: "credit_card", label: "Card", icon: CreditCard },
		{ value: "check", label: "Check", icon: FileCheck }
	];

	const numpadKeys = [
		["1", "2", "3"],
		["4", "5", "6"],
		["7", "8", "9"],
		[".", "0", "back"]
	];
</script>

<svelte:window onkeydown={handleKeydown} />

<Dialog bind:open>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<div class="flex items-center justify-between">
				<DialogTitle>Collect Payment</DialogTitle>
				<div class="text-right">
					<p class="text-lg font-bold text-primary">
						{formatCurrency(remainingCents(), currencySymbol)}
					</p>
					{#if partialPayments.length > 0}
						<p class="text-xs text-muted-foreground">
							{formatCurrency(totalPaidCents(), currencySymbol)} paid
						</p>
					{/if}
				</div>
			</div>
		</DialogHeader>

		<div class="space-y-4">
			<!-- Partial payments list -->
			{#if partialPayments.length > 0}
				<div class="rounded-md border p-3 space-y-2">
					<p class="text-xs font-medium text-muted-foreground">Payments</p>
					{#each partialPayments as payment, i (i)}
						<div class="flex items-center justify-between text-sm">
							<div class="flex items-center gap-2">
								{#if payment.method === "cash"}
									<Banknote class="h-4 w-4" />
									<span>Cash</span>
								{:else if payment.method === "credit_card"}
									<CreditCard class="h-4 w-4" />
									<span>Card {payment.cardDetails?.lastFour ? `****${payment.cardDetails.lastFour}` : ""}</span>
								{:else if payment.method === "check"}
									<FileCheck class="h-4 w-4" />
									<span>Check</span>
								{/if}
							</div>
							<span class="font-medium">{formatCurrency(payment.amountCents, currencySymbol)}</span>
						</div>
					{/each}
					<Separator />
					<div class="flex items-center justify-between text-sm font-medium">
						<span>Remaining</span>
						<span>{formatCurrency(remainingCents(), currencySymbol)}</span>
					</div>
				</div>
			{/if}

			<!-- Payment method selection -->
			<div class="flex gap-2">
				{#each methods as method}
					<Button
						variant={selectedMethod === method.value ? "default" : "outline"}
						class="flex-1 flex-col gap-1 h-auto py-3"
						onclick={() => selectMethod(method.value)}
					>
						<method.icon class="h-5 w-5" />
						<span class="text-xs">{method.label}</span>
					</Button>
				{/each}
			</div>

			<!-- Card payment type selection -->
			{#if selectedMethod === "credit_card"}
				<div class="flex gap-2">
					{#if settingsStore.getBoolean("sola_gateway_card_present")}
						<Button
							variant={cardPaymentType === "card_present" ? "default" : "outline"}
							class="flex-1"
							onclick={() => selectCardType("card_present")}
						>
							Card Present
						</Button>
					{/if}
					{#if settingsStore.getBoolean("sola_gateway_card_not_present")}
						<Button
							variant={cardPaymentType === "card_not_present" ? "default" : "outline"}
							class="flex-1"
							onclick={() => selectCardType("card_not_present")}
						>
							Key In
						</Button>
					{/if}
				</div>
			{/if}

			<!-- Card present flow -->
			{#if selectedMethod === "credit_card" && cardPaymentType === "card_present"}
				<div class="space-y-3">
					<!-- Device selection -->
					<div>
						<Label>Terminal Device</Label>
						<div class="flex gap-2 mt-1.5">
							<div class="flex-1">
								<DeviceSelector
									bind:devices
									bind:selectedDeviceId
									onSelect={(id) => (selectedDeviceId = id)}
									disabled={loadingDevices || processing}
								/>
							</div>
							<Button
								variant="outline"
								size="icon"
								onclick={loadDevices}
								disabled={loadingDevices || processing}
							>
								<RefreshCw class={loadingDevices ? "h-4 w-4 animate-spin" : "h-4 w-4"} />
							</Button>
						</div>
					</div>

					<!-- Amount input -->
					<div>
						<Label>Amount</Label>
						<Input
							type="number"
							step="0.01"
							min="0"
							max={remainingCents() / 100}
							bind:value={amountInput}
							placeholder="0.00"
							disabled={processing}
							class="mt-1.5"
						/>
					</div>

					<!-- Quick amounts -->
					<div class="flex gap-2">
						<Button variant="outline" size="sm" class="flex-1" onclick={setExact} disabled={processing}>
							Exact
						</Button>
						{#each quickAmounts() as amount}
							<Button
								variant="outline"
								size="sm"
								class="flex-1"
								onclick={() => setQuickAmount(amount)}
								disabled={processing}
							>
								{currencySymbol}{amount}
							</Button>
						{/each}
					</div>
				</div>
			{/if}

			<!-- Card not present (key in) -->
			{#if selectedMethod === "credit_card" && cardPaymentType === "card_not_present"}
				<div class="rounded-md border border-dashed p-4 text-center text-sm text-muted-foreground">
					Card not present (key in) coming soon
				</div>
			{/if}

			<!-- Cash payment flow -->
			{#if selectedMethod === "cash"}
				<!-- Amount display -->
				<div
					class="flex h-12 items-center justify-center rounded-md border bg-muted/50 text-xl font-mono"
				>
					{#if amountInput}
						{currencySymbol}{amountInput}
					{:else}
						<span class="text-muted-foreground">{currencySymbol}0.00</span>
					{/if}
				</div>

				<!-- Totals display -->
				<div class="space-y-1 text-sm">
					<div class="flex justify-between">
						<span class="text-muted-foreground">Amount Due</span>
						<span>{formatCurrency(remainingCents(), currencySymbol)}</span>
					</div>
					<div class="flex justify-between">
						<span class="text-muted-foreground">Amount Paid</span>
						<span>{formatCurrency(amountCents(), currencySymbol)}</span>
					</div>
					{#if changeCents() > 0}
						<div class="flex justify-between font-medium text-green-600">
							<span>Change</span>
							<span>{formatCurrency(changeCents(), currencySymbol)}</span>
						</div>
					{:else if amountInput && amountCents() < remainingCents()}
						<div class="flex justify-between font-medium text-destructive">
							<span>Remaining</span>
							<span>{formatCurrency(remainingCents() - amountCents(), currencySymbol)}</span>
						</div>
					{/if}
				</div>

				<!-- Quick amounts -->
				<div class="flex gap-2">
					<Button variant="outline" size="sm" class="flex-1" onclick={setExact}>
						Exact
					</Button>
					{#each quickAmounts() as amount}
						<Button variant="outline" size="sm" class="flex-1" onclick={() => setQuickAmount(amount)}>
							{currencySymbol}{amount}
						</Button>
					{/each}
				</div>

				<!-- Numpad -->
				<div class="grid grid-cols-3 gap-2">
					{#each numpadKeys as row}
						{#each row as key}
							{#if key === "back"}
								<Button variant="outline" class="h-12" onclick={deleteDigit}>
									<Delete class="h-5 w-5" />
								</Button>
							{:else if key === "."}
								<Button variant="outline" class="h-12 text-lg" onclick={() => appendDigit(".")}>
									.
								</Button>
							{:else}
								<Button
									variant="outline"
									class="h-12 text-lg font-medium"
									onclick={() => appendDigit(key)}
								>
									{key}
								</Button>
							{/if}
						{/each}
					{/each}
				</div>
			{/if}

			<!-- Check payment -->
			{#if selectedMethod === "check"}
				<!-- Just show the total -->
				<div class="space-y-2 rounded-md border p-4 text-center">
					<p class="text-sm text-muted-foreground">Check payment for</p>
					<p class="text-2xl font-bold">{formatCurrency(remainingCents(), currencySymbol)}</p>
				</div>
			{/if}

			<Separator />

			<!-- Action buttons -->
			<div class="flex gap-3">
				<Button variant="outline" class="flex-1" onclick={onCancel}>
					Cancel
				</Button>

				{#if selectedMethod === "credit_card" && cardPaymentType === "card_present"}
					<Button class="flex-1" disabled={!canSendToDevice()} onclick={handleSendToDevice}>
						{#if processing}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						{/if}
						Send to Device
					</Button>
				{:else if selectedMethod === "credit_card" && cardPaymentType === "card_not_present"}
					<Button class="flex-1" disabled>
						Coming Soon
					</Button>
				{:else}
					<Button
						class="flex-1"
						disabled={(selectedMethod === "cash" && !canCompleteCash()) || processing}
						onclick={handleComplete}
					>
						{#if processing}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						{/if}
						Complete Payment
					</Button>
				{/if}
			</div>
		</div>
	</DialogContent>
</Dialog>

<!-- Transaction result dialog -->
<TransactionResultDialog
	bind:open={transactionResultOpen}
	result={transactionResult}
	amountCents={amountCents()}
	{currencySymbol}
	onClose={handleResultClose}
/>
