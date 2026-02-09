<script lang="ts">
	import { Banknote, CreditCard, FileCheck, Delete, Loader2 } from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle
	} from "$lib/components/ui/dialog/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import { formatCurrency } from "$lib/utils.js";
	import type { PaymentMethod } from "$lib/types/index.js";

	interface Props {
		open: boolean;
		totalCents: number;
		currencySymbol: string;
		onComplete: (method: PaymentMethod, amountCents: number) => void;
		onCancel: () => void;
	}

	let { open = $bindable(), totalCents, currencySymbol, onComplete, onCancel }: Props = $props();

	let selectedMethod = $state<PaymentMethod>("cash");
	let amountInput = $state("");
	let processing = $state(false);

	const amountCents = $derived(() => {
		if (!amountInput) return 0;
		const val = parseFloat(amountInput);
		return isNaN(val) ? 0 : Math.round(val * 100);
	});

	const changeCents = $derived(() => {
		const paid = amountCents();
		return Math.max(0, paid - totalCents);
	});

	const canComplete = $derived(() => {
		if (selectedMethod === "cash") {
			return amountCents() >= totalCents;
		}
		// Card/check: exact amount
		return true;
	});

	// Quick amount suggestions based on total
	const quickAmounts = $derived(() => {
		const totalDollars = totalCents / 100;
		const suggestions: number[] = [];

		// Round up to nearest 5, 10, 20, 50
		const roundups = [5, 10, 20, 50, 100];
		for (const r of roundups) {
			const rounded = Math.ceil(totalDollars / r) * r;
			if (rounded >= totalDollars && !suggestions.includes(rounded)) {
				suggestions.push(rounded);
			}
			if (suggestions.length >= 4) break;
		}

		return suggestions;
	});

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

	function clearAmount() {
		amountInput = "";
	}

	function setExact() {
		amountInput = (totalCents / 100).toFixed(2);
	}

	function setQuickAmount(dollars: number) {
		amountInput = dollars.toFixed(2);
	}

	function selectMethod(method: PaymentMethod) {
		selectedMethod = method;
		if (method !== "cash") {
			// Auto-set exact amount for non-cash
			setExact();
		} else {
			amountInput = "";
		}
	}

	async function handleComplete() {
		processing = true;
		try {
			const amount = selectedMethod === "cash" ? amountCents() : totalCents;
			onComplete(selectedMethod, amount);
		} finally {
			processing = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!open) return;
		if (e.key >= "0" && e.key <= "9") {
			appendDigit(e.key);
		} else if (e.key === ".") {
			appendDigit(".");
		} else if (e.key === "Backspace") {
			deleteDigit();
		} else if (e.key === "Escape") {
			onCancel();
		} else if (e.key === "Enter" && canComplete()) {
			handleComplete();
		}
	}

	// Reset state when opening
	$effect(() => {
		if (open) {
			selectedMethod = "cash";
			amountInput = "";
			processing = false;
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
						{formatCurrency(totalCents, currencySymbol)}
					</p>
				</div>
			</div>
		</DialogHeader>

		<div class="space-y-4">
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

			{#if selectedMethod === "cash"}
				<!-- Amount display -->
				<div class="flex h-12 items-center justify-center rounded-md border bg-muted/50 text-xl font-mono">
					{#if amountInput}
						{currencySymbol}{amountInput}
					{:else}
						<span class="text-muted-foreground">{currencySymbol}0.00</span>
					{/if}
				</div>

				<!-- Totals display -->
				<div class="space-y-1 text-sm">
					<div class="flex justify-between">
						<span class="text-muted-foreground">Total Bill</span>
						<span>{formatCurrency(totalCents, currencySymbol)}</span>
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
					{:else if amountInput && amountCents() < totalCents}
						<div class="flex justify-between font-medium text-destructive">
							<span>Remaining</span>
							<span>{formatCurrency(totalCents - amountCents(), currencySymbol)}</span>
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
								<Button variant="outline" class="h-12 text-lg font-medium" onclick={() => appendDigit(key)}>
									{key}
								</Button>
							{/if}
						{/each}
					{/each}
				</div>
			{:else}
				<!-- Non-cash: just show the total -->
				<div class="space-y-2 rounded-md border p-4 text-center">
					<p class="text-sm text-muted-foreground">
						{selectedMethod === "credit_card" ? "Card" : "Check"} payment for
					</p>
					<p class="text-2xl font-bold">{formatCurrency(totalCents, currencySymbol)}</p>
				</div>
			{/if}

			<Separator />

			<!-- Action buttons -->
			<div class="flex gap-3">
				<Button variant="outline" class="flex-1" onclick={onCancel}>
					Cancel
				</Button>
				<Button
					class="flex-1"
					disabled={!canComplete() || processing}
					onclick={handleComplete}
				>
					{#if processing}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
					{/if}
					Complete Payment
				</Button>
			</div>
		</div>
	</DialogContent>
</Dialog>
