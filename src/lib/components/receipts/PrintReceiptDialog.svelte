<script lang="ts">
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle,
		DialogFooter
	} from "$lib/components/ui/dialog/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Printer } from "lucide-svelte";
	import ReceiptTemplate from "./ReceiptTemplate.svelte";
	import type { Order, OrderItem, Payment, Customer } from "$lib/types/index.js";

	interface Props {
		open: boolean;
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
		onClose: () => void;
	}

	let {
		open = $bindable(),
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
		onClose
	}: Props = $props();

	function handlePrint() {
		window.print();
	}
</script>

<Dialog bind:open>
	<DialogContent class="max-w-2xl max-h-[90vh] overflow-y-auto no-print">
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
					{taxLabel}
					{currencySymbol}
					{printerType}
				/>
			</div>

			<DialogFooter>
				<Button variant="outline" onclick={onClose}>Cancel</Button>
				<Button onclick={handlePrint}>
					<Printer class="h-4 w-4 mr-2" />
					Print
				</Button>
			</DialogFooter>
		{/if}
	</DialogContent>
</Dialog>

<!-- Hidden print-only copy -->
{#if order && open}
	<div class="print-only">
		<ReceiptTemplate
			{order}
			{items}
			{payments}
			{customer}
			{storeName}
			{storeAddress}
			{storePhone}
			{taxLabel}
			{currencySymbol}
			{printerType}
		/>
	</div>
{/if}

<style>
	.receipt-preview {
		border: 1px solid hsl(var(--border));
		border-radius: 8px;
		overflow: hidden;
		background: white;
	}

	/* Hide print-only div normally */
	.print-only {
		display: none;
	}

	/* Print styles */
	@media print {
		/* Hide everything except print-only */
		.no-print,
		:global(body > *:not(.print-only)) {
			display: none !important;
		}

		/* Show the print-only div */
		.print-only {
			display: block !important;
		}
	}
</style>
