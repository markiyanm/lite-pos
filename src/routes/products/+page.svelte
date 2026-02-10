<script lang="ts">
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import {
		Package, Plus, Search, Loader2, AlertTriangle,
		Pencil, Trash2, Eye, EyeOff
	} from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { Card, CardContent } from "$lib/components/ui/card/index.js";
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
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle,
		DialogDescription,
		DialogFooter
	} from "$lib/components/ui/dialog/index.js";
	import { getProducts, deleteProduct, updateProduct } from "$lib/commands/products.js";
	import { getCategories } from "$lib/commands/categories.js";
	import { formatCurrency } from "$lib/utils.js";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import type { Product, Category } from "$lib/types/index.js";

	let products = $state<Product[]>([]);
	let categories = $state<Category[]>([]);
	let loading = $state(true);
	let searchQuery = $state("");
	let filterCategoryId = $state<string>("");
	let showInactive = $state(false);
	let deleteDialogOpen = $state(false);
	let deletingProduct = $state<Product | null>(null);
	let deleting = $state(false);

	const currencySymbol = $derived(settingsStore.get("currency_symbol") || "$");

	const categoryMap = $derived(
		new Map(categories.map((c) => [c.id, c]))
	);

	const filteredProducts = $derived(() => {
		let result = products;
		if (searchQuery.trim()) {
			const q = searchQuery.toLowerCase();
			result = result.filter(
				(p) =>
					p.name.toLowerCase().includes(q) ||
					(p.sku && p.sku.toLowerCase().includes(q)) ||
					(p.barcode && p.barcode.toLowerCase().includes(q))
			);
		}
		if (filterCategoryId) {
			const catId = parseInt(filterCategoryId, 10);
			result = result.filter((p) => p.category_id === catId);
		}
		return result;
	});

	onMount(async () => {
		loading = true;
		try {
			[products, categories] = await Promise.all([
				getProducts({ activeOnly: false }),
				getCategories()
			]);
		} catch {
			products = [];
			categories = [];
		} finally {
			loading = false;
		}
	});

	async function reload() {
		products = await getProducts({ activeOnly: false });
	}

	function isLowStock(product: Product): boolean {
		return product.is_active && product.stock_quantity <= product.low_stock_threshold;
	}

	function openDeleteDialog(product: Product) {
		deletingProduct = product;
		deleteDialogOpen = true;
	}

	async function handleDelete() {
		if (!deletingProduct) return;
		deleting = true;
		try {
			await deleteProduct(deletingProduct.id);
			deleteDialogOpen = false;
			deletingProduct = null;
			await reload();
		} catch {
			// silent
		} finally {
			deleting = false;
		}
	}

	async function toggleActive(product: Product) {
		await updateProduct(product.id, { is_active: !product.is_active });
		await reload();
	}

	const filterLabel = $derived(() => {
		if (!filterCategoryId) return "All Categories";
		return categoryMap.get(parseInt(filterCategoryId, 10))?.name ?? "All Categories";
	});
</script>

