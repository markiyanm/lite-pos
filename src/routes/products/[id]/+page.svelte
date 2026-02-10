<script lang="ts">
	import { onMount } from "svelte";
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import { Package, ArrowLeft, Save, Loader2, ImagePlus, X } from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { Textarea } from "$lib/components/ui/textarea/index.js";
	import { Switch } from "$lib/components/ui/switch/index.js";
	import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "$lib/components/ui/card/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import {
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger
	} from "$lib/components/ui/select/index.js";
	import { getProduct, createProduct, updateProduct } from "$lib/commands/products.js";
	import { getCategories } from "$lib/commands/categories.js";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import type { Category } from "$lib/types/index.js";
	import { readFileAsDataUrl } from "$lib/utils.js";

	const paramId = $derived(page.params.id as string);
	const isNew = $derived(paramId === "new");
	const productId = $derived(isNew ? null : parseInt(paramId, 10));

	let categories = $state<Category[]>([]);
	let loading = $state(true);
	let saving = $state(false);
	let error = $state("");

	// Form fields
	let name = $state("");
	let description = $state("");
	let sku = $state("");
	let barcode = $state("");
	let categoryId = $state<string>("");
	let costPrice = $state("");
	let salePrice = $state("");
	let taxRate = $state("");
	let stockQuantity = $state("0");
	let lowStockThreshold = $state("5");
	let isActive = $state(true);
	let imagePath = $state<string | null>(null);
	let imageFileInput = $state<HTMLInputElement>(undefined!);

	const currencySymbol = $derived(settingsStore.get("currency_symbol") || "$");

	const categoryLabel = $derived(() => {
		if (!categoryId) return "No Category";
		const cat = categories.find((c) => String(c.id) === categoryId);
		return cat?.name ?? "No Category";
	});

	onMount(async () => {
		loading = true;
		try {
			categories = await getCategories();

			if (!isNew && productId) {
				const product = await getProduct(productId);
				if (product) {
					name = product.name;
					description = product.description ?? "";
					sku = product.sku ?? "";
					barcode = product.barcode ?? "";
					categoryId = product.category_id ? String(product.category_id) : "";
					costPrice = (product.cost_price_cents / 100).toFixed(2);
					salePrice = (product.sale_price_cents / 100).toFixed(2);
					taxRate = (product.tax_rate_bps / 100).toFixed(2);
					stockQuantity = String(product.stock_quantity);
					lowStockThreshold = String(product.low_stock_threshold);
					isActive = Boolean(product.is_active);
					imagePath = product.image_path ?? null;
				} else {
					error = "Product not found.";
				}
			} else {
				// Default tax rate from settings
				const defaultBps = settingsStore.getNumber("default_tax_rate_bps");
				taxRate = (defaultBps / 100).toFixed(2);
				lowStockThreshold = settingsStore.get("low_stock_default_threshold") || "5";
			}
		} catch {
			error = "Failed to load product data.";
		} finally {
			loading = false;
		}
	});

	function dollarsToCenter(dollars: string): number {
		const val = parseFloat(dollars);
		if (isNaN(val)) return 0;
		return Math.round(val * 100);
	}

	function percentToBps(percent: string): number {
		const val = parseFloat(percent);
		if (isNaN(val)) return 0;
		return Math.round(val * 100);
	}

	async function handleImageSelect(e: Event) {
		const input = e.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		try {
			imagePath = await readFileAsDataUrl(file, 400);
		} catch {
			// silent
		}
		input.value = "";
	}

	async function handleSave() {
		error = "";

		if (!name.trim()) {
			error = "Product name is required.";
			return;
		}
		if (!salePrice || parseFloat(salePrice) < 0) {
			error = "Sale price is required and must be non-negative.";
			return;
		}

		saving = true;
		try {
			const data = {
				name: name.trim(),
				description: description.trim() || null,
				sku: sku.trim() || null,
				barcode: barcode.trim() || null,
				category_id: categoryId ? parseInt(categoryId, 10) : null,
				cost_price_cents: dollarsToCenter(costPrice),
				sale_price_cents: dollarsToCenter(salePrice),
				tax_rate_bps: percentToBps(taxRate),
				stock_quantity: parseInt(stockQuantity, 10) || 0,
				low_stock_threshold: parseInt(lowStockThreshold, 10) || 5,
				image_path: imagePath,
				is_active: isActive,
				sort_order: 0
			};

			if (isNew) {
				await createProduct(data);
			} else if (productId) {
				await updateProduct(productId, data);
			}

			goto("/products");
		} catch {
			error = "Failed to save product.";
		} finally {
			saving = false;
		}
	}
</script>

