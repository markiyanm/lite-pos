<script lang="ts">
	import { onMount } from "svelte";
	import ProductGrid from "$lib/components/pos/ProductGrid.svelte";
	import OrderPanel from "$lib/components/pos/OrderPanel.svelte";
	import PaymentModal from "$lib/components/pos/PaymentModal.svelte";
	import { orderStore } from "$lib/stores/order.svelte.js";
	import { posFilters } from "$lib/stores/pos-filters.svelte.js";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import { session } from "$lib/stores/session.svelte.js";
	import { getProducts, adjustStock } from "$lib/commands/products.js";
	import { getCategories } from "$lib/commands/categories.js";
	import {
		getNextOrderNumber,
		createOrder,
		addOrderItem,
		completeOrder
	} from "$lib/commands/orders.js";
	import { addPayment } from "$lib/commands/payments.js";
	import { toast } from "svelte-sonner";
	import type { Product, Category, PaymentMethod } from "$lib/types/index.js";

	let products = $state<Product[]>([]);
	let categories = $state<Category[]>([]);
	let loading = $state(true);
	let paymentOpen = $state(false);

	const currencySymbol = $derived(settingsStore.get("currency_symbol") || "$");

	onMount(async () => {
		loading = true;
		try {
			[products, categories] = await Promise.all([
				getProducts({ activeOnly: true }),
				getCategories()
			]);
		} catch {
			toast.error("Failed to load products");
			products = [];
			categories = [];
		} finally {
			loading = false;
		}
	});

	function handleProductClick(product: Product) {
		orderStore.addItem(product);
	}

	function handlePay() {
		if (orderStore.items.length === 0) return;
		paymentOpen = true;
	}

	async function handleSaveDraft() {
		if (orderStore.items.length === 0) return;
		if (!session.user) return;

		try {
			const orderNumber = await getNextOrderNumber();
			const { lastInsertId: orderId } = await createOrder({
				orderNumber,
				userId: session.user.id,
				customerId: orderStore.customer?.id,
				subtotalCents: orderStore.subtotalCents,
				discountCents: 0,
				taxTotalCents: orderStore.taxTotalCents,
				totalCents: orderStore.totalCents,
				notes: orderStore.notes || undefined,
				status: "draft"
			});

			// Add line items
			for (const item of orderStore.items) {
				const lineSubtotal = item.product.sale_price_cents * item.quantity;
				const lineTax = Math.round((lineSubtotal * item.product.tax_rate_bps) / 10000);
				await addOrderItem({
					orderId,
					productId: item.product.id,
					productName: item.product.name,
					productSku: item.product.sku ?? undefined,
					quantity: item.quantity,
					unitPriceCents: item.product.sale_price_cents,
					taxRateBps: item.product.tax_rate_bps,
					lineSubtotalCents: lineSubtotal,
					lineTaxCents: lineTax,
					lineTotalCents: lineSubtotal + lineTax,
					notes: item.notes || undefined
				});
			}

			orderStore.clear();
			toast.success("Draft saved");
		} catch {
			toast.error("Failed to save draft");
		}
	}

	function handleClear() {
		orderStore.clear();
	}

	async function handlePaymentComplete(method: PaymentMethod, amountCents: number) {
		if (!session.user) return;

		try {
			const orderNumber = await getNextOrderNumber();

			// Create order
			const { lastInsertId: orderId } = await createOrder({
				orderNumber,
				userId: session.user.id,
				customerId: orderStore.customer?.id,
				subtotalCents: orderStore.subtotalCents,
				discountCents: 0,
				taxTotalCents: orderStore.taxTotalCents,
				totalCents: orderStore.totalCents,
				notes: orderStore.notes || undefined,
				status: "completed"
			});

			// Add line items + deduct stock
			for (const item of orderStore.items) {
				const lineSubtotal = item.product.sale_price_cents * item.quantity;
				const lineTax = Math.round((lineSubtotal * item.product.tax_rate_bps) / 10000);
				await addOrderItem({
					orderId,
					productId: item.product.id,
					productName: item.product.name,
					productSku: item.product.sku ?? undefined,
					quantity: item.quantity,
					unitPriceCents: item.product.sale_price_cents,
					taxRateBps: item.product.tax_rate_bps,
					lineSubtotalCents: lineSubtotal,
					lineTaxCents: lineTax,
					lineTotalCents: lineSubtotal + lineTax,
					notes: item.notes || undefined
				});

				// Deduct stock
				await adjustStock(item.product.id, -item.quantity);
			}

			// Record payment
			const changeCents = Math.max(0, amountCents - orderStore.totalCents);
			await addPayment({
				orderId,
				method,
				amountCents,
				changeCents
			});

			// Complete order
			await completeOrder(orderId);

			// Clear cart and close modal
			paymentOpen = false;
			orderStore.clear();

			// Reload products to get updated stock
			products = await getProducts({ activeOnly: true });
			toast.success("Payment completed");
		} catch {
			toast.error("Payment failed");
		}
	}

	function handlePaymentCancel() {
		paymentOpen = false;
	}
</script>

<div class="flex h-full">
	<!-- Product grid (left side) -->
	<div class="flex-1 p-4">
		<ProductGrid
			{products}
			{categories}
			{loading}
			searchQuery={posFilters.searchQuery}
			selectedCategoryId={posFilters.selectedCategoryId}
			{currencySymbol}
			onSearchChange={(q) => posFilters.setSearch(q)}
			onCategoryChange={(id) => posFilters.setCategory(id)}
			onProductClick={handleProductClick}
		/>
	</div>

	<!-- Order panel (right side) -->
	<div class="w-80 shrink-0 xl:w-96">
		<OrderPanel
			{currencySymbol}
			onPay={handlePay}
			onSaveDraft={handleSaveDraft}
			onClear={handleClear}
		/>
	</div>
</div>

<!-- Payment modal -->
<PaymentModal
	bind:open={paymentOpen}
	totalCents={orderStore.totalCents}
	{currencySymbol}
	onComplete={handlePaymentComplete}
	onCancel={handlePaymentCancel}
/>
