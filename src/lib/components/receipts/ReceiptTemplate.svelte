<script lang="ts">
	import { formatCurrency } from "$lib/utils.js";
	import type { Order, OrderItem, Payment, Customer } from "$lib/types/index.js";

	interface Props {
		order: Order;
		items: OrderItem[];
		payments: Payment[];
		customer: Customer | null;
		storeName: string;
		storeAddress: string;
		storePhone: string;
		receiptHeader?: string;
		receiptFooter?: string;
		taxLabel: string;
		currencySymbol: string;
		printerType: "standard" | "thermal";
	}

	let {
		order,
		items,
		payments,
		customer,
		storeName,
		storeAddress,
		storePhone,
		receiptHeader = "",
		receiptFooter = "",
		taxLabel,
		currencySymbol,
		printerType
	}: Props = $props();

	function formatDateTime(dateStr: string): string {
		const date = new Date(dateStr);
		return date.toLocaleString("en-US", {
			dateStyle: "short",
			timeStyle: "short"
		});
	}

	function getPaymentMethodLabel(method: string): string {
		switch (method) {
			case "cash":
				return "Cash";
			case "credit_card":
				return "Card";
			case "check":
				return "Check";
			case "other":
				return "Other";
			default:
				return method;
		}
	}
</script>

<div class="receipt-container" data-printer-type={printerType}>
	<div class="receipt-content">
		<!-- Store Header -->
		<div class="store-header">
			<div class="store-name">{storeName}</div>
			{#if storeAddress}
				<div class="store-info">{storeAddress}</div>
			{/if}
			{#if storePhone}
				<div class="store-info">{storePhone}</div>
			{/if}
			{#if receiptHeader}
				<div class="receipt-header-text">{receiptHeader}</div>
			{/if}
		</div>

		<div class="divider"></div>

		<!-- Order Info -->
		<div class="order-info">
			<div class="info-row">
				<span>Order #:</span>
				<span>{order.order_number}</span>
			</div>
			<div class="info-row">
				<span>Date:</span>
				<span>{formatDateTime(order.completed_at || order.created_at)}</span>
			</div>
			{#if customer}
				<div class="info-row">
					<span>Customer:</span>
					<span>{customer.first_name} {customer.last_name}</span>
				</div>
			{/if}
		</div>

		<div class="divider"></div>

		<!-- Items -->
		<div class="items-section">
			{#each items as item}
				<div class="item-row">
					<div class="item-name-line">
						<span class="item-name">{item.product_name}</span>
					</div>
					<div class="item-detail-line">
						<span class="item-qty-price"
							>{item.quantity} x {formatCurrency(item.unit_price_cents, currencySymbol)}</span
						>
						<span class="item-total">{formatCurrency(item.line_total_cents, currencySymbol)}</span
						>
					</div>
				</div>
			{/each}
		</div>

		<div class="divider"></div>

		<!-- Totals -->
		<div class="totals-section">
			<div class="total-row">
				<span>Subtotal:</span>
				<span>{formatCurrency(order.subtotal_cents, currencySymbol)}</span>
			</div>
			{#if order.discount_cents > 0}
				<div class="total-row">
					<span>Discount:</span>
					<span>-{formatCurrency(order.discount_cents, currencySymbol)}</span>
				</div>
			{/if}
			<div class="total-row">
				<span>{taxLabel}:</span>
				<span>{formatCurrency(order.tax_total_cents, currencySymbol)}</span>
			</div>
			<div class="total-row total">
				<span>Total:</span>
				<span>{formatCurrency(order.total_cents, currencySymbol)}</span>
			</div>
		</div>

		<div class="divider"></div>

		<!-- Payments -->
		<div class="payments-section">
			{#each payments as payment}
				<div class="payment-row">
					<span>{getPaymentMethodLabel(payment.method)}</span>
					<span>{formatCurrency(payment.amount_cents, currencySymbol)}</span>
				</div>
				{#if payment.change_cents > 0}
					<div class="payment-row change">
						<span>Change:</span>
						<span>{formatCurrency(payment.change_cents, currencySymbol)}</span>
					</div>
				{/if}
				{#if payment.reference_number}
					<div class="payment-ref">{payment.reference_number}</div>
				{/if}
			{/each}
		</div>

		<div class="divider"></div>

		<!-- Footer -->
		{#if receiptFooter}
			<div class="receipt-footer">
				<div>{receiptFooter}</div>
			</div>
		{/if}
	</div>
</div>

<style>
	/* Base styles - shared between thermal and standard */
	.receipt-container {
		background: white;
		color: black;
		font-family: "Courier New", monospace;
		padding: 1rem;
	}

	.receipt-content {
		max-width: 100%;
	}

	.store-header {
		text-align: center;
		margin-bottom: 1rem;
	}

	.store-name {
		font-size: 1.25rem;
		font-weight: bold;
		margin-bottom: 0.25rem;
	}

	.store-info {
		font-size: 0.875rem;
		margin-bottom: 0.125rem;
	}

	.receipt-header-text {
		font-size: 0.875rem;
		margin-top: 0.375rem;
		white-space: pre-line;
	}

	.divider {
		border-top: 1px dashed #333;
		margin: 0.75rem 0;
	}

	.order-info,
	.totals-section,
	.payments-section {
		font-size: 0.875rem;
	}

	.info-row,
	.total-row,
	.payment-row {
		display: flex;
		justify-content: space-between;
		margin-bottom: 0.25rem;
	}

	.items-section {
		font-size: 0.875rem;
	}

	.item-row {
		margin-bottom: 0.5rem;
	}

	.item-name-line {
		margin-bottom: 0.125rem;
	}

	.item-name {
		font-weight: 500;
	}

	.item-detail-line {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding-left: 1rem;
		font-size: 0.8rem;
		color: #444;
	}

	.item-qty-price {
		flex: 1;
	}

	.item-total {
		font-weight: 600;
		color: #000;
	}

	.total-row.total {
		font-size: 1rem;
		font-weight: bold;
		margin-top: 0.5rem;
		padding-top: 0.5rem;
		border-top: 1px solid #333;
	}

	.payment-row.change {
		font-style: italic;
		color: #666;
	}

	.payment-ref {
		font-size: 0.75rem;
		color: #666;
		margin-top: 0.125rem;
		margin-bottom: 0.5rem;
	}

	.receipt-footer {
		text-align: center;
		font-size: 0.875rem;
		margin-top: 1rem;
	}

	/* Thermal printer specific (80mm = ~302px at 96dpi) */
	.receipt-container[data-printer-type="thermal"] {
		width: 80mm;
		max-width: 80mm;
		font-size: 11px;
		padding: 0.5rem;
	}

	.receipt-container[data-printer-type="thermal"] .store-name {
		font-size: 1rem;
	}

	.receipt-container[data-printer-type="thermal"] .store-info {
		font-size: 0.75rem;
	}

	/* Standard printer specific */
	.receipt-container[data-printer-type="standard"] {
		max-width: 400px;
		margin: 0 auto;
	}

	/* Print media query */
	@media print {
		@page {
			margin: 0.5cm;
			size: auto;
		}

		.receipt-container {
			padding: 0;
			max-width: 100%;
		}

		.receipt-container[data-printer-type="thermal"] {
			width: 80mm;
			max-width: 80mm;
			font-size: 10px;
		}

		.receipt-container[data-printer-type="standard"] {
			max-width: 400px;
		}
	}
</style>
