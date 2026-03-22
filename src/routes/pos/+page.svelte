<script lang="ts">
	import { onMount } from "svelte";
	import ProductGrid from "$lib/components/pos/ProductGrid.svelte";
	import OrderPanel from "$lib/components/pos/OrderPanel.svelte";
	import PaymentModal from "$lib/components/pos/PaymentModal.svelte";
	import ReceiptPreviewDialog from "$lib/components/receipts/ReceiptPreviewDialog.svelte";
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
	import { withTransaction } from "$lib/db/index.js";
	import { printReceipt, type ReceiptData } from "$lib/commands/printing.js";
	import { formatCurrency } from "$lib/utils.js";
	import { log } from "$lib/utils/logger.js";
	import { AlertTriangle } from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { toast } from "svelte-sonner";
	import type {
		Product,
		Category,
		PartialPayment,
		Order,
		OrderItem,
		Payment,
		Customer
	} from "$lib/types/index.js";

	let products = $state<Product[]>([]);
	let categories = $state<Category[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let paymentOpen = $state(false);

	// Receipt preview state
	let receiptPreviewOpen = $state(false);
	let pendingReceiptData = $state<ReceiptData | null>(null);
	let pendingPreviewOrder = $state<Order | null>(null);
	let pendingPreviewItems = $state<OrderItem[]>([]);
	let pendingPreviewPayments = $state<Payment[]>([]);
	let pendingPreviewCustomer = $state<Customer | null>(null);

	const currencySymbol = $derived(settingsStore.get("currency_symbol") || "$");

	async function loadData() {
		loading = true;
		error = null;
		try {
			[products, categories] = await Promise.all([
				getProducts({ activeOnly: true }),
				getCategories()
			]);
		} catch {
			toast.error("Failed to load products");
			error = "Failed to load products and categories. Please check your connection and try again.";
			products = [];
			categories = [];
		} finally {
			loading = false;
		}
	}

	onMount(loadData);

	function handleProductClick(product: Product) {
		orderStore.addItem(product);
	}

	async function handlePay() {
		if (orderStore.items.length === 0) return;
		paymentOpen = true;
	}

	async function handleSaveDraft() {
		if (orderStore.items.length === 0) return;
		if (!session.user) return;

		try {
			// Wrap all DB writes in a transaction to prevent partial writes
			// and avoid wasting order numbers on failure
			await withTransaction(async () => {
				const orderNumber = await getNextOrderNumber();

				const { lastInsertId: orderId } = await createOrder({
					orderNumber,
					userId: session.user!.id,
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
			});

			orderStore.clear();
			toast.success("Draft saved");
		} catch (err: unknown) {
			const message = err instanceof Error ? err.message : String(err);
			log.error("order", `Failed to save draft: ${message}`);
			toast.error(`Failed to save draft: ${message}`);
		}
	}

	function handleClear() {
		orderStore.clear();
	}

	function formatDateTime(): string {
		return new Date().toLocaleString("en-US", {
			dateStyle: "short",
			timeStyle: "short"
		});
	}

	function buildReceiptData(
		orderNumber: string,
		payments: PartialPayment[]
	): ReceiptData {
		const storeName = settingsStore.get("store_name") || "Store";
		const storeAddress = settingsStore.get("store_address") || "";
		const storePhone = settingsStore.get("store_phone") || "";
		const taxLabel = settingsStore.get("tax_label") || "Tax";
		const receiptHeader = settingsStore.get("receipt_header") || "";
		const receiptFooter = settingsStore.get("receipt_footer") || "";

		return {
			store_name: storeName,
			store_address: storeAddress || undefined,
			store_phone: storePhone || undefined,
			header: receiptHeader || undefined,
			order_number: orderNumber,
			date: formatDateTime(),
			customer_name: orderStore.customer
				? `${orderStore.customer.first_name} ${orderStore.customer.last_name}`
				: undefined,
			items: orderStore.items.map((item) => {
				const lineSubtotal = item.product.sale_price_cents * item.quantity;
				const lineTax = Math.round(
					(lineSubtotal * item.product.tax_rate_bps) / 10000
				);
				return {
					name: item.product.name,
					quantity: item.quantity,
					unit_price: formatCurrency(
						item.product.sale_price_cents,
						currencySymbol
					),
					line_total: formatCurrency(lineSubtotal + lineTax, currencySymbol)
				};
			}),
			subtotal: formatCurrency(orderStore.subtotalCents, currencySymbol),
			discount: undefined,
			tax_label: taxLabel,
			tax: formatCurrency(orderStore.taxTotalCents, currencySymbol),
			total: formatCurrency(orderStore.totalCents, currencySymbol),
			payments: payments.map((p) => {
				let method = "Cash";
				if (p.method === "credit_card") method = "Card";
				else if (p.method === "check") method = "Check";
				else if (p.method === "other") method = "Other";

				let reference: string | undefined;
				if (p.cardDetails) {
					reference = `${p.cardDetails.cardType} ending in ${p.cardDetails.lastFour}`;
				}

				return {
					method,
					amount: formatCurrency(p.amountCents, currencySymbol),
					change:
						p.changeCents > 0
							? formatCurrency(p.changeCents, currencySymbol)
							: undefined,
					reference
				};
			}),
			footer: receiptFooter || undefined
		};
	}

	/**
	 * Build fake Order/OrderItem/Payment objects for the ReceiptTemplate preview.
	 * These are only used for the visual preview — the actual printing uses ReceiptData.
	 */
	function buildPreviewObjects(
		orderNumber: string,
		payments: PartialPayment[]
	) {
		const now = new Date().toISOString();

		const previewOrder: Order = {
			id: 0,
			uuid: "",
			order_number: orderNumber,
			status: "completed",
			customer_id: orderStore.customer?.id ?? null,
			user_id: session.user?.id ?? 0,
			subtotal_cents: orderStore.subtotalCents,
			discount_cents: 0,
			tax_total_cents: orderStore.taxTotalCents,
			total_cents: orderStore.totalCents,
			notes: orderStore.notes || null,
			completed_at: now,
			created_at: now,
			updated_at: now,
			deleted_at: null
		};

		const previewItems: OrderItem[] = orderStore.items.map((item, i) => {
			const lineSubtotal = item.product.sale_price_cents * item.quantity;
			const lineTax = Math.round(
				(lineSubtotal * item.product.tax_rate_bps) / 10000
			);
			return {
				id: i,
				uuid: "",
				order_id: 0,
				product_id: item.product.id,
				product_name: item.product.name,
				product_sku: item.product.sku ?? null,
				quantity: item.quantity,
				unit_price_cents: item.product.sale_price_cents,
				tax_rate_bps: item.product.tax_rate_bps,
				line_subtotal_cents: lineSubtotal,
				line_tax_cents: lineTax,
				line_total_cents: lineSubtotal + lineTax,
				notes: item.notes || null,
				created_at: now,
				updated_at: now
			};
		});

		const previewPayments: Payment[] = payments.map((p, i) => {
			let referenceNumber = "";
			if (p.cardDetails) {
				referenceNumber = `${p.cardDetails.cardType} ending in ${p.cardDetails.lastFour}, Auth: ${p.cardDetails.authCode}`;
			}
			return {
				id: i,
				uuid: "",
				order_id: 0,
				method: p.method,
				amount_cents: p.amountCents,
				change_cents: p.changeCents,
				reference_number: referenceNumber || null,
				card_auth_code: p.cardDetails?.authCode ?? null,
				card_last_four: p.cardDetails?.lastFour ?? null,
				card_type: p.cardDetails?.cardType ?? null,
				card_entry_mode: p.cardDetails?.entryMode ?? null,
				gateway_ref_num: p.cardDetails?.gatewayRefNum ?? null,
				gateway_response: p.cardDetails?.gatewayResponse ?? null,
				created_at: now,
				updated_at: now
			};
		});

		return { previewOrder, previewItems, previewPayments };
	}

	async function finalizeOrder() {
		// Clear cart and close modal
		paymentOpen = false;
		orderStore.clear();

		// Reload products to get updated stock
		products = await getProducts({ activeOnly: true });
		toast.success("Payment completed");
	}

	async function handlePaymentComplete(payments: PartialPayment[]) {
		if (!session.user) return;

		try {
			// Wrap all DB writes in a transaction to prevent partial writes
			// and avoid wasting order numbers on failure
			let orderNumber = "";
			await withTransaction(async () => {
				// Generate order number inside transaction so it's not wasted on failure
				orderNumber = await getNextOrderNumber();

				// Create order
				const { lastInsertId: orderId } = await createOrder({
					orderNumber,
					userId: session.user!.id,
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

				// Record all payments
				for (const payment of payments) {
					let referenceNumber = "";

					if (payment.cardDetails) {
						referenceNumber = `${payment.cardDetails.cardType} ending in ${payment.cardDetails.lastFour}, Auth: ${payment.cardDetails.authCode}`;
					}

					await addPayment({
						orderId,
						method: payment.method,
						amountCents: payment.amountCents,
						changeCents: payment.changeCents,
						referenceNumber: referenceNumber || undefined,
						cardAuthCode: payment.cardDetails?.authCode,
						cardLastFour: payment.cardDetails?.lastFour,
						cardType: payment.cardDetails?.cardType,
						cardEntryMode: payment.cardDetails?.entryMode,
						gatewayRefNum: payment.cardDetails?.gatewayRefNum,
						gatewayResponse: payment.cardDetails?.gatewayResponse
					});
				}

				// Complete order
				await completeOrder(orderId);
			});

			// Handle receipt printing
			const receiptMode = settingsStore.get("receipt_mode") || "off";
			const printerName = settingsStore.get("printer_name");
			if (receiptMode !== "off" && printerName) {
				const receiptData = buildReceiptData(orderNumber, payments);

				if (receiptMode === "prompt") {
					// Show preview dialog — defer cart clearing
					const { previewOrder, previewItems, previewPayments } =
						buildPreviewObjects(orderNumber, payments);
					pendingReceiptData = receiptData;
					pendingPreviewOrder = previewOrder;
					pendingPreviewItems = previewItems;
					pendingPreviewPayments = previewPayments;
					pendingPreviewCustomer = orderStore.customer
						? { ...orderStore.customer }
						: null;
					paymentOpen = false;
					receiptPreviewOpen = true;
					// Don't clear cart yet — wait for dialog close
					return;
				} else {
					// Auto-print silently
					try {
						await printReceipt(receiptData, printerName);
					} catch (err) {
						console.error("Receipt print failed:", err);
					}
				}
			}

			await finalizeOrder();
		} catch (err: unknown) {
			const message = err instanceof Error ? err.message : String(err);
			log.error("order", `Order creation failed: ${message}`);
			toast.error(`Order creation failed: ${message}`);
			throw err;
		}
	}

	async function handleReceiptPrinted() {
		receiptPreviewOpen = false;
		pendingReceiptData = null;
		await finalizeOrder();
	}

	async function handleReceiptSkipped() {
		receiptPreviewOpen = false;
		pendingReceiptData = null;
		await finalizeOrder();
	}

	function handlePaymentCancel() {
		paymentOpen = false;
	}
</script>

<div class="flex h-full flex-col">
	{#if error}
		<div class="mx-4 mt-4 flex items-center gap-2 rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-destructive">
			<AlertTriangle class="h-5 w-5 shrink-0" />
			<p class="text-sm">{error}</p>
			<Button variant="outline" size="sm" class="ml-auto" onclick={() => { error = null; loadData(); }}>
				Retry
			</Button>
		</div>
	{/if}
	<div class="flex flex-1">
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
</div>

<!-- Payment modal -->
<PaymentModal
	bind:open={paymentOpen}
	totalCents={orderStore.totalCents}
	{currencySymbol}
	customerId={orderStore.customer?.id ?? null}
	onComplete={handlePaymentComplete}
	onCancel={handlePaymentCancel}
/>

<!-- Receipt preview dialog (shown when prompt-before-print is enabled) -->
<ReceiptPreviewDialog
	bind:open={receiptPreviewOpen}
	receiptData={pendingReceiptData}
	order={pendingPreviewOrder}
	items={pendingPreviewItems}
	payments={pendingPreviewPayments}
	customer={pendingPreviewCustomer}
	storeName={settingsStore.get("store_name") || "Store"}
	storeAddress={settingsStore.get("store_address") || ""}
	storePhone={settingsStore.get("store_phone") || ""}
	taxLabel={settingsStore.get("tax_label") || "Tax"}
	{currencySymbol}
	printerType={(settingsStore.get("printer_type") || "standard") as "standard" | "thermal"}
	printerName={settingsStore.get("printer_name") || ""}
	onClose={handleReceiptSkipped}
	onPrinted={handleReceiptPrinted}
/>