<div class="p-6">
	<!-- Header -->
	<div class="mb-6 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<Button variant="ghost" size="sm" onclick={() => goto("/products")}>
				<ArrowLeft class="h-4 w-4" />
			</Button>
			<Package class="h-6 w-6" />
			<h1 class="text-2xl font-semibold">
				{isNew ? "New Product" : "Edit Product"}
			</h1>
		</div>
		<Button onclick={handleSave} disabled={saving || loading}>
			{#if saving}
				<Loader2 class="mr-2 h-4 w-4 animate-spin" />
				Saving...
			{:else}
				<Save class="mr-2 h-4 w-4" />
				{isNew ? "Create Product" : "Save Changes"}
			{/if}
		</Button>
	</div>

	{#if loading}
		<div class="flex justify-center py-16">
			<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else}
		{#if error}
			<div class="mb-4 rounded-md bg-destructive/10 p-3 text-sm text-destructive">
				{error}
			</div>
		{/if}

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main info (2 cols) -->
			<div class="space-y-6 lg:col-span-2">
				<Card>
					<CardHeader>
						<CardTitle>Basic Information</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid gap-2">
							<Label>Product Name *</Label>
							<Input bind:value={name} placeholder="e.g. Coffee Mug" />
						</div>
						<div class="grid gap-2">
							<Label>Description</Label>
							<Textarea bind:value={description} placeholder="Product description..." />
						</div>
						<div class="grid grid-cols-2 gap-4">
							<div class="grid gap-2">
								<Label>SKU</Label>
								<Input bind:value={sku} placeholder="e.g. MUG-001" class="font-mono" />
							</div>
							<div class="grid gap-2">
								<Label>Barcode</Label>
								<Input bind:value={barcode} placeholder="e.g. 012345678901" class="font-mono" />
							</div>
						</div>
						<div class="grid gap-2">
							<Label>Category</Label>
							<Select
								type="single"
								value={categoryId}
								onValueChange={(v) => (categoryId = v ?? "")}
							>
								<SelectTrigger>{categoryLabel()}</SelectTrigger>
								<SelectContent>
									<SelectItem value="">No Category</SelectItem>
									{#each categories as cat}
										<SelectItem value={String(cat.id)}>
											{cat.name}
										</SelectItem>
									{/each}
								</SelectContent>
							</Select>
						</div>
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<CardTitle>Pricing</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid grid-cols-2 gap-4">
							<div class="grid gap-2">
								<Label>Cost Price ({currencySymbol})</Label>
								<Input
									type="number"
									bind:value={costPrice}
									placeholder="0.00"
									step="0.01"
									min="0"
								/>
								<p class="text-xs text-muted-foreground">What you paid for it</p>
							</div>
							<div class="grid gap-2">
								<Label>Sale Price ({currencySymbol}) *</Label>
								<Input
									type="number"
									bind:value={salePrice}
									placeholder="0.00"
									step="0.01"
									min="0"
								/>
								<p class="text-xs text-muted-foreground">What customers pay</p>
							</div>
						</div>
						<div class="grid gap-2 max-w-xs">
							<Label>Tax Rate (%)</Label>
							<Input
								type="number"
								bind:value={taxRate}
								placeholder="0.00"
								step="0.01"
								min="0"
							/>
							<p class="text-xs text-muted-foreground">
								Applied at checkout. Default from settings: {(settingsStore.getNumber("default_tax_rate_bps") / 100).toFixed(2)}%
							</p>
						</div>
					</CardContent>
				</Card>
			</div>

			<!-- Sidebar (1 col) -->
			<div class="space-y-6">
				<Card>
					<CardHeader>
						<CardTitle>Product Image</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="flex flex-col items-center gap-3">
							{#if imagePath}
								<div class="relative w-full">
									<img
										src={imagePath}
										alt="Product"
										class="w-full rounded-md border object-cover"
									/>
									<Button
										variant="destructive"
										size="icon"
										class="absolute right-1 top-1 h-6 w-6"
										onclick={() => (imagePath = null)}
									>
										<X class="h-3 w-3" />
									</Button>
								</div>
							{:else}
								<div class="flex h-32 w-full items-center justify-center rounded-md border border-dashed bg-muted/50">
									<ImagePlus class="h-8 w-8 text-muted-foreground/50" />
								</div>
							{/if}
							<input
								bind:this={imageFileInput}
								type="file"
								accept="image/*"
								class="hidden"
								onchange={handleImageSelect}
							/>
							<Button
								variant="outline"
								size="sm"
								class="w-full"
								onclick={() => imageFileInput.click()}
							>
								<ImagePlus class="mr-2 h-4 w-4" />
								{imagePath ? "Change Image" : "Upload Image"}
							</Button>
						</div>
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<CardTitle>Status</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="flex items-center justify-between">
							<div>
								<p class="font-medium">Active</p>
								<p class="text-sm text-muted-foreground">
									Visible in POS and product listings
								</p>
							</div>
							<Switch
								checked={isActive}
								onCheckedChange={(v) => (isActive = v)}
							/>
						</div>
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<CardTitle>Inventory</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid gap-2">
							<Label>Stock Quantity</Label>
							<Input
								type="number"
								bind:value={stockQuantity}
								min="0"
							/>
						</div>
						<Separator />
						<div class="grid gap-2">
							<Label>Low Stock Threshold</Label>
							<Input
								type="number"
								bind:value={lowStockThreshold}
								min="0"
							/>
							<p class="text-xs text-muted-foreground">
								Alert when stock falls to this level
							</p>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	{/if}
</div>
