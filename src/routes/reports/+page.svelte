<script lang="ts">
	import { onMount } from "svelte";
	import {
		BarChart3, Loader2, DollarSign, ShoppingCart,
		TrendingUp, Package, AlertTriangle, Calendar
	} from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "$lib/components/ui/card/index.js";
	import { Tabs, TabsContent, TabsList, TabsTrigger } from "$lib/components/ui/tabs/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from "$lib/components/ui/table/index.js";
	import {
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger
	} from "$lib/components/ui/select/index.js";
	import {
		getSalesByPeriod,
		getProductMetrics,
		getInventorySummary,
		type SalesByPeriod,
		type ProductMetric,
		type InventorySummary,
		type GroupBy
	} from "$lib/commands/reports.js";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import { formatCurrency } from "$lib/utils.js";
	import { toast } from "svelte-sonner";

	let loading = $state(true);

	// Sales tab state
	let salesGroupBy = $state<GroupBy>("day");
	let salesDateFrom = $state("");
	let salesDateTo = $state("");
	let salesData = $state<SalesByPeriod[]>([]);
	let salesLoading = $state(false);

	// Products tab state
	let productDateFrom = $state("");
	let productDateTo = $state("");
	let productData = $state<ProductMetric[]>([]);
	let productLoading = $state(false);

	// Inventory tab state
	let inventoryData = $state<InventorySummary[]>([]);

	const currencySymbol = $derived(settingsStore.get("currency_symbol") || "$");

	// Sales summary stats
	const salesSummary = $derived(() => {
		const totalRevenue = salesData.reduce((s, r) => s + r.total_cents, 0);
		const totalOrders = salesData.reduce((s, r) => s + r.order_count, 0);
		const totalTax = salesData.reduce((s, r) => s + r.tax_cents, 0);
		const avgOrder = totalOrders > 0 ? Math.round(totalRevenue / totalOrders) : 0;
		return { totalRevenue, totalOrders, totalTax, avgOrder };
	});

	// Product summary stats
	const productSummary = $derived(() => {
		const totalRevenue = productData.reduce((s, p) => s + p.total_revenue_cents, 0);
		const totalUnits = productData.reduce((s, p) => s + p.total_quantity, 0);
		const uniqueProducts = productData.length;
		return { totalRevenue, totalUnits, uniqueProducts };
	});

	// Inventory stats
	const inventoryStats = $derived(() => {
		const totalItems = inventoryData.length;
		const totalStock = inventoryData.reduce((s, i) => s + i.stock_quantity, 0);
		const lowStock = inventoryData.filter((i) => i.stock_quantity <= i.low_stock_threshold).length;
		const outOfStock = inventoryData.filter((i) => i.stock_quantity === 0).length;
		const totalValue = inventoryData.reduce((s, i) => s + i.cost_price_cents * i.stock_quantity, 0);
		return { totalItems, totalStock, lowStock, outOfStock, totalValue };
	});

	// Max value for bar chart scaling
	const salesMaxCents = $derived(() => {
		if (salesData.length === 0) return 1;
		return Math.max(...salesData.map((r) => r.total_cents));
	});

	const productMaxCents = $derived(() => {
		if (productData.length === 0) return 1;
		return Math.max(...productData.map((p) => p.total_revenue_cents));
	});

	const groupByLabel = $derived(() => {
		switch (salesGroupBy) {
			case "day": return "Daily";
			case "week": return "Weekly";
			case "month": return "Monthly";
		}
	});

	onMount(async () => {
		loading = true;
		try {
			// Set default date range: last 30 days
			const now = new Date();
			const thirtyDaysAgo = new Date(now);
			thirtyDaysAgo.setDate(now.getDate() - 30);
			salesDateFrom = thirtyDaysAgo.toISOString().split("T")[0];
			salesDateTo = now.toISOString().split("T")[0];
			productDateFrom = salesDateFrom;
			productDateTo = salesDateTo;

			await Promise.all([loadSales(), loadProducts(), loadInventory()]);
		} finally {
			loading = false;
		}
	});

	async function loadSales() {
		salesLoading = true;
		try {
			salesData = await getSalesByPeriod({
				groupBy: salesGroupBy,
				dateFrom: salesDateFrom || undefined,
				dateTo: salesDateTo || undefined
			});
		} catch {
			toast.error("Failed to load sales data");
			salesData = [];
		} finally {
			salesLoading = false;
		}
	}

	async function loadProducts() {
		productLoading = true;
		try {
			productData = await getProductMetrics({
				dateFrom: productDateFrom || undefined,
				dateTo: productDateTo || undefined
			});
		} catch {
			toast.error("Failed to load product data");
			productData = [];
		} finally {
			productLoading = false;
		}
	}

	async function loadInventory() {
		try {
			inventoryData = await getInventorySummary();
		} catch {
			toast.error("Failed to load inventory data");
			inventoryData = [];
		}
	}

	function formatPeriodLabel(period: string): string {
		if (salesGroupBy === "day") {
			const d = new Date(period + "T00:00:00");
			return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
		}
		if (salesGroupBy === "week") {
			return period; // e.g., "2025-W03"
		}
		if (salesGroupBy === "month") {
			const [y, m] = period.split("-");
			const d = new Date(Number(y), Number(m) - 1);
			return d.toLocaleDateString(undefined, { year: "numeric", month: "short" });
		}
		return period;
	}
