<script lang="ts">
	import { Banknote, CreditCard, FileCheck, Delete, Loader2, RefreshCw, Lock, AlertCircle } from "lucide-svelte";
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
	import {
		processSolaTransaction,
		cancelSolaTransaction,
		buildSolaRequestInfo,
		processSolaCnpTransaction,
		processSolaTokenTransaction
	} from "$lib/commands/sola.js";
	import { getCustomerTokens, createPaymentToken } from "$lib/commands/payment-tokens.js";
	import { loadIfields, isIfieldsLoaded, IFIELDS_IFRAME_URL } from "$lib/utils/ifields-loader.js";
	import DeviceSelector from "./DeviceSelector.svelte";
	import TransactionResultDialog from "./TransactionResultDialog.svelte";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import type {
		PaymentMethod,
		PartialPayment,
		SolaDevice,
		SolaDeviceListResponse,
		SolaTransactionResponse,
		SolaRequestInfo,
		SolaTransactionResult,
		CustomerPaymentToken
	} from "$lib/types/index.js";

	interface Props {
		open: boolean;
		totalCents: number;
		currencySymbol: string;
		customerId?: number | null;
		onComplete: (payments: PartialPayment[]) => void;
		onCancel: () => void;
	}

	let { open = $bindable(), totalCents, currencySymbol, customerId = null, onComplete, onCancel }: Props = $props();

	// Generate a temporary invoice reference for Sola gateway transactions.
	// This is NOT the order number — the real order number is assigned when the order is created.
	let invoiceNumber = $state("");

	function generateInvoiceRef(): string {
		return `TXN-${Date.now()}`;
	}

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

	// Card not present (iFields) state
	let cnpLoading = $state(false);
	let cnpLoadError = $state<string | null>(null);
	let cnpInitialized = $state(false);
	let cnpCardNumberValid = $state(false);
	let cnpCvvValid = $state(false);
	let cnpCardBrand = $state("");
	let cnpExpiry = $state("");
	let cnpName = $state("");
	let cnpZip = $state("");
	let cnpProcessing = $state(false);

	const cnpExpiryFormatted = $derived.by(() => {
		// Convert MM/YY or MMYY to MMYY for gateway
		return cnpExpiry.replace(/\//g, "");
	});

	const cnpExpiryValid = $derived.by(() => {
		const digits = cnpExpiry.replace(/\D/g, "");
		if (digits.length !== 4) return false;
		const month = parseInt(digits.substring(0, 2), 10);
		return month >= 1 && month <= 12;
	});

	const canProcessCnp = $derived.by(() => {
		return (
			cnpCardNumberValid &&
			cnpCvvValid &&
			cnpExpiryValid &&
			remainingCents > 0 &&
			!cnpProcessing &&
			!processing
		);
	});

	// Card on file state
	let savedCards = $state<CustomerPaymentToken[]>([]);
	let selectedSavedCard = $state<CustomerPaymentToken | null>(null);
	let saveCardChecked = $state(false);
	let tokenProcessing = $state(false);

	const cardOnFileEnabled = $derived.by(() => {
		return settingsStore.getBoolean("enable_card_on_file") && !!customerId;
	});

	const canPayWithToken = $derived.by(() => {
		return (
			selectedSavedCard !== null &&
			remainingCents > 0 &&
			!tokenProcessing &&
			!processing
		);
	});

	// Transaction result state
	let transactionResultOpen = $state(false);
	let txRequestInfo = $state<SolaRequestInfo | null>(null);
	let txResult = $state<SolaTransactionResult | null>(null);
	let txError = $state<string | null>(null);
	let txPending = $state(false);
	// Cancel state
	let txCancelling = $state(false);
	let txCancelRequestInfo = $state<SolaRequestInfo | null>(null);
	let txCancelResult = $state<SolaTransactionResult | null>(null);
	let txCancelError = $state<string | null>(null);
	// Keep a ref to the decrypted key for cancel
	let activeDecryptedKey = $state("");

	// Partial payments tracking
	let partialPayments = $state<PartialPayment[]>([]);

	// Derived values
	const amountCents = $derived.by(() => {
		if (!amountInput) return 0;
		const val = parseFloat(amountInput);
		return isNaN(val) ? 0 : Math.round(val * 100);
	});

	const changeCents = $derived.by(() => {
		const paid = amountCents;
		return Math.max(0, paid - remainingCents);
	});

	const totalPaidCents = $derived.by(() => {
		return partialPayments.reduce((sum, p) => sum + p.amountCents, 0);
	});

	const remainingCents = $derived.by(() => {
		return totalCents - totalPaidCents;
	});

	const canCompleteCash = $derived.by(() => {
		return amountCents >= remainingCents;
	});

	const canSendToDevice = $derived.by(() => {
		return (
			cardPaymentType === "card_present" &&
			selectedDeviceId &&
			amountCents > 0 &&
			amountCents <= remainingCents &&
			!processing
		);
	});

	// Quick amount suggestions based on remaining balance
	const quickAmounts = $derived.by(() => {
		const remainingDollars = remainingCents / 100;
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
		amountInput = (remainingCents / 100).toFixed(2);
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
				const cnpEnabled = settingsStore.getBoolean("sola_gateway_card_not_present") && !!settingsStore.get("ifields_key");
				if (cnpEnabled) {
					cardPaymentType = "card_not_present";
					setExact();
					initIfields();
				}
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

		if (type === "card_not_present") {
			initIfields();
		}
	}

	// Initialize iFields for CNP
	async function initIfields() {
		if (cnpInitialized || cnpLoading) return;
		cnpLoading = true;
		cnpLoadError = null;

		try {
			await loadIfields();

			// Get iFields key
			const encryptedIfieldsKey = settingsStore.get("ifields_key");
			if (!encryptedIfieldsKey) {
				cnpLoadError = "iFields key not configured. Go to Settings > Payments to add it.";
				return;
			}

			const ifieldsKey = await decryptValue(encryptedIfieldsKey);
			if (!ifieldsKey) {
				cnpLoadError = "Failed to decrypt iFields key.";
				return;
			}

			// Initialize iFields account
			setAccount(ifieldsKey, "Vira", "1.0.0");

			// Apply styles to match our UI
			const isDark = document.documentElement.classList.contains("dark");
			const style = isDark
				? "border: 1px solid hsl(240 3.7% 25%); border-radius: 0.375rem; padding: 0.5rem 0.75rem; font-size: 0.875rem; background: hsl(240 10% 3.9%); color: hsl(0 0% 98%); width: 100%; height: 36px; box-sizing: border-box;"
				: "border: 1px solid hsl(240 5.9% 80%); border-radius: 0.375rem; padding: 0.5rem 0.75rem; font-size: 0.875rem; background: hsl(0 0% 100%); color: hsl(240 10% 3.9%); width: 100%; height: 36px; box-sizing: border-box;";

			setIfieldStyle("card-number", style);
			setIfieldStyle("cvv", style);

			// Enable auto-formatting
			enableAutoFormatting("-");

			// Set up keypress callback for card brand detection and validation
			addIfieldKeyPressCallback((data: IfieldsKeyPressData) => {
				cnpCardNumberValid = data.cardNumberIsValid;
				cnpCvvValid = data.cvvIsValid;
				if (data.issuer) {
					cnpCardBrand = data.issuer;
				}
			});

			cnpInitialized = true;
		} catch (err) {
			console.error("Failed to initialize iFields:", err);
			cnpLoadError =
				"Card entry is unavailable. Please check your internet connection or use another payment method.";
		} finally {
			cnpLoading = false;
		}
	}

	// Handle expiry input formatting (MM/YY)
	function handleExpiryInput(e: Event) {
		const input = e.currentTarget as HTMLInputElement;
		let val = input.value.replace(/\D/g, "");
		if (val.length > 4) val = val.substring(0, 4);
		if (val.length >= 3) {
			val = val.substring(0, 2) + "/" + val.substring(2);
		}
		cnpExpiry = val;
		input.value = val;
	}

	// Process CNP payment
	async function handleCnpPayment() {
		cnpProcessing = true;
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

			// Step 1: Get tokens from iFields
			const tokens = await new Promise<void>((resolve, reject) => {
				getTokens(
					() => resolve(),
					(error: IfieldsError) => reject(new Error(error.errorMessage || "Failed to get card tokens")),
					30000
				);
			});

			// Step 2: Read token values from hidden inputs
			const cardTokenEl = document.querySelector('[data-ifields-id="card-number-token"]') as HTMLInputElement;
			const cvvTokenEl = document.querySelector('[data-ifields-id="cvv-token"]') as HTMLInputElement;

			if (!cardTokenEl?.value || !cvvTokenEl?.value) {
				throw new Error("Card tokens not available. Please try again.");
			}

			// Step 3: Send to gateway via Rust command
			const result = await processSolaCnpTransaction({
				apiKey: decryptedKey,
				cardToken: cardTokenEl.value,
				cvvToken: cvvTokenEl.value,
				exp: cnpExpiryFormatted,
				amountCents: remainingCents,
				invoiceNumber,
				name: cnpName || undefined,
				zip: cnpZip || undefined
			});

			const response = result.response;

			if (response.xResult === "A") {
				// Approved
				const lastFour = response.xMaskedCardNumber?.slice(-4) || "";

				partialPayments.push({
					method: "credit_card",
					amountCents: remainingCents,
					changeCents: 0,
					cardDetails: {
						authCode: response.xAuthCode || "",
						lastFour,
						cardType: response.xCardType || "",
						entryMode: "keyed",
						gatewayRefNum: response.xRefnum,
						gatewayResponse: JSON.stringify(response)
					}
				});

				toast.success("Payment approved");

				// Save card token if checkbox was checked
				await maybeSaveCardToken(response);

				if (remainingCents === 0) {
					onComplete(partialPayments);
				} else {
					amountInput = "";
					selectedMethod = "cash";
					cardPaymentType = null;
					toast.success(
						`Collected ${formatCurrency(remainingCents, currencySymbol)}. ${formatCurrency(remainingCents, currencySymbol)} remaining.`
					);
				}
			} else if (response.xResult === "D") {
				toast.error(`Payment declined: ${response.xError || "Unknown reason"}`);
			} else {
				toast.error(`Payment error: ${response.xError || "Unknown error"}`);
			}
		} catch (err) {
			console.error("CNP transaction error:", err);
			const message = err instanceof Error ? err.message : String(err);
			if (message.includes("timed out") || message.includes("timeout")) {
				toast.error("Card processing timed out. Please try again.");
			} else {
				toast.error(message);
			}
		} finally {
			cnpProcessing = false;
			processing = false;
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

		// Reset transaction state
		txRequestInfo = null;
		txResult = null;
		txError = null;
		txPending = true;
		txCancelling = false;
		txCancelRequestInfo = null;
		txCancelResult = null;
		txCancelError = null;

		try {
			const apiKey = settingsStore.get("sola_gateway_api_key");
			if (!apiKey) {
				toast.error("Sola Gateway API key not configured");
				processing = false;
				return;
			}

			const decryptedKey = await decryptValue(apiKey);
			if (!decryptedKey) {
				toast.error("Failed to decrypt API key");
				processing = false;
				return;
			}

			activeDecryptedKey = decryptedKey;

			// Build request info and show dialog IMMEDIATELY
			const reqInfo = await buildSolaRequestInfo({
				apiKey: decryptedKey,
				deviceId: selectedDeviceId,
				amountCents: amountCents,
				invoiceNumber,
				command: "cc:sale"
			});
			txRequestInfo = reqInfo;
			transactionResultOpen = true;

			// Now fire the actual transaction (this waits for the device)
			const txResultData = await processSolaTransaction({
				apiKey: decryptedKey,
				deviceId: selectedDeviceId,
				amountCents: amountCents,
				invoiceNumber
			});

			txPending = false;
			txResult = txResultData;

			const response = txResultData.response;

			if (response.xResult === "A") {
				// Approved - add to partial payments
				const lastFour = response.xMaskedCardNumber?.slice(-4) || "";

				partialPayments.push({
					method: "credit_card",
					amountCents: amountCents,
					changeCents: 0,
					cardDetails: {
						authCode: response.xAuthCode || "",
						lastFour,
						cardType: response.xCardType || "",
						entryMode: "card_present",
						gatewayRefNum: response.xRefnum,
						gatewayResponse: JSON.stringify(response)
					}
				});

				// Save card token if checkbox was checked
				await maybeSaveCardToken(response);

				// Check if fully paid
				if (remainingCents === 0) {
					// Close result dialog and complete order
					transactionResultOpen = false;
					onComplete(partialPayments);
				} else {
					// Reset for next payment
					amountInput = "";
					selectedMethod = "cash";
					cardPaymentType = null;
					toast.success(
						`Collected ${formatCurrency(amountCents, currencySymbol)}. ${formatCurrency(remainingCents, currencySymbol)} remaining.`
					);
				}
			}
			// If declined or error, result dialog will show the error
		} catch (err) {
			console.error("Transaction error:", err);
			txPending = false;
			txError = err instanceof Error ? err.message : String(err);
		} finally {
			processing = false;
		}
	}

	// Cancel a pending transaction on the device
	async function handleCancelTransaction() {
		txCancelling = true;
		txCancelRequestInfo = null;
		txCancelResult = null;
		txCancelError = null;

		try {
			// Build cancel request info and show it immediately
			const cancelReqInfo = await buildSolaRequestInfo({
				apiKey: activeDecryptedKey,
				deviceId: selectedDeviceId,
				amountCents: 0,
				command: "cc:cancel"
			});
			txCancelRequestInfo = cancelReqInfo;

			// Fire the cancel
			const cancelResultData = await cancelSolaTransaction({
				apiKey: activeDecryptedKey,
				deviceId: selectedDeviceId
			});
			txCancelResult = cancelResultData;
		} catch (err) {
			console.error("Cancel error:", err);
			txCancelError = err instanceof Error ? err.message : String(err);
		} finally {
			txCancelling = false;
			txPending = false;
			processing = false;
		}
	}

	// Complete cash/check payment
	async function handleComplete() {
		processing = true;
		try {
			const amount = selectedMethod === "cash" ? amountCents : remainingCents;
			const change = selectedMethod === "cash" ? changeCents : 0;

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

	// Pay with saved card token
	async function handlePayWithToken() {
		if (!selectedSavedCard || !customerId) return;
		tokenProcessing = true;

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

			// Decrypt the saved token
			const decryptedToken = await decryptValue(selectedSavedCard.token);
			if (!decryptedToken) {
				toast.error("Failed to decrypt saved card token");
				return;
			}

			const result = await processSolaTokenTransaction({
				apiKey: decryptedKey,
				token: decryptedToken,
				amountCents: remainingCents,
				invoiceNumber
			});

			const response = result.response;

			if (response.xResult === "A") {
				// Approved
				const lastFour = selectedSavedCard.card_last_four;

				partialPayments.push({
					method: "credit_card",
					amountCents: remainingCents,
					changeCents: 0,
					cardDetails: {
						authCode: response.xAuthCode || "",
						lastFour,
						cardType: selectedSavedCard.card_brand || response.xCardType || "",
						entryMode: "token",
						gatewayRefNum: response.xRefnum,
						gatewayResponse: JSON.stringify(response)
					}
				});

				if (remainingCents === 0) {
					onComplete(partialPayments);
				} else {
					amountInput = "";
					selectedMethod = "cash";
					cardPaymentType = null;
					selectedSavedCard = null;
					toast.success(
						`Collected ${formatCurrency(remainingCents, currencySymbol)} via saved card. ${formatCurrency(remainingCents, currencySymbol)} remaining.`
					);
				}
			} else {
				toast.error(response.xError || "Transaction declined");
			}
		} catch (err) {
			console.error("Token transaction error:", err);
			toast.error(err instanceof Error ? err.message : "Transaction failed");
		} finally {
			tokenProcessing = false;
		}
	}

	// Save card token after successful transaction (if checkbox checked)
	async function maybeSaveCardToken(response: SolaTransactionResponse) {
		if (!saveCardChecked || !customerId || !response.xToken) return;
		try {
			const maskedNum = response.xMaskedCardNumber || "";
			const lastFour = maskedNum.slice(-4);
			const expStr = response.xExp || "";
			let expMonth: number | null = null;
			let expYear: number | null = null;
			if (expStr.length === 4) {
				expMonth = parseInt(expStr.substring(0, 2), 10);
				expYear = 2000 + parseInt(expStr.substring(2, 4), 10);
			}

			await createPaymentToken({
				customerId,
				token: response.xToken,
				cardLastFour: lastFour,
				cardBrand: response.xCardType || null,
				expMonth,
				expYear,
				isDefault: savedCards.length === 0 // first card becomes default
			});
			toast.success("Card saved for future use");
		} catch (err) {
			// FR-8: don't fail the transaction
			console.error("Failed to save card token:", err);
		}
	}

	// Close transaction result dialog
	function handleResultClose() {
		transactionResultOpen = false;
		if (txResult?.response?.xResult === "A" && remainingCents === 0) {
			// Order was completed
			return;
		}
		// Reset for retry or different payment method
		txRequestInfo = null;
		txResult = null;
		txError = null;
		txCancelRequestInfo = null;
		txCancelResult = null;
		txCancelError = null;
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
			if (selectedMethod === "cash" && canCompleteCash) {
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
			invoiceNumber = generateInvoiceRef();
			// Reset transaction debug state
			txRequestInfo = null;
			txResult = null;
			txError = null;
			txPending = false;
			txCancelling = false;
			txCancelRequestInfo = null;
			txCancelResult = null;
			txCancelError = null;
			activeDecryptedKey = "";
			// Reset CNP state
			cnpLoadError = null;
			cnpInitialized = false;
			cnpCardNumberValid = false;
			cnpCvvValid = false;
			cnpCardBrand = "";
			cnpExpiry = "";
			cnpName = "";
			cnpZip = "";
			cnpProcessing = false;
			// Reset card-on-file state
			savedCards = [];
			selectedSavedCard = null;
			saveCardChecked = false;
			tokenProcessing = false;
			// Load saved cards if customer is attached and card-on-file enabled
			if (customerId && settingsStore.getBoolean("enable_card_on_file")) {
				getCustomerTokens(customerId).then((tokens) => {
					savedCards = tokens;
				}).catch(() => {
					// Non-fatal — just no saved cards shown
				});
			}
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
						{formatCurrency(remainingCents, currencySymbol)}
					</p>
					{#if partialPayments.length > 0}
						<p class="text-xs text-muted-foreground">
							{formatCurrency(totalPaidCents, currencySymbol)} paid
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
						<span>{formatCurrency(remainingCents, currencySymbol)}</span>
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
					{#if settingsStore.getBoolean("sola_gateway_card_not_present") && settingsStore.get("ifields_key")}
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

			<!-- Saved cards section -->
			{#if selectedMethod === "credit_card" && savedCards.length > 0 && cardOnFileEnabled}
				<div class="space-y-2">
					<Label class="text-xs text-muted-foreground">Saved Cards</Label>
					{#each savedCards as card (card.id)}
						<Button
							variant={selectedSavedCard?.id === card.id ? "default" : "outline"}
							class="w-full justify-start gap-3"
							onclick={() => {
								selectedSavedCard = selectedSavedCard?.id === card.id ? null : card;
								if (selectedSavedCard) {
									cardPaymentType = null;
								}
							}}
						>
							<CreditCard class="h-4 w-4" />
							<span class="text-sm">
								{card.card_brand || "Card"} **** {card.card_last_four}
							</span>
							{#if card.exp_month && card.exp_year}
								<span class="text-xs text-muted-foreground">
									{String(card.exp_month).padStart(2, "0")}/{String(card.exp_year).slice(-2)}
								</span>
							{/if}
							{#if card.is_default}
								<Badge variant="secondary" class="ml-auto text-xs">Default</Badge>
							{/if}
						</Button>
					{/each}
					{#if selectedSavedCard}
						<Separator />
					{:else}
						<div class="relative">
							<Separator />
							<span class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 bg-background px-2 text-xs text-muted-foreground">
								or pay with new card
							</span>
						</div>
					{/if}
				</div>
			{/if}

			<!-- Card present flow -->
			{#if selectedMethod === "credit_card" && cardPaymentType === "card_present" && !selectedSavedCard}
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
							max={remainingCents / 100}
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
						{#each quickAmounts as amount}
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
			{#if selectedMethod === "credit_card" && cardPaymentType === "card_not_present" && !selectedSavedCard}
				{#if cnpLoading}
					<div class="flex items-center justify-center gap-2 rounded-md border p-6 text-sm text-muted-foreground">
						<Loader2 class="h-4 w-4 animate-spin" />
						Loading secure card entry...
					</div>
				{:else if cnpLoadError}
					<div class="rounded-md border border-destructive/50 bg-destructive/10 p-4 space-y-2">
						<div class="flex items-center gap-2 text-sm text-destructive">
							<AlertCircle class="h-4 w-4" />
							{cnpLoadError}
						</div>
						<Button variant="outline" size="sm" onclick={() => { cnpInitialized = false; initIfields(); }}>
							Retry
						</Button>
					</div>
				{:else if cnpInitialized}
					<div class="space-y-3">
						<!-- Secure entry label -->
						<div class="flex items-center gap-2 text-xs text-muted-foreground">
							<Lock class="h-3 w-3" />
							<span>Secure Entry</span>
						</div>

						<!-- Amount display -->
						<div class="flex items-center justify-between rounded-md border bg-muted/50 px-3 py-2">
							<span class="text-sm text-muted-foreground">Amount</span>
							<span class="text-lg font-bold">{formatCurrency(remainingCents, currencySymbol)}</span>
						</div>

						<!-- Card Number -->
						<div class="space-y-1.5">
							<div class="flex items-center justify-between">
								<Label class="text-sm">Card Number</Label>
								{#if cnpCardBrand}
									<span class="text-xs font-medium text-muted-foreground uppercase">{cnpCardBrand}</span>
								{/if}
							</div>
							<iframe
								data-ifields-id="card-number"
								data-ifields-placeholder="Card number"
								src={IFIELDS_IFRAME_URL}
								title="Card Number"
								class="h-9 w-full rounded-md"
								style="border: none;"
							></iframe>
							<input type="hidden" data-ifields-id="card-number-token" />
						</div>

						<!-- Expiry + CVV row -->
						<div class="grid grid-cols-2 gap-3">
							<div class="space-y-1.5">
								<Label class="text-sm">Expiration</Label>
								<Input
									type="text"
									placeholder="MM/YY"
									value={cnpExpiry}
									oninput={handleExpiryInput}
									maxlength={5}
									disabled={cnpProcessing}
								/>
							</div>
							<div class="space-y-1.5">
								<Label class="text-sm">CVV</Label>
								<iframe
									data-ifields-id="cvv"
									data-ifields-placeholder="CVV"
									src={IFIELDS_IFRAME_URL}
									title="CVV"
									class="h-9 w-full rounded-md"
									style="border: none;"
								></iframe>
								<input type="hidden" data-ifields-id="cvv-token" />
							</div>
						</div>

						<!-- Name -->
						<div class="space-y-1.5">
							<Label class="text-sm">Cardholder Name <span class="text-muted-foreground">(optional)</span></Label>
							<Input
								type="text"
								placeholder="Name on card"
								bind:value={cnpName}
								disabled={cnpProcessing}
							/>
						</div>

						<!-- ZIP -->
						<div class="space-y-1.5">
							<Label class="text-sm">Billing ZIP <span class="text-muted-foreground">(optional)</span></Label>
							<Input
								type="text"
								placeholder="ZIP code"
								bind:value={cnpZip}
								maxlength={10}
								disabled={cnpProcessing}
							/>
						</div>
					</div>
				{:else}
					<div class="rounded-md border border-dashed p-4 text-center text-sm text-muted-foreground">
						Initializing secure card entry...
					</div>
				{/if}
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
						<span>{formatCurrency(remainingCents, currencySymbol)}</span>
					</div>
					<div class="flex justify-between">
						<span class="text-muted-foreground">Amount Paid</span>
						<span>{formatCurrency(amountCents, currencySymbol)}</span>
					</div>
					{#if changeCents > 0}
						<div class="flex justify-between font-medium text-green-600">
							<span>Change</span>
							<span>{formatCurrency(changeCents, currencySymbol)}</span>
						</div>
					{:else if amountInput && amountCents < remainingCents}
						<div class="flex justify-between font-medium text-destructive">
							<span>Remaining</span>
							<span>{formatCurrency(remainingCents - amountCents, currencySymbol)}</span>
						</div>
					{/if}
				</div>

				<!-- Quick amounts -->
				<div class="flex gap-2">
					<Button variant="outline" size="sm" class="flex-1" onclick={setExact}>
						Exact
					</Button>
					{#each quickAmounts as amount}
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
					<p class="text-2xl font-bold">{formatCurrency(remainingCents, currencySymbol)}</p>
				</div>
			{/if}

			<!-- Save card checkbox -->
			{#if selectedMethod === "credit_card" && !selectedSavedCard && cardOnFileEnabled && (cardPaymentType === "card_present" || cardPaymentType === "card_not_present")}
				<label class="flex items-center gap-2 text-sm">
					<input type="checkbox" bind:checked={saveCardChecked} class="rounded" />
					Save card for future use
				</label>
			{/if}

			<Separator />

			<!-- Action buttons -->
			<div class="flex gap-3">
				<Button variant="outline" class="flex-1" onclick={onCancel}>
					Cancel
				</Button>

				{#if selectedMethod === "credit_card" && selectedSavedCard}
					<Button class="flex-1" disabled={!canPayWithToken} onclick={handlePayWithToken}>
						{#if tokenProcessing}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							Processing...
						{:else}
							Pay with Saved Card
						{/if}
					</Button>
				{:else if selectedMethod === "credit_card" && cardPaymentType === "card_present"}
					<Button class="flex-1" disabled={!canSendToDevice} onclick={handleSendToDevice}>
						{#if processing}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						{/if}
						Send to Device
					</Button>
				{:else if selectedMethod === "credit_card" && cardPaymentType === "card_not_present"}
					<Button class="flex-1" disabled={!canProcessCnp} onclick={handleCnpPayment}>
						{#if cnpProcessing}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							Processing...
						{:else}
							Process Payment
						{/if}
					</Button>
				{:else}
					<Button
						class="flex-1"
						disabled={(selectedMethod === "cash" && !canCompleteCash) || processing}
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
	requestInfo={txRequestInfo}
	result={txResult}
	error={txError}
	pending={txPending}
	amountCents={amountCents}
	{currencySymbol}
	onClose={handleResultClose}
	onCancel={handleCancelTransaction}
	cancelling={txCancelling}
	cancelRequestInfo={txCancelRequestInfo}
	cancelResult={txCancelResult}
	cancelError={txCancelError}
/>
