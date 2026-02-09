<script lang="ts">
	import { Search, Loader2, Package } from "lucide-svelte";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { formatCurrency } from "$lib/utils.js";
	import type { Product, Category } from "$lib/types/index.js";

	interface Props {
		products: Product[];
		categories: Category[];
		loading: boolean;
		searchQuery: string;
		selectedCategoryId: number | null;
		currencySymbol: string;
		onSearchChange: (query: string) => void;
		onCategoryChange: (categoryId: number | null) => void;
		onProductClick: (product: Product) => void;
	}

	let {
		products,
		categories,
		loading,
		searchQuery,
		selectedCategoryId,
		currencySymbol,
		onSearchChange,
		onCategoryChange,
		onProductClick
	}: Props = $props();

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
		if (selectedCategoryId !== null) {
			result = result.filter((p) => p.category_id === selectedCategoryId);
		}
		return result;
	});
</script>

<div class="flex h-full flex-col">
	<!-- Search bar -->
	<div class="mb-3 flex items-center gap-3">
		<div class="relative flex-1">
			<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
			<Input
				value={searchQuery}
				placeholder="Search products..."
				class="pl-9"
				oninput={(e) => onSearchChange(e.currentTarget.value)}
			/>
		</div>
	</div>

	<!-- Category tabs -->
	<div class="mb-3 flex gap-2 overflow-x-auto pb-1">
		<Button
			variant={selectedCategoryId === null ? "default" : "outline"}
			size="sm"
			onclick={() => onCategoryChange(null)}
		>
			Show All
		</Button>
		{#each categories as cat}
			<Button
				variant={selectedCategoryId === cat.id ? "default" : "outline"}
				size="sm"
				onclick={() => onCategoryChange(cat.id)}
				class="shrink-0"
			>
				<div class="mr-1.5 h-2.5 w-2.5 rounded-full" style="background-color: {cat.color}"></div>
				{cat.name}
			</Button>
		{/each}
	</div>

	<!-- Product grid -->
	{#if loading}
		<div class="flex flex-1 items-center justify-center">
			<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if filteredProducts().length === 0}
		<div class="flex flex-1 flex-col items-center justify-center text-muted-foreground">
			<Package class="mb-2 h-10 w-10 opacity-50" />
			<p class="text-sm">No products found</p>
		</div>
	{:else}
		<ScrollArea class="flex-1">
			<div class="grid grid-cols-2 gap-3 pr-3 xl:grid-cols-3 2xl:grid-cols-4">
				{#each filteredProducts() as product (product.id)}
					<button
						type="button"
						class="flex flex-col rounded-lg border bg-card p-3 text-left transition-colors hover:bg-accent/50 active:bg-accent"
						onclick={() => onProductClick(product)}
					>
						<p class="mb-1 text-sm font-medium leading-tight line-clamp-2">{product.name}</p>
						<p class="mt-auto text-sm font-semibold text-primary">
							{formatCurrency(product.sale_price_cents, currencySymbol)}
						</p>
						{#if product.stock_quantity <= product.low_stock_threshold}
							<p class="mt-1 text-xs text-amber-600">
								{product.stock_quantity} left
							</p>
						{/if}
					</button>
				{/each}
			</div>
		</ScrollArea>
	{/if}
</div>