</script>

<div class="p-6">
	<div class="mb-6 flex items-center gap-3">
		<BarChart3 class="h-6 w-6" />
		<h1 class="text-2xl font-semibold">Reports</h1>
	</div>

	{#if loading}
		<div class="flex justify-center py-16">
			<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else}
		<Tabs value="sales" class="w-full">
			<TabsList class="mb-4">
				<TabsTrigger value="sales">Sales</TabsTrigger>
				<TabsTrigger value="products">Products</TabsTrigger>
				<TabsTrigger value="inventory">Inventory</TabsTrigger>
			</TabsList>

			<!-- Sales Tab -->
			<TabsContent value="sales">
				<!-- Summary Cards -->
				<div class="mb-6 grid grid-cols-1 gap-4 sm:grid-cols-4">
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Total Revenue</CardTitle>
							<DollarSign class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{formatCurrency(salesSummary().totalRevenue, currencySymbol)}</div>
						</CardContent>
					</Card>
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Orders</CardTitle>
							<ShoppingCart class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{salesSummary().totalOrders}</div>
						</CardContent>
					</Card>
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Avg Order</CardTitle>
							<TrendingUp class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{formatCurrency(salesSummary().avgOrder, currencySymbol)}</div>
						</CardContent>
					</Card>
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Tax Collected</CardTitle>
							<DollarSign class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{formatCurrency(salesSummary().totalTax, currencySymbol)}</div>
						</CardContent>
					</Card>
				</div>

				<!-- Filters -->
				<div class="mb-4 flex flex-wrap items-center gap-3">
					<Select
						type="single"
						value={salesGroupBy}
						onValueChange={(v) => { salesGroupBy = (v as GroupBy) ?? "day"; loadSales(); }}
					>
						<SelectTrigger class="w-32">{groupByLabel()}</SelectTrigger>
						<SelectContent>
							<SelectItem value="day">Daily</SelectItem>
							<SelectItem value="week">Weekly</SelectItem>
							<SelectItem value="month">Monthly</SelectItem>
						</SelectContent>
					</Select>
					<div class="flex items-center gap-2">
						<Calendar class="h-4 w-4 text-muted-foreground" />
						<Input
							type="date"
							bind:value={salesDateFrom}
							class="w-36"
						/>
						<span class="text-sm text-muted-foreground">to</span>
						<Input
							type="date"
							bind:value={salesDateTo}
							class="w-36"
						/>
					</div>
					<Button variant="outline" size="sm" onclick={loadSales} disabled={salesLoading}>
						{#if salesLoading}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						{/if}
						Apply
					</Button>
				</div>

				<!-- Sales Chart (CSS bar chart) -->
				{#if salesData.length === 0}
					<Card>
						<CardContent class="flex flex-col items-center justify-center py-16">
							<BarChart3 class="mb-4 h-12 w-12 text-muted-foreground/50" />
							<p class="text-lg font-medium">No sales data</p>
							<p class="text-sm text-muted-foreground">Adjust the date range or complete some orders.</p>
						</CardContent>
					</Card>
				{:else}
					<Card>
						<CardHeader>
							<CardTitle>Sales Over Time</CardTitle>
							<CardDescription>{groupByLabel()} revenue for the selected period</CardDescription>
						</CardHeader>
						<CardContent>
							<div class="space-y-2">
								{#each salesData as row}
									{@const pct = (row.total_cents / salesMaxCents()) * 100}
									<div class="flex items-center gap-3">
										<span class="w-20 shrink-0 text-right text-xs text-muted-foreground">
											{formatPeriodLabel(row.period)}
										</span>
										<div class="flex-1">
											<div
												class="h-7 rounded bg-primary/80 transition-all"
												style="width: {Math.max(pct, 1)}%"
											></div>
										</div>
										<span class="w-24 shrink-0 text-right text-sm font-medium">
											{formatCurrency(row.total_cents, currencySymbol)}
										</span>
										<span class="w-12 shrink-0 text-right text-xs text-muted-foreground">
											{row.order_count} ord
										</span>
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>

					<!-- Sales Table -->
					<Card class="mt-4">
						<CardHeader>
							<CardTitle>Sales Breakdown</CardTitle>
						</CardHeader>
						<CardContent>
							<Table>
								<TableHeader>
									<TableRow>
										<TableHead>Period</TableHead>
										<TableHead class="text-right">Orders</TableHead>
										<TableHead class="text-right">Revenue</TableHead>
										<TableHead class="text-right">Tax</TableHead>
										<TableHead class="text-right">Avg Order</TableHead>
									</TableRow>
								</TableHeader>
								<TableBody>
									{#each salesData as row}
										<TableRow>
											<TableCell class="font-medium">{formatPeriodLabel(row.period)}</TableCell>
											<TableCell class="text-right">{row.order_count}</TableCell>
											<TableCell class="text-right font-medium">{formatCurrency(row.total_cents, currencySymbol)}</TableCell>
											<TableCell class="text-right text-muted-foreground">{formatCurrency(row.tax_cents, currencySymbol)}</TableCell>
											<TableCell class="text-right">{formatCurrency(row.avg_order_cents, currencySymbol)}</TableCell>
										</TableRow>
									{/each}
								</TableBody>
							</Table>
						</CardContent>
					</Card>
				{/if}
			</TabsContent>

			<!-- Products Tab -->
			<TabsContent value="products">
				<!-- Summary Cards -->
				<div class="mb-6 grid grid-cols-1 gap-4 sm:grid-cols-3">
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Product Revenue</CardTitle>
							<DollarSign class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{formatCurrency(productSummary().totalRevenue, currencySymbol)}</div>
						</CardContent>
					</Card>
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Units Sold</CardTitle>
							<Package class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{productSummary().totalUnits}</div>
						</CardContent>
					</Card>
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Unique Products</CardTitle>
							<BarChart3 class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{productSummary().uniqueProducts}</div>
						</CardContent>
					</Card>
				</div>

				<!-- Filters -->
				<div class="mb-4 flex flex-wrap items-center gap-3">
					<div class="flex items-center gap-2">
						<Calendar class="h-4 w-4 text-muted-foreground" />
						<Input
							type="date"
							bind:value={productDateFrom}
							class="w-36"
						/>
						<span class="text-sm text-muted-foreground">to</span>
						<Input
							type="date"
							bind:value={productDateTo}
							class="w-36"
						/>
					</div>
					<Button variant="outline" size="sm" onclick={loadProducts} disabled={productLoading}>
						{#if productLoading}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						{/if}
						Apply
					</Button>
				</div>

				{#if productData.length === 0}
					<Card>
						<CardContent class="flex flex-col items-center justify-center py-16">
							<Package class="mb-4 h-12 w-12 text-muted-foreground/50" />
							<p class="text-lg font-medium">No product data</p>
							<p class="text-sm text-muted-foreground">Adjust the date range or complete some orders.</p>
						</CardContent>
					</Card>
				{:else}
					<!-- Top Products Bar Chart -->
					<Card>
						<CardHeader>
							<CardTitle>Top Products by Revenue</CardTitle>
							<CardDescription>Product performance for the selected period</CardDescription>
						</CardHeader>
						<CardContent>
							<div class="space-y-2">
								{#each productData.slice(0, 10) as product}
									{@const pct = (product.total_revenue_cents / productMaxCents()) * 100}
									<div class="flex items-center gap-3">
										<span class="w-32 shrink-0 truncate text-right text-xs text-muted-foreground" title={product.product_name}>
											{product.product_name}
										</span>
										<div class="flex-1">
											<div
												class="h-7 rounded bg-primary/80 transition-all"
												style="width: {Math.max(pct, 1)}%"
											></div>
										</div>
										<span class="w-24 shrink-0 text-right text-sm font-medium">
											{formatCurrency(product.total_revenue_cents, currencySymbol)}
										</span>
										<span class="w-16 shrink-0 text-right text-xs text-muted-foreground">
											{product.total_quantity} sold
										</span>
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>

					<!-- Products Table -->
					<Card class="mt-4">
						<CardHeader>
							<CardTitle>All Products</CardTitle>
						</CardHeader>
						<CardContent>
							<Table>
								<TableHeader>
									<TableRow>
										<TableHead>Product</TableHead>
										<TableHead>SKU</TableHead>
										<TableHead class="text-right">Qty Sold</TableHead>
										<TableHead class="text-right">Revenue</TableHead>
										<TableHead class="text-right">Tax</TableHead>
										<TableHead class="text-right">Orders</TableHead>
									</TableRow>
								</TableHeader>
								<TableBody>
									{#each productData as product}
										<TableRow>
											<TableCell class="font-medium">{product.product_name}</TableCell>
											<TableCell class="font-mono text-sm text-muted-foreground">{product.product_sku ?? "—"}</TableCell>
											<TableCell class="text-right">{product.total_quantity}</TableCell>
											<TableCell class="text-right font-medium">{formatCurrency(product.total_revenue_cents, currencySymbol)}</TableCell>
											<TableCell class="text-right text-muted-foreground">{formatCurrency(product.total_tax_cents, currencySymbol)}</TableCell>
											<TableCell class="text-right">{product.order_count}</TableCell>
										</TableRow>
									{/each}
								</TableBody>
							</Table>
						</CardContent>
					</Card>
				{/if}
			</TabsContent>

			<!-- Inventory Tab -->
			<TabsContent value="inventory">
				<!-- Summary Cards -->
				<div class="mb-6 grid grid-cols-1 gap-4 sm:grid-cols-4">
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Active Products</CardTitle>
							<Package class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{inventoryStats().totalItems}</div>
						</CardContent>
					</Card>
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Total Stock</CardTitle>
							<ShoppingCart class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{inventoryStats().totalStock}</div>
						</CardContent>
					</Card>
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Low Stock</CardTitle>
							<AlertTriangle class="h-4 w-4 text-amber-500" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold text-amber-600">{inventoryStats().lowStock}</div>
						</CardContent>
					</Card>
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Inventory Value</CardTitle>
							<DollarSign class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{formatCurrency(inventoryStats().totalValue, currencySymbol)}</div>
							<p class="text-xs text-muted-foreground">At cost price</p>
						</CardContent>
					</Card>
				</div>

				{#if inventoryData.length === 0}
					<Card>
						<CardContent class="flex flex-col items-center justify-center py-16">
							<Package class="mb-4 h-12 w-12 text-muted-foreground/50" />
							<p class="text-lg font-medium">No inventory data</p>
						</CardContent>
					</Card>
				{:else}
					<Card>
						<CardHeader>
							<CardTitle>Inventory Status</CardTitle>
							<CardDescription>Current stock levels sorted by lowest stock first</CardDescription>
						</CardHeader>
						<CardContent>
							<Table>
								<TableHeader>
									<TableRow>
										<TableHead>Product</TableHead>
										<TableHead>SKU</TableHead>
										<TableHead>Category</TableHead>
										<TableHead class="text-right">Stock</TableHead>
										<TableHead class="text-right">Threshold</TableHead>
										<TableHead class="text-right">Cost</TableHead>
										<TableHead class="text-right">Price</TableHead>
										<TableHead>Status</TableHead>
									</TableRow>
								</TableHeader>
								<TableBody>
									{#each inventoryData as item}
										{@const isLow = item.stock_quantity <= item.low_stock_threshold && item.stock_quantity > 0}
										{@const isOut = item.stock_quantity === 0}
										<TableRow>
											<TableCell class="font-medium">{item.name}</TableCell>
											<TableCell class="font-mono text-sm text-muted-foreground">{item.sku ?? "—"}</TableCell>
											<TableCell class="text-sm">{item.category_name ?? "—"}</TableCell>
											<TableCell class="text-right font-medium {isOut ? 'text-destructive' : isLow ? 'text-amber-600' : ''}">
												{item.stock_quantity}
											</TableCell>
											<TableCell class="text-right text-muted-foreground">{item.low_stock_threshold}</TableCell>
											<TableCell class="text-right text-muted-foreground">{formatCurrency(item.cost_price_cents, currencySymbol)}</TableCell>
											<TableCell class="text-right">{formatCurrency(item.sale_price_cents, currencySymbol)}</TableCell>
											<TableCell>
												{#if isOut}
													<Badge variant="destructive">Out of Stock</Badge>
												{:else if isLow}
													<Badge variant="outline" class="border-amber-500 text-amber-600">Low Stock</Badge>
												{:else}
													<Badge variant="outline" class="border-green-500 text-green-600">In Stock</Badge>
												{/if}
											</TableCell>
										</TableRow>
									{/each}
								</TableBody>
							</Table>
						</CardContent>
					</Card>
				{/if}
			</TabsContent>
		</Tabs>
	{/if}
</div>
