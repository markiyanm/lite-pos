<script lang="ts">
	import { onMount } from "svelte";
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import {
		ClipboardList, ArrowLeft, Loader2, Ban,
		RotateCcw, CreditCard, Banknote, FileCheck, User,
		UserPlus, X, Search
	} from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { Textarea } from "$lib/components/ui/textarea/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import { Switch } from "$lib/components/ui/switch/index.js";
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from "$lib/components/ui/table/index.js";
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle,
		DialogDescription,
		DialogFooter
	} from "$lib/components/ui/dialog/index.js";
	import { getOrder, getOrderItems, getOrderPayments, voidOrder, updateOrderCustomer } from "$lib/commands/orders.js";
	import { getCustomer, getCustomers } from "$lib/commands/customers.js";
	import { adjustStock } from "$lib/commands/products.js";
	import { getRefundsByOrder, createRefund, addRefundItem, setOrderRefunded } from "$lib/commands/refunds.js";
	import { session } from "$lib/stores/session.svelte.js";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import { formatCurrency } from "$lib/utils.js";
	import { toast } from "svelte-sonner";
	import type { Order, OrderItem, Payment, Customer, Refund } from "$lib/types/index.js";

	const paramId = $derived(page.params.id as string);
	const orderId = $derived(parseInt(paramId, 10));

	let loading = $state(true);
	let order = $state<Order | null>(null);
	let items = $state<OrderItem[]>([]);
	let payments = $state<Payment[]>([]);
	let customer = $state<Customer | null>(null);
	let refunds = $state<Refund[]>([]);

	// Void dialog
	let voidDialogOpen = $state(false);
	let voiding = $state(false);

	// Refund dialog
	let refundDialogOpen = $state(false);
	let refunding = $state(false);
	let refundReason = $state("");
	let refundItems = $state<{ orderItemId: number; name: string; maxQty: number; qty: number; unitCents: number; taxBps: number; restock: boolean }[]>([]);

	// Customer dialog
	let customerDialogOpen = $state(false);
	let customerSearch = $state("");
	let allCustomers = $state<Customer[]>([]);
	let customerLoading = $state(false);

	const filteredCustomers = $derived(() => {
		if (!customerSearch.trim()) return allCustomers;
		const q = customerSearch.toLowerCase();
		return allCustomers.filter(
			(c) =>
				`${c.first_name} ${c.last_name}`.toLowerCase().includes(q) ||
				(c.email && c.email.toLowerCase().includes(q)) ||
				(c.phone && c.phone.includes(q))
		);
	});

	const currencySymbol = $derived(settingsStore.get("currency_symbol") || "$");

	const refundTotalCents = $derived(() => {
		return refundItems.reduce((sum, ri) => {
			const lineSubtotal = ri.unitCents * ri.qty;
			const lineTax = Math.round((lineSubtotal * ri.taxBps) / 10000);
			return sum + lineSubtotal + lineTax;
		}, 0);
	});

	const canRefund = $derived(() => {
		return refundItems.some((ri) => ri.qty > 0);
	});

	onMount(loadOrder);

	async function loadOrder() {
		loading = true;
		try {
			order = await getOrder(orderId);
			if (order) {
				[items, payments, refunds] = await Promise.all([
					getOrderItems(orderId),
					getOrderPayments(orderId),
					getRefundsByOrder(orderId)
				]);
				if (order.customer_id) {
					customer = await getCustomer(order.customer_id);
				}
			}
		} catch {
			toast.error("Failed to load order");
			order = null;
		} finally {
			loading = false;
		}
	}

	function statusVariant(status: string): "default" | "secondary" | "outline" | "destructive" {
		switch (status) {
			case "completed": return "default";
			case "draft": return "secondary";
			case "void": return "destructive";
			case "refunded": return "outline";
			default: return "secondary";
		}
	}

	function paymentMethodLabel(method: string): string {
		switch (method) {
			case "cash": return "Cash";
			case "credit_card": return "Card";
			case "check": return "Check";
			default: return method;
		}
	}

	function paymentMethodIcon(method: string) {
		switch (method) {
			case "cash": return Banknote;
			case "credit_card": return CreditCard;
			case "check": return FileCheck;
			default: return Banknote;
		}
	}

	function formatDate(iso: string): string {
		return new Date(iso).toLocaleDateString(undefined, {
			year: "numeric",
			month: "short",
			day: "numeric",
			hour: "2-digit",
			minute: "2-digit"
		});
	}

	// Void
	async function handleVoid() {
		voiding = true;
		try {
			await voidOrder(orderId);
			voidDialogOpen = false;
			toast.success("Order voided");
			await loadOrder();
		} catch {
			toast.error("Failed to void order");
		} finally {
			voiding = false;
		}
	}

	// Refund
	function openRefundDialog() {
		refundReason = "";
		refundItems = items.map((item) => ({
			orderItemId: item.id,
			name: item.product_name,
			maxQty: item.quantity,
			qty: item.quantity,
			unitCents: item.unit_price_cents,
			taxBps: item.tax_rate_bps,
			restock: true
		}));
		refundDialogOpen = true;
	}

	async function handleRefund() {
		if (!session.user || !order) return;
		const itemsToRefund = refundItems.filter((ri) => ri.qty > 0);
		if (itemsToRefund.length === 0) return;

		refunding = true;
		try {
			const totalRefundCents = refundTotalCents();

			// Create refund record
			const { lastInsertId: refundId } = await createRefund({
				orderId: order.id,
				userId: session.user.id,
				totalRefundCents,
				reason: refundReason.trim() || undefined
			});

			// Add refund items + restock
			for (const ri of itemsToRefund) {
				const lineSubtotal = ri.unitCents * ri.qty;
				const lineTax = Math.round((lineSubtotal * ri.taxBps) / 10000);
				await addRefundItem({
					refundId,
					orderItemId: ri.orderItemId,
					quantity: ri.qty,
					refundAmountCents: lineSubtotal + lineTax,
					restock: ri.restock
				});

				// Restock if requested
				if (ri.restock) {
					const item = items.find((i) => i.id === ri.orderItemId);
					if (item?.product_id) {
						await adjustStock(item.product_id, ri.qty);
					}
				}
			}

			// Mark order as refunded
			await setOrderRefunded(order.id);

			refundDialogOpen = false;
			toast.success("Refund processed");
			await loadOrder();
		} catch {
			toast.error("Failed to process refund");
		} finally {
			refunding = false;
		}
	}

	// Customer management
	async function openCustomerDialog() {
		customerSearch = "";
		customerLoading = true;
		customerDialogOpen = true;
		try {
			allCustomers = await getCustomers();
		} catch {
			toast.error("Failed to load customers");
		} finally {
			customerLoading = false;
		}
	}

	async function assignCustomer(c: Customer) {
		if (!order) return;
		try {
			await updateOrderCustomer(order.id, c.id);
			customer = c;
			order.customer_id = c.id;
			customerDialogOpen = false;
			toast.success(`Customer ${c.first_name} ${c.last_name} assigned`);
		} catch {
			toast.error("Failed to assign customer");
		}
	}

	async function removeCustomer() {
		if (!order) return;
		try {
			await updateOrderCustomer(order.id, null);
			customer = null;
			order.customer_id = null;
			toast.success("Customer removed from order");
		} catch {
			toast.error("Failed to remove customer");
		}
	}
