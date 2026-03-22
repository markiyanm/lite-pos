<script lang="ts">
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle,
		DialogFooter
	} from "$lib/components/ui/dialog/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Printer, Loader2, X } from "lucide-svelte";
	import ReceiptTemplate from "./ReceiptTemplate.svelte";
	import { printReceipt, type ReceiptData } from "$lib/commands/printing.js";
	import { toast } from "svelte-sonner";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import type { Order, OrderItem, Payment, Customer } from "$lib/types/index.js";

	interface Props {
		open: boolean;
		/** Pre-built receipt data for ESC/POS printing */
		receiptData: ReceiptData | null;
		/** Visual preview props for ReceiptTemplate */
		order: Order | null;
		items: OrderItem[];
		payments: Payment[];
		customer: Customer | null;
		storeName: string;
		storeAddress: string;
		storePhone: string;
		taxLabel: string;
		currencySymbol: string;
		printerType: "standard" | "thermal";
		printerName: string;
		onClose: () => void;
		onPrinted: () => void;
	}

	let {
		open = $bindable(),
		receiptData,
		order,
		items,
		payments,
		customer,
		storeName,
		storeAddress,
		storePhone,
		taxLabel,
		currencySymbol,
		printerType,
		printerName,
		onClose,
		onPrinted
	}: Props = $props();

	let printing = $state(false);
	const receiptHeader = $derived(settingsStore.get("receipt_header") || "");
	const receiptFooter = $derived(settingsStore.get("receipt_footer") || "");

	async function handlePrint() {
		if (!receiptData || !printerName) return;

		printing = true;
		try {
			await printReceipt(receiptData, printerName);
			toast.success("Receipt sent to printer");
			onPrinted();
		} catch (err: any) {
			console.error("Print error:", err);
			toast.error(`Print failed: ${err?.message || err}`);
		} finally {
			printing = false;
		}
	}

	function handleSkip() {
		onClose();
	}
</script>

<Dialog bind:open>
	<DialogContent class="max-w-2xl max-h-[90vh] overflow-y-auto">
		<DialogHeader>
			<DialogTitle class="flex items-center gap-2">
				<Printer class="h-5 w-5" />
				Print Receipt
			</DialogTitle>
		</DialogHeader>

		{#if order}
			<div class="receipt-preview">
				<ReceiptTemplate
					{order}
					{items}
					{payments}
					{customer}
					{storeName}
					{storeAddress}
					{storePhone}
					{receiptHeader}
					{receiptFooter}
					{taxLabel}
					{currencySymbol}
					{printerType}
				/>
			</div>

			<DialogFooter class="gap-2 sm:gap-0">
				<Button variant="outline" onclick={handleSkip}>
					<X class="h-4 w-4 mr-2" />
					Skip
				</Button>
				<Button onclick={handlePrint} disabled={printing || !printerName}>
					{#if printing}
						<Loader2 class="h-4 w-4 mr-2 animate-spin" />
						Printing...
					{:else}
						<Printer class="h-4 w-4 mr-2" />
						Print
					{/if}
				</Button>
			</DialogFooter>
		{/if}
	</DialogContent>
</Dialog>

<style>
	.receipt-preview {
		border: 1px solid hsl(var(--border));
		border-radius: 8px;
		overflow: hidden;
		background: white;
	}
</style>
