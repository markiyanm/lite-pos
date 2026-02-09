<script lang="ts">
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import {
		ClipboardList, Search, Loader2, DollarSign,
		ShoppingCart, TrendingUp, Calendar
	} from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card/index.js";
	import {
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger
	} from "$lib/components/ui/select/index.js";
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from "$lib/components/ui/table/index.js";
	import { getOrders } from "$lib/commands/orders.js";
	import { getCustomers } from "$lib/commands/customers.js";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import { formatCurrency } from "$lib/utils.js";
	import { toast } from "svelte-sonner";
	import type { Order, Customer } from "$lib/types/index.js";

	let orders = $state<Order[]>([]);
	let customers = $state<Customer[]>([]);
	let loading = $state(true);
	let searchQuery = $state("");
	let filterStatus = $state<string>("");
	let filterDateFrom = $state("");
	let filterDateTo = $state("");

	const currencySymbol = $derived(settingsStore.get("currency_symbol") || "$");

	const customerMap = $derived(
		new Map(customers.map((c) => [c.id, c]))
	);

	const filteredOrders = $derived(() => {
		let result = orders;
		if (searchQuery.trim()) {
			const q = searchQuery.toLowerCase();
			result = result.filter((o) => {
				if (o.order_number.toLowerCase().includes(q)) return true;
				if (o.customer_id) {
					const cust = customerMap.get(o.customer_id);
					if (cust) {
						const name = `${cust.first_name} ${cust.last_name}`.toLowerCase();
						if (name.includes(q)) return true;
					}
				}
				return false;
			});
		}
		if (filterStatus) {
			result = result.filter((o) => o.status === filterStatus);
		}
		if (filterDateFrom) {
			result = result.filter((o) => o.created_at >= filterDateFrom);
		}
		if (filterDateTo) {
			// Add end of day to include the full day
			const endDate = filterDateTo + "T23:59:59";
			result = result.filter((o) => o.created_at <= endDate);
		}
		return result;
	});

	// Stats derived from filtered orders
	const stats = $derived(() => {
		const completed = filteredOrders().filter((o) => o.status === "completed");
		const totalSales = completed.reduce((sum, o) => sum + o.total_cents, 0);
		const orderCount = completed.length;
		const avgOrder = orderCount > 0 ? Math.round(totalSales / orderCount) : 0;
		return { totalSales, orderCount, avgOrder };
	});

	const statusLabel = $derived(() => {
		if (!filterStatus) return "All Statuses";
		return filterStatus.charAt(0).toUpperCase() + filterStatus.slice(1);
	});

	onMount(async () => {
		loading = true;
		try {
			[orders, customers] = await Promise.all([
				getOrders(),
				getCustomers()
			]);
		} catch {
			toast.error("Failed to load orders");
			orders = [];
			customers = [];
		} finally {
			loading = false;
		}
	});

	function customerName(customerId: number | null): string {
		if (!customerId) return "—";
		const c = customerMap.get(customerId);
		return c ? `${c.first_name} ${c.last_name}` : "—";
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

	function formatDate(iso: string): string {
		return new Date(iso).toLocaleDateString(undefined, {
			year: "numeric",
			month: "short",
			day: "numeric",
			hour: "2-digit",
			minute: "2-digit"
		});
	}

	function clearFilters() {
		searchQuery = "";
		filterStatus = "";
		filterDateFrom = "";
		filterDateTo = "";
	}

	const hasFilters = $derived(
		searchQuery.trim() !== "" || filterStatus !== "" || filterDateFrom !== "" || filterDateTo !== ""
	);
</script>

<div class="p-6">
	<div class="mb-6 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<ClipboardList class="h-6 w-6" />
			<h1 class="text-2xl font-semibold">Orders</h1>
			<Badge variant="secondary">{orders.length}</Badge>
		</div>
	</div>

	<!-- Stats Cards -->
	{#if !loading}
		<div class="mb-6 grid grid-cols-1 gap-4 sm:grid-cols-3">
			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium">Total Sales</CardTitle>
					<DollarSign class="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{formatCurrency(stats().totalSales, currencySymbol)}</div>
					<p class="text-xs text-muted-foreground">From completed orders</p>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium">Order Count</CardTitle>
					<ShoppingCart class="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats().orderCount}</div>
					<p class="text-xs text-muted-foreground">Completed orders</p>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium">Average Order</CardTitle>
					<TrendingUp class="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{formatCurrency(stats().avgOrder, currencySymbol)}</div>
					<p class="text-xs text-muted-foreground">Per completed order</p>
				</CardContent>
			</Card>
		</div>
	{/if}

	<!-- Filters -->
	<div class="mb-4 flex flex-wrap items-center gap-3">
		<div class="relative flex-1 min-w-[200px] max-w-sm">
			<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
			<Input
				bind:value={searchQuery}
				placeholder="Search orders or customers..."
				class="pl-9"
			/>
		</div>
		<Select
			type="single"
			value={filterStatus}
			onValueChange={(v) => (filterStatus = v ?? "")}
		>
			<SelectTrigger class="w-40">{statusLabel()}</SelectTrigger>
			<SelectContent>
				<SelectItem value="">All Statuses</SelectItem>
				<SelectItem value="completed">Completed</SelectItem>
				<SelectItem value="draft">Draft</SelectItem>
				<SelectItem value="refunded">Refunded</SelectItem>
				<SelectItem value="void">Void</SelectItem>
			</SelectContent>
		</Select>
		<div class="flex items-center gap-2">
			<Calendar class="h-4 w-4 text-muted-foreground" />
			<Input
				type="date"
				bind:value={filterDateFrom}
				class="w-36"
			/>
			<span class="text-sm text-muted-foreground">to</span>
			<Input
				type="date"
				bind:value={filterDateTo}
				class="w-36"
			/>
		</div>
		{#if hasFilters}
			<Button variant="ghost" size="sm" onclick={clearFilters}>
				Clear filters
			</Button>
		{/if}
	</div>

	{#if loading}
		<div class="flex justify-center py-16">
			<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if orders.length === 0}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-16">
				<ClipboardList class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<p class="mb-2 text-lg font-medium">No orders yet</p>
				<p class="mb-4 text-sm text-muted-foreground">Orders will appear here after your first sale.</p>
				<Button onclick={() => goto("/pos")}>
					Go to POS
				</Button>
			</CardContent>
		</Card>
	{:else}
		<Card>
			<Table>
				<TableHeader>
					<TableRow>
						<TableHead>Order #</TableHead>
						<TableHead>Date</TableHead>
						<TableHead>Customer</TableHead>
						<TableHead>Status</TableHead>
						<TableHead class="text-right">Total</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{#each filteredOrders() as order (order.id)}
						<TableRow
							class="cursor-pointer"
							onclick={() => goto(`/orders/${order.id}`)}
						>
							<TableCell class="font-mono text-sm font-medium">
								{order.order_number}
							</TableCell>
							<TableCell class="text-sm">
								{formatDate(order.created_at)}
							</TableCell>
							<TableCell>
								{customerName(order.customer_id)}
							</TableCell>
							<TableCell>
								<Badge variant={statusVariant(order.status)}>
									{order.status}
								</Badge>
							</TableCell>
							<TableCell class="text-right font-medium">
								{formatCurrency(order.total_cents, currencySymbol)}
							</TableCell>
						</TableRow>
					{/each}
				</TableBody>
			</Table>
		</Card>

		{#if filteredOrders().length === 0 && hasFilters}
			<p class="py-8 text-center text-sm text-muted-foreground">
				No orders match your filters.
			</p>
		{/if}
	{/if}
</div>