<div class="p-6">
	<div class="mb-6 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<Package class="h-6 w-6" />
			<h1 class="text-2xl font-semibold">Products</h1>
			<Badge variant="secondary">{products.length}</Badge>
		</div>
		<Button onclick={() => goto("/products/new")}>
			<Plus class="mr-2 h-4 w-4" />
			Add Product
		</Button>
	</div>

	<!-- Filters -->
	<div class="mb-4 flex items-center gap-3">
		<div class="relative flex-1 max-w-sm">
			<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
			<Input
				bind:value={searchQuery}
				placeholder="Search products..."
				class="pl-9"
			/>
		</div>
		<Select
			type="single"
			value={filterCategoryId}
			onValueChange={(v) => (filterCategoryId = v ?? "")}
		>
			<SelectTrigger class="w-48">{filterLabel()}</SelectTrigger>
			<SelectContent>
				<SelectItem value="">All Categories</SelectItem>
				{#each categories as cat}
					<SelectItem value={String(cat.id)}>{cat.name}</SelectItem>
				{/each}
			</SelectContent>
		</Select>
	</div>

	{#if loading}
		<div class="flex justify-center py-16">
			<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if products.length === 0}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-16">
				<Package class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<p class="mb-2 text-lg font-medium">No products yet</p>
				<p class="mb-4 text-sm text-muted-foreground">Add your first product to start selling.</p>
				<Button onclick={() => goto("/products/new")}>
					<Plus class="mr-2 h-4 w-4" />
					Add Product
				</Button>
			</CardContent>
		</Card>
	{:else}
		<Card>
			<Table>
				<TableHeader>
					<TableRow>
						<TableHead>Product</TableHead>
						<TableHead>SKU</TableHead>
						<TableHead>Category</TableHead>
						<TableHead class="text-right">Price</TableHead>
						<TableHead class="text-right">Stock</TableHead>
						<TableHead>Status</TableHead>
						<TableHead class="text-right">Actions</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{#each filteredProducts() as product (product.id)}
						<TableRow class={!product.is_active ? "opacity-50" : ""}>
							<TableCell>
								<div class="flex items-center gap-3">
									{#if product.image_path}
										<img
											src={product.image_path}
											alt={product.name}
											class="h-8 w-8 shrink-0 rounded object-cover"
										/>
									{:else}
										<div class="flex h-8 w-8 shrink-0 items-center justify-center rounded bg-muted">
											<Package class="h-4 w-4 text-muted-foreground" />
										</div>
									{/if}
									<div>
										<p class="font-medium">{product.name}</p>
										{#if product.description}
											<p class="text-sm text-muted-foreground line-clamp-1">{product.description}</p>
										{/if}
									</div>
								</div>
							</TableCell>
							<TableCell class="font-mono text-sm">
								{product.sku ?? "—"}
							</TableCell>
							<TableCell>
								{#if product.category_id}
									{@const cat = categoryMap.get(product.category_id)}
									{#if cat}
										<div class="flex items-center gap-2">
											<div class="h-3 w-3 rounded-full" style="background-color: {cat.color}"></div>
											<span class="text-sm">{cat.name}</span>
										</div>
									{:else}
										<span class="text-muted-foreground">—</span>
									{/if}
								{:else}
									<span class="text-muted-foreground">—</span>
								{/if}
							</TableCell>
							<TableCell class="text-right font-medium">
								{formatCurrency(product.sale_price_cents, currencySymbol)}
							</TableCell>
							<TableCell class="text-right">
								<div class="flex items-center justify-end gap-1">
									{#if isLowStock(product)}
										<AlertTriangle class="h-4 w-4 text-amber-500" />
									{/if}
									<span
										class:text-amber-600={isLowStock(product)}
										class:font-medium={isLowStock(product)}
									>
										{product.stock_quantity}
									</span>
								</div>
							</TableCell>
							<TableCell>
								{#if product.is_active}
									<Badge variant="outline" class="border-green-500 text-green-600">Active</Badge>
								{:else}
									<Badge variant="outline" class="border-muted-foreground text-muted-foreground">Inactive</Badge>
								{/if}
							</TableCell>
							<TableCell class="text-right">
								<div class="flex justify-end gap-1">
									<Button
										variant="ghost"
										size="sm"
										onclick={() => goto(`/products/${product.id}`)}
									>
										<Pencil class="h-4 w-4" />
									</Button>
									<Button
										variant="ghost"
										size="sm"
										onclick={() => toggleActive(product)}
									>
										{#if product.is_active}
											<EyeOff class="h-4 w-4 text-muted-foreground" />
										{:else}
											<Eye class="h-4 w-4 text-green-500" />
										{/if}
									</Button>
									<Button
										variant="ghost"
										size="sm"
										onclick={() => openDeleteDialog(product)}
									>
										<Trash2 class="h-4 w-4 text-destructive" />
									</Button>
								</div>
							</TableCell>
						</TableRow>
					{/each}
				</TableBody>
			</Table>
		</Card>

		{#if filteredProducts().length === 0 && (searchQuery || filterCategoryId)}
			<p class="py-8 text-center text-sm text-muted-foreground">
				No products match your filters.
			</p>
		{/if}
	{/if}
</div>

<!-- Delete Confirmation Dialog -->
<Dialog bind:open={deleteDialogOpen}>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Delete Product</DialogTitle>
			<DialogDescription>
				Are you sure you want to delete "{deletingProduct?.name}"? This action cannot be undone.
			</DialogDescription>
		</DialogHeader>
		<DialogFooter>
			<Button variant="outline" onclick={() => (deleteDialogOpen = false)}>Cancel</Button>
			<Button variant="destructive" onclick={handleDelete} disabled={deleting}>
				{#if deleting}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
				{/if}
				Delete
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
