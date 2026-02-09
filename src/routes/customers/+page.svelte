<script lang="ts">
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { Users, Plus, Search, Loader2, Pencil, Trash2, Mail, Phone } from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { Card, CardContent } from "$lib/components/ui/card/index.js";
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
	import { getCustomers, deleteCustomer } from "$lib/commands/customers.js";
	import { toast } from "svelte-sonner";
	import type { Customer } from "$lib/types/index.js";

	let customers = $state<Customer[]>([]);
	let loading = $state(true);
	let searchQuery = $state("");
	let deleteDialogOpen = $state(false);
	let deletingCustomer = $state<Customer | null>(null);
	let deleting = $state(false);

	const filteredCustomers = $derived(() => {
		if (!searchQuery.trim()) return customers;
		const q = searchQuery.toLowerCase();
		return customers.filter(
			(c) =>
				c.first_name.toLowerCase().includes(q) ||
				c.last_name.toLowerCase().includes(q) ||
				(c.email && c.email.toLowerCase().includes(q)) ||
				(c.phone && c.phone.includes(q))
		);
	});

	onMount(loadCustomers);

	async function loadCustomers() {
		loading = true;
		try {
			customers = await getCustomers();
		} catch {
			toast.error("Failed to load customers");
			customers = [];
		} finally {
			loading = false;
		}
	}

	function customerName(c: Customer): string {
		return `${c.first_name} ${c.last_name}`.trim();
	}

	function openDeleteDialog(customer: Customer) {
		deletingCustomer = customer;
		deleteDialogOpen = true;
	}

	async function handleDelete() {
		if (!deletingCustomer) return;
		deleting = true;
		try {
			await deleteCustomer(deletingCustomer.id);
			deleteDialogOpen = false;
			deletingCustomer = null;
			await loadCustomers();
		} catch {
			toast.error("Failed to delete customer");
		} finally {
			deleting = false;
		}
	}
</script>

<div class="p-6">
	<div class="mb-6 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<Users class="h-6 w-6" />
			<h1 class="text-2xl font-semibold">Customers</h1>
			<Badge variant="secondary">{customers.length}</Badge>
		</div>
		<Button onclick={() => goto("/customers/new")}>
			<Plus class="mr-2 h-4 w-4" />
			Add Customer
		</Button>
	</div>

	<!-- Search -->
	<div class="mb-4 flex items-center gap-3">
		<div class="relative flex-1 max-w-sm">
			<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
			<Input
				bind:value={searchQuery}
				placeholder="Search customers..."
				class="pl-9"
			/>
		</div>
	</div>

	{#if loading}
		<div class="flex justify-center py-16">
			<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if customers.length === 0}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-16">
				<Users class="mb-4 h-12 w-12 text-muted-foreground/50" />
				<p class="mb-2 text-lg font-medium">No customers yet</p>
				<p class="mb-4 text-sm text-muted-foreground">Add your first customer to track purchases.</p>
				<Button onclick={() => goto("/customers/new")}>
					<Plus class="mr-2 h-4 w-4" />
					Add Customer
				</Button>
			</CardContent>
		</Card>
	{:else}
		<Card>
			<Table>
				<TableHeader>
					<TableRow>
						<TableHead>Name</TableHead>
						<TableHead>Email</TableHead>
						<TableHead>Phone</TableHead>
						<TableHead>City</TableHead>
						<TableHead class="text-right">Actions</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{#each filteredCustomers() as customer (customer.id)}
						<TableRow>
							<TableCell>
								<button
									type="button"
									class="text-left font-medium hover:underline"
									onclick={() => goto(`/customers/${customer.id}`)}
								>
									{customerName(customer)}
								</button>
							</TableCell>
							<TableCell>
								{#if customer.email}
									<div class="flex items-center gap-1.5 text-sm">
										<Mail class="h-3.5 w-3.5 text-muted-foreground" />
										{customer.email}
									</div>
								{:else}
									<span class="text-muted-foreground">—</span>
								{/if}
							</TableCell>
							<TableCell>
								{#if customer.phone}
									<div class="flex items-center gap-1.5 text-sm">
										<Phone class="h-3.5 w-3.5 text-muted-foreground" />
										{customer.phone}
									</div>
								{:else}
									<span class="text-muted-foreground">—</span>
								{/if}
							</TableCell>
							<TableCell>
								{customer.billing_city ?? "—"}
							</TableCell>
							<TableCell class="text-right">
								<div class="flex justify-end gap-1">
									<Button
										variant="ghost"
										size="sm"
										onclick={() => goto(`/customers/${customer.id}`)}
									>
										<Pencil class="h-4 w-4" />
									</Button>
									<Button
										variant="ghost"
										size="sm"
										onclick={() => openDeleteDialog(customer)}
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

		{#if filteredCustomers().length === 0 && searchQuery}
			<p class="py-8 text-center text-sm text-muted-foreground">
				No customers match your search.
			</p>
		{/if}
	{/if}
</div>

<!-- Delete Confirmation Dialog -->
<Dialog bind:open={deleteDialogOpen}>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Delete Customer</DialogTitle>
			<DialogDescription>
				Are you sure you want to delete "{deletingCustomer ? `${deletingCustomer.first_name} ${deletingCustomer.last_name}` : ''}"? This action cannot be undone.
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