</script>

<div class="p-6">
	<!-- Header -->
	<div class="mb-6 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<Button variant="ghost" size="sm" onclick={() => goto("/orders")}>
				<ArrowLeft class="h-4 w-4" />
			</Button>
			<ClipboardList class="h-6 w-6" />
			<h1 class="text-2xl font-semibold">
				{order ? `Order ${order.order_number}` : "Order Detail"}
			</h1>
			{#if order}
				<Badge variant={statusVariant(order.status)}>{order.status}</Badge>
			{/if}
		</div>
		{#if order && order.status === "completed"}
			<div class="flex gap-2">
				<Button variant="outline" onclick={openRefundDialog}>
					<RotateCcw class="mr-2 h-4 w-4" />
					Refund
				</Button>
				<Button variant="destructive" onclick={() => (voidDialogOpen = true)}>
					<Ban class="mr-2 h-4 w-4" />
					Void
				</Button>
			</div>
		{/if}
	</div>

	{#if loading}
		<div class="flex justify-center py-16">
			<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if !order}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-16">
				<ClipboardList class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<p class="text-lg font-medium">Order not found</p>
			</CardContent>
		</Card>
	{:else}
		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content (2 cols) -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Line Items -->
				<Card>
					<CardHeader>
						<CardTitle>Items</CardTitle>
					</CardHeader>
					<CardContent>
						<Table>
							<TableHeader>
								<TableRow>
									<TableHead>Product</TableHead>
									<TableHead class="text-right">Price</TableHead>
									<TableHead class="text-right">Qty</TableHead>
									<TableHead class="text-right">Tax</TableHead>
									<TableHead class="text-right">Total</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{#each items as item (item.id)}
									<TableRow>
										<TableCell>
											<p class="font-medium">{item.product_name}</p>
											{#if item.product_sku}
												<p class="text-xs text-muted-foreground font-mono">{item.product_sku}</p>
											{/if}
										</TableCell>
										<TableCell class="text-right">
											{formatCurrency(item.unit_price_cents, currencySymbol)}
										</TableCell>
										<TableCell class="text-right">
											{item.quantity}
										</TableCell>
										<TableCell class="text-right text-muted-foreground">
											{formatCurrency(item.line_tax_cents, currencySymbol)}
										</TableCell>
										<TableCell class="text-right font-medium">
											{formatCurrency(item.line_total_cents, currencySymbol)}
										</TableCell>
									</TableRow>
								{/each}
							</TableBody>
						</Table>
					</CardContent>
				</Card>

				<!-- Payments -->
				<Card>
					<CardHeader>
						<CardTitle>Payments</CardTitle>
					</CardHeader>
					<CardContent>
						{#if payments.length === 0}
							<p class="text-sm text-muted-foreground">No payments recorded.</p>
						{:else}
							<div class="space-y-3">
								{#each payments as payment (payment.id)}
									{@const Icon = paymentMethodIcon(payment.method)}
									<div class="flex items-center justify-between rounded-md border p-3">
										<div class="flex items-center gap-3">
											<Icon class="h-5 w-5 text-muted-foreground" />
											<div>
												<p class="text-sm font-medium">{paymentMethodLabel(payment.method)}</p>
												<p class="text-xs text-muted-foreground">
													{formatDate(payment.created_at)}
												</p>
											</div>
										</div>
										<div class="text-right">
											<p class="font-medium">{formatCurrency(payment.amount_cents, currencySymbol)}</p>
											{#if payment.change_cents > 0}
												<p class="text-xs text-muted-foreground">
													Change: {formatCurrency(payment.change_cents, currencySymbol)}
												</p>
											{/if}
										</div>
									</div>
								{/each}
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Refund History -->
				{#if refunds.length > 0}
					<Card>
						<CardHeader>
							<CardTitle>Refunds</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-3">
								{#each refunds as refund (refund.id)}
									<div class="rounded-md border border-destructive/30 bg-destructive/5 p-3">
										<div class="flex items-center justify-between">
											<div>
												<p class="text-sm font-medium text-destructive">
													Refund — {formatCurrency(refund.total_refund_cents, currencySymbol)}
												</p>
												<p class="text-xs text-muted-foreground">{formatDate(refund.created_at)}</p>
												{#if refund.reason}
													<p class="mt-1 text-sm text-muted-foreground">{refund.reason}</p>
												{/if}
											</div>
										</div>
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}
			</div>

			<!-- Sidebar (1 col) -->
			<div class="space-y-6">
				<!-- Order Summary -->
				<Card>
					<CardHeader>
						<CardTitle>Summary</CardTitle>
					</CardHeader>
					<CardContent class="space-y-3">
						<div class="flex justify-between text-sm">
							<span class="text-muted-foreground">Subtotal</span>
							<span>{formatCurrency(order.subtotal_cents, currencySymbol)}</span>
						</div>
						{#if order.discount_cents > 0}
							<div class="flex justify-between text-sm">
								<span class="text-muted-foreground">Discount</span>
								<span class="text-green-600">-{formatCurrency(order.discount_cents, currencySymbol)}</span>
							</div>
						{/if}
						<div class="flex justify-between text-sm">
							<span class="text-muted-foreground">Tax</span>
							<span>{formatCurrency(order.tax_total_cents, currencySymbol)}</span>
						</div>
						<Separator />
						<div class="flex justify-between font-semibold">
							<span>Total</span>
							<span>{formatCurrency(order.total_cents, currencySymbol)}</span>
						</div>
					</CardContent>
				</Card>

				<!-- Order Details -->
				<Card>
					<CardHeader>
						<CardTitle>Details</CardTitle>
					</CardHeader>
					<CardContent class="space-y-3 text-sm">
						<div class="flex justify-between">
							<span class="text-muted-foreground">Order #</span>
							<span class="font-mono">{order.order_number}</span>
						</div>
						<div class="flex justify-between">
							<span class="text-muted-foreground">Created</span>
							<span>{formatDate(order.created_at)}</span>
						</div>
						{#if order.completed_at}
							<div class="flex justify-between">
								<span class="text-muted-foreground">Completed</span>
								<span>{formatDate(order.completed_at)}</span>
							</div>
						{/if}
						{#if order.notes}
							<Separator />
							<div>
								<p class="text-muted-foreground mb-1">Notes</p>
								<p>{order.notes}</p>
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Customer -->
				<Card>
					<CardHeader class="flex flex-row items-center justify-between space-y-0">
						<CardTitle>Customer</CardTitle>
						{#if customer}
							<div class="flex gap-1">
								<Button variant="ghost" size="icon" class="h-7 w-7" onclick={openCustomerDialog} title="Change customer">
									<UserPlus class="h-3.5 w-3.5" />
								</Button>
								<Button variant="ghost" size="icon" class="h-7 w-7 text-destructive" onclick={removeCustomer} title="Remove customer">
									<X class="h-3.5 w-3.5" />
								</Button>
							</div>
						{/if}
					</CardHeader>
					<CardContent>
						{#if customer}
							<button
								type="button"
								class="flex w-full items-center gap-3 rounded-md p-2 text-left hover:bg-muted/50"
								onclick={() => goto(`/customers/${customer!.id}`)}
							>
								<div class="flex h-9 w-9 items-center justify-center rounded-full bg-muted">
									<User class="h-4 w-4 text-muted-foreground" />
								</div>
								<div>
									<p class="text-sm font-medium">{customer.first_name} {customer.last_name}</p>
									{#if customer.email}
										<p class="text-xs text-muted-foreground">{customer.email}</p>
									{/if}
								</div>
							</button>
						{:else}
							<div class="flex flex-col items-center gap-3 py-2">
								<p class="text-sm text-muted-foreground">No customer attached</p>
								<Button variant="outline" size="sm" onclick={openCustomerDialog}>
									<UserPlus class="mr-2 h-4 w-4" />
									Assign Customer
								</Button>
							</div>
						{/if}
					</CardContent>
				</Card>
			</div>
		</div>
	{/if}
</div>

<!-- Void Confirmation Dialog -->
<Dialog bind:open={voidDialogOpen}>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Void Order</DialogTitle>
			<DialogDescription>
				Are you sure you want to void order {order?.order_number}? This will mark the order as void. This action cannot be undone.
			</DialogDescription>
		</DialogHeader>
		<DialogFooter>
			<Button variant="outline" onclick={() => (voidDialogOpen = false)}>Cancel</Button>
			<Button variant="destructive" onclick={handleVoid} disabled={voiding}>
				{#if voiding}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
				{/if}
				Void Order
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>

<!-- Refund Dialog -->
<Dialog bind:open={refundDialogOpen}>
	<DialogContent class="sm:max-w-lg">
		<DialogHeader>
			<DialogTitle>Process Refund</DialogTitle>
			<DialogDescription>
				Select items and quantities to refund for order {order?.order_number}.
			</DialogDescription>
		</DialogHeader>

		<div class="space-y-4 py-2">
			<!-- Refund items -->
			<div class="space-y-3">
				{#each refundItems as ri, idx}
					<div class="rounded-md border p-3">
						<div class="mb-2 flex items-center justify-between">
							<p class="text-sm font-medium">{ri.name}</p>
							<p class="text-sm text-muted-foreground">
								{formatCurrency(ri.unitCents, currencySymbol)} each
							</p>
						</div>
						<div class="flex items-center gap-4">
							<div class="flex items-center gap-2">
								<Label class="text-xs text-muted-foreground">Qty</Label>
								<Input
									type="number"
									min="0"
									max={ri.maxQty}
									value={String(ri.qty)}
									oninput={(e: Event) => {
										const val = parseInt((e.target as HTMLInputElement).value, 10);
										refundItems[idx].qty = isNaN(val) ? 0 : Math.min(Math.max(0, val), ri.maxQty);
									}}
									class="w-20 h-8 text-sm"
								/>
								<span class="text-xs text-muted-foreground">of {ri.maxQty}</span>
							</div>
							<div class="flex items-center gap-2 ml-auto">
								<Switch
									checked={ri.restock}
									onCheckedChange={(v) => { refundItems[idx].restock = v; }}
								/>
								<Label class="text-xs">Restock</Label>
							</div>
						</div>
					</div>
				{/each}
			</div>

			<!-- Reason -->
			<div class="grid gap-2">
				<Label>Reason (optional)</Label>
				<Textarea
					bind:value={refundReason}
					placeholder="Reason for refund..."
					rows={2}
				/>
			</div>

			<Separator />

			<!-- Refund total -->
			<div class="flex justify-between text-lg font-semibold">
				<span>Refund Total</span>
				<span class="text-destructive">{formatCurrency(refundTotalCents(), currencySymbol)}</span>
			</div>
		</div>

		<DialogFooter>
			<Button variant="outline" onclick={() => (refundDialogOpen = false)}>Cancel</Button>
			<Button
				variant="destructive"
				onclick={handleRefund}
				disabled={refunding || !canRefund()}
			>
				{#if refunding}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
				{/if}
				Process Refund
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>

<!-- Customer Selection Dialog -->
<Dialog bind:open={customerDialogOpen}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>Select Customer</DialogTitle>
			<DialogDescription>
				Search and select a customer to assign to this order.
			</DialogDescription>
		</DialogHeader>
		<div class="space-y-3 py-2">
			<div class="relative">
				<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
				<Input
					bind:value={customerSearch}
					placeholder="Search customers..."
					class="pl-9"
				/>
			</div>
			{#if customerLoading}
				<div class="flex justify-center py-6">
					<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
				</div>
			{:else if filteredCustomers().length === 0}
				<p class="py-6 text-center text-sm text-muted-foreground">No customers found.</p>
			{:else}
				<div class="max-h-64 space-y-1 overflow-y-auto">
					{#each filteredCustomers() as c (c.id)}
						<button
							type="button"
							class="flex w-full items-center gap-3 rounded-md p-2 text-left hover:bg-accent/50"
							onclick={() => assignCustomer(c)}
						>
							<div class="flex h-8 w-8 items-center justify-center rounded-full bg-muted">
								<User class="h-4 w-4 text-muted-foreground" />
							</div>
							<div>
								<p class="text-sm font-medium">{c.first_name} {c.last_name}</p>
								{#if c.email}
									<p class="text-xs text-muted-foreground">{c.email}</p>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{/if}
		</div>
	</DialogContent>
</Dialog>
