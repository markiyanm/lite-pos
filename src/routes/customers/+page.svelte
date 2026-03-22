<script lang="ts">
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { Users, Plus, Search, Loader2, Pencil, Trash2, Mail, Phone, AlertTriangle, Cloud, CloudUpload, CloudAlert, CloudOff } from "lucide-svelte";
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
	import type { Customer, GatewaySyncStatus } from "$lib/types/index.js";
	import { settingsStore } from "$lib/stores/settings.svelte.js";

	let customers = $state<Customer[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let searchQuery = $state("");
	let deleteDialogOpen = $state(false);
	let deletingCustomer = $state<Customer | null>(null);
	let deleting = $state(false);

	const syncEnabled = $derived(
		settingsStore.getBoolean("gateway_customer_sync_enabled") &&
		!!settingsStore.get("sola_gateway_api_key")
	);

	const filteredCustomers = $derived.by(() => {
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
		error = null;
		try {
			customers = await getCustomers();
		} catch {
			toast.error("Failed to load customers");
			error = "Failed to load customers. Please check your connection and try again.";
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

	{#if error}
		<div class="mb-4 flex items-center gap-2 rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-destructive">
			<AlertTriangle class="h-5 w-5 shrink-0" />
			<p class="text-sm">{error}</p>
			<Button variant="outline" size="sm" class="ml-auto" onclick={() => { error = null; loadCustomers(); }}>
				Retry
			</Button>
		</div>
	{/if}

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
						{#if syncEnabled}
							<TableHead class="w-10 text-center">Sync</TableHead>
						{/if}
						<TableHead class="text-right">Actions</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{#each filteredCustomers as customer (customer.id)}
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
							{#if syncEnabled}
								<TableCell class="text-center">
									{#if customer.gateway_sync_status === "synced"}
										<Cloud class="inline-block h-4 w-4 text-emerald-500" />
									{:else if customer.gateway_sync_status === "pending"}
										<CloudUpload class="inline-block h-4 w-4 animate-pulse text-blue-500" />
									{:else if customer.gateway_sync_status === "error"}
										<CloudAlert class="inline-block h-4 w-4 text-orange-500" />
									{:else if customer.gateway_sync_status === "archived" || customer.gateway_sync_status === "orphaned"}
										<CloudOff class="inline-block h-4 w-4 text-muted-foreground" />
									{/if}
								</TableCell>
							{/if}
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

		{#if filteredCustomers.length === 0 && searchQuery}
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
