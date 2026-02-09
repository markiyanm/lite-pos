<script lang="ts">
	import { onMount } from "svelte";
	import { Tags, Plus, Pencil, Trash2, Loader2, GripVertical } from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { Textarea } from "$lib/components/ui/textarea/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { Card, CardContent } from "$lib/components/ui/card/index.js";
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle,
		DialogDescription,
		DialogFooter
	} from "$lib/components/ui/dialog/index.js";
	import {
		getCategories,
		createCategory,
		updateCategory,
		deleteCategory
	} from "$lib/commands/categories.js";
	import { toast } from "svelte-sonner";
	import type { Category } from "$lib/types/index.js";

	let categories = $state<Category[]>([]);
	let loading = $state(true);
	let dialogOpen = $state(false);
	let deleteDialogOpen = $state(false);
	let saving = $state(false);
	let editingCategory = $state<Category | null>(null);
	let deletingCategory = $state<Category | null>(null);

	// Form state
	let formName = $state("");
	let formDescription = $state("");
	let formColor = $state("#6366f1");
	let formError = $state("");

	const presetColors = [
		"#ef4444", "#f97316", "#f59e0b", "#84cc16",
		"#22c55e", "#14b8a6", "#06b6d4", "#3b82f6",
		"#6366f1", "#8b5cf6", "#a855f7", "#ec4899"
	];

	onMount(loadCategories);

	async function loadCategories() {
		loading = true;
		try {
			categories = await getCategories();
		} catch {
			toast.error("Failed to load categories");
			categories = [];
		} finally {
			loading = false;
		}
	}

	function openNewDialog() {
		editingCategory = null;
		formName = "";
		formDescription = "";
		formColor = "#6366f1";
		formError = "";
		dialogOpen = true;
	}

	function openEditDialog(cat: Category) {
		editingCategory = cat;
		formName = cat.name;
		formDescription = cat.description ?? "";
		formColor = cat.color;
		formError = "";
		dialogOpen = true;
	}

	function openDeleteDialog(cat: Category) {
		deletingCategory = cat;
		deleteDialogOpen = true;
	}

	async function handleSave() {
		formError = "";
		if (!formName.trim()) {
			formError = "Name is required.";
			return;
		}

		saving = true;
		try {
			if (editingCategory) {
				await updateCategory(editingCategory.id, {
					name: formName.trim(),
					description: formDescription.trim() || null,
					color: formColor
				});
			} else {
				await createCategory({
					name: formName.trim(),
					description: formDescription.trim() || undefined,
					color: formColor,
					sort_order: categories.length
				});
			}
			dialogOpen = false;
			await loadCategories();
		} catch {
			formError = "Failed to save category.";
			toast.error("Failed to save category");
		} finally {
			saving = false;
		}
	}

	async function handleDelete() {
		if (!deletingCategory) return;
		saving = true;
		try {
			await deleteCategory(deletingCategory.id);
			deleteDialogOpen = false;
			deletingCategory = null;
			await loadCategories();
		} catch {
			toast.error("Failed to delete category");
		} finally {
			saving = false;
		}
	}
</script>

<div class="p-6">
	<div class="mb-6 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<Tags class="h-6 w-6" />
			<h1 class="text-2xl font-semibold">Categories</h1>
			<Badge variant="secondary">{categories.length}</Badge>
		</div>
		<Button onclick={openNewDialog}>
			<Plus class="mr-2 h-4 w-4" />
			Add Category
		</Button>
	</div>

	{#if loading}
		<div class="flex justify-center py-16">
			<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if categories.length === 0}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-16">
				<Tags class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<p class="mb-2 text-lg font-medium">No categories yet</p>
				<p class="mb-4 text-sm text-muted-foreground">Create your first category to organize products.</p>
				<Button onclick={openNewDialog}>
					<Plus class="mr-2 h-4 w-4" />
					Add Category
				</Button>
			</CardContent>
		</Card>
	{:else}
		<div class="grid gap-3">
			{#each categories as cat (cat.id)}
				<Card class="transition-colors hover:bg-muted/30">
					<CardContent class="flex items-center gap-4 py-4">
						<div class="flex items-center gap-3 text-muted-foreground">
							<GripVertical class="h-4 w-4" />
						</div>

						<!-- Color dot + Name -->
						<div
							class="h-4 w-4 shrink-0 rounded-full"
							style="background-color: {cat.color}"
						></div>
						<div class="flex-1">
							<p class="font-medium">{cat.name}</p>
							{#if cat.description}
								<p class="text-sm text-muted-foreground">{cat.description}</p>
							{/if}
						</div>

						<!-- Actions -->
						<div class="flex items-center gap-1">
							<Button variant="ghost" size="sm" onclick={() => openEditDialog(cat)}>
								<Pencil class="h-4 w-4" />
							</Button>
							<Button variant="ghost" size="sm" onclick={() => openDeleteDialog(cat)}>
								<Trash2 class="h-4 w-4 text-destructive" />
							</Button>
						</div>
					</CardContent>
				</Card>
			{/each}
		</div>
	{/if}
</div>

<!-- Create/Edit Dialog -->
<Dialog bind:open={dialogOpen}>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>{editingCategory ? "Edit Category" : "New Category"}</DialogTitle>
			<DialogDescription>
				{editingCategory ? "Update this category." : "Create a new product category."}
			</DialogDescription>
		</DialogHeader>

		<div class="space-y-4 py-4">
			{#if formError}
				<p class="text-sm text-destructive">{formError}</p>
			{/if}

			<div class="grid gap-2">
				<Label>Name</Label>
				<Input bind:value={formName} placeholder="Category name" />
			</div>

			<div class="grid gap-2">
				<Label>Description (optional)</Label>
				<Textarea bind:value={formDescription} placeholder="Brief description" />
			</div>

			<div class="grid gap-2">
				<Label>Color</Label>
				<div class="flex flex-wrap gap-2">
					{#each presetColors as color}
						<button
							type="button"
							class="h-8 w-8 rounded-full border-2 transition-transform hover:scale-110"
							class:border-foreground={formColor === color}
							class:border-transparent={formColor !== color}
							style="background-color: {color}"
							onclick={() => (formColor = color)}
							aria-label="Select color {color}"
						></button>
					{/each}
					<label class="relative flex h-8 w-8 cursor-pointer items-center justify-center rounded-full border-2 border-dashed border-muted-foreground/30">
						<input
							type="color"
							class="absolute inset-0 cursor-pointer opacity-0"
							bind:value={formColor}
						/>
						<span class="text-xs text-muted-foreground">+</span>
					</label>
				</div>
				<div class="flex items-center gap-2">
					<div class="h-5 w-5 rounded" style="background-color: {formColor}"></div>
					<span class="text-sm text-muted-foreground">{formColor}</span>
				</div>
			</div>
		</div>

		<DialogFooter>
			<Button variant="outline" onclick={() => (dialogOpen = false)}>Cancel</Button>
			<Button onclick={handleSave} disabled={saving}>
				{#if saving}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
				{/if}
				{editingCategory ? "Update" : "Create"}
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>

<!-- Delete Confirmation Dialog -->
<Dialog bind:open={deleteDialogOpen}>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Delete Category</DialogTitle>
			<DialogDescription>
				Are you sure you want to delete "{deletingCategory?.name}"? Products in this category will become uncategorized.
			</DialogDescription>
		</DialogHeader>
		<DialogFooter>
			<Button variant="outline" onclick={() => (deleteDialogOpen = false)}>Cancel</Button>
			<Button variant="destructive" onclick={handleDelete} disabled={saving}>
				{#if saving}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
				{/if}
				Delete
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
