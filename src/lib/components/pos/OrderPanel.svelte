<script lang="ts">
	import { Minus, Plus, Trash2, ShoppingCart, FileText, X, Users, ChevronsUpDown, Check, UserPlus, Loader2 } from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import {
		Popover,
		PopoverContent,
		PopoverTrigger
	} from "$lib/components/ui/popover/index.js";
	import {
		Command,
		CommandEmpty,
		CommandGroup,
		CommandInput,
		CommandItem,
		CommandList
	} from "$lib/components/ui/command/index.js";
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle,
		DialogFooter,
		DialogDescription
	} from "$lib/components/ui/dialog/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { formatCurrency } from "$lib/utils.js";
	import { orderStore } from "$lib/stores/order.svelte.js";
	import { getCustomers, getCustomer, createCustomer } from "$lib/commands/customers.js";
	import { toast } from "svelte-sonner";
	import type { Customer } from "$lib/types/index.js";

	interface Props {
		currencySymbol: string;
		onPay: () => void;
		onSaveDraft: () => void;
		onClear: () => void;
		busy?: boolean;
	}

	let { currencySymbol, onPay, onSaveDraft, onClear, busy = false }: Props = $props();

	let customerPopoverOpen = $state(false);
	let customers = $state<Customer[]>([]);
	let customerSearch = $state("");
	let customersLoaded = $state(false);

	async function loadCustomers() {
		if (customersLoaded) return;
		try {
			customers = await getCustomers();
			customersLoaded = true;
		} catch {
			customers = [];
		}
	}

	function customerName(c: Customer): string {
		return `${c.first_name} ${c.last_name}`.trim();
	}

	function selectCustomer(customer: Customer) {
		orderStore.setCustomer(customer);
		customerPopoverOpen = false;
	}

	function clearCustomer() {
		orderStore.setCustomer(null);
		customerPopoverOpen = false;
	}

	const filteredCustomers = $derived.by(() => {
		if (!customerSearch.trim()) return customers;
		const q = customerSearch.toLowerCase();
		return customers.filter(
			(c) =>
				c.first_name.toLowerCase().includes(q) ||
				c.last_name.toLowerCase().includes(q) ||
				(c.email && c.email.toLowerCase().includes(q)) ||
				(c.phone && c.phone.includes(q))
		);
	});

	let quickAddOpen = $state(false);
	let quickAddFirst = $state("");
	let quickAddLast = $state("");
	let quickAddSaving = $state(false);

	function openQuickAdd() {
		customerPopoverOpen = false;
		quickAddFirst = "";
		quickAddLast = "";
		quickAddOpen = true;
	}

	async function handleQuickAddCustomer() {
		const firstName = quickAddFirst.trim();
		const lastName = quickAddLast.trim();
		if (!firstName || !lastName) {
			toast.error("First name and last name are required");
			return;
		}

		quickAddSaving = true;
		try {
			const { lastInsertId } = await createCustomer({
				first_name: firstName,
				last_name: lastName,
				email: null,
				phone: null,
				billing_address_line1: null,
				billing_address_line2: null,
				billing_city: null,
				billing_state: null,
				billing_zip: null,
				shipping_address_line1: null,
				shipping_address_line2: null,
				shipping_city: null,
				shipping_state: null,
				shipping_zip: null,
				notes: null
			});
			const newCustomer = await getCustomer(lastInsertId);
			if (newCustomer) {
				orderStore.setCustomer(newCustomer);
				customersLoaded = false;
				toast.success(`Customer "${firstName} ${lastName}" created`);
			}
			quickAddOpen = false;
		} catch (err) {
			toast.error("Failed to create customer");
		} finally {
			quickAddSaving = false;
		}
	}
</script>

<div class="flex h-full flex-col border-l bg-card">
	<!-- Header -->
	<div class="flex items-center justify-between border-b px-4 py-3">
		<div class="flex items-center gap-2">
			<ShoppingCart class="h-4 w-4" />
			<h2 class="font-semibold">Current Order</h2>
		</div>
		{#if orderStore.itemCount > 0}
			<span class="text-sm text-muted-foreground">{orderStore.itemCount} item{orderStore.itemCount === 1 ? "" : "s"}</span>
		{/if}
	</div>

	<!-- Customer select -->
	<div class="border-b px-4 py-2">
		<Popover bind:open={customerPopoverOpen}>
			<PopoverTrigger
				class="flex w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background hover:bg-accent hover:text-accent-foreground"
				onclick={loadCustomers}
			>
				{#if orderStore.customer}
					<div class="flex items-center gap-2">
						<Users class="h-3.5 w-3.5 text-muted-foreground" />
						<span>{customerName(orderStore.customer)}</span>
					</div>
				{:else}
					<div class="flex items-center gap-2 text-muted-foreground">
						<Users class="h-3.5 w-3.5" />
						<span>Select customer (optional)</span>
					</div>
				{/if}
				<ChevronsUpDown class="ml-2 h-3.5 w-3.5 shrink-0 opacity-50" />
			</PopoverTrigger>
			<PopoverContent class="w-72 p-0" align="start">
				<Command>
					<CommandInput
						placeholder="Search customers..."
						value={customerSearch}
						oninput={(e: Event) => { customerSearch = (e.target as HTMLInputElement).value; }}
					/>
					<CommandList>
						<CommandEmpty>No customers found.</CommandEmpty>
						<CommandGroup>
							{#if orderStore.customer}
								<CommandItem value="__clear__" onSelect={clearCustomer}>
									<X class="mr-2 h-4 w-4" />
									Clear selection
								</CommandItem>
							{/if}
							{#each filteredCustomers as customer (customer.id)}
								<CommandItem
									value="{customer.first_name} {customer.last_name} {customer.email ?? ''}"
									onSelect={() => selectCustomer(customer)}
								>
									<Check class="mr-2 h-4 w-4 {orderStore.customer?.id === customer.id ? 'opacity-100' : 'opacity-0'}" />
									<div>
										<p class="text-sm">{customerName(customer)}</p>
										{#if customer.email}
											<p class="text-xs text-muted-foreground">{customer.email}</p>
										{/if}
									</div>
								</CommandItem>
							{/each}
							<CommandItem value="__new_customer__" onSelect={openQuickAdd}>
								<UserPlus class="mr-2 h-4 w-4" />
								New Customer
							</CommandItem>
						</CommandGroup>
					</CommandList>
				</Command>
			</PopoverContent>
		</Popover>
	</div>

	<!-- Line items -->
	{#if orderStore.items.length === 0}
		<div class="flex flex-1 flex-col items-center justify-center px-4 text-muted-foreground">
			<ShoppingCart class="mb-2 h-10 w-10 opacity-30" />
			<p class="text-sm">No items in order</p>
			<p class="text-xs">Click products to add them</p>
		</div>
	{:else}
		<ScrollArea class="flex-1">
			<div class="space-y-1 p-3">
				{#each orderStore.items as item (item.product.id)}
					{@const lineTotal = item.product.sale_price_cents * item.quantity}
					<div class="rounded-md border p-3">
						<div class="mb-2 flex items-start justify-between gap-2">
							<div class="flex-1 min-w-0">
								<p class="text-sm font-medium leading-tight">{item.product.name}</p>
								<p class="text-xs text-primary">
									{formatCurrency(item.product.sale_price_cents, currencySymbol)} x {item.quantity} = {formatCurrency(lineTotal, currencySymbol)}
								</p>
							</div>
							<Button
								variant="ghost"
								size="sm"
								class="h-7 w-7 shrink-0 p-0 text-destructive hover:text-destructive"
								onclick={() => orderStore.removeItem(item.product.id)}
							>
								<Trash2 class="h-3.5 w-3.5" />
							</Button>
						</div>
						<div class="flex items-center gap-2">
							<Button
								variant="outline"
								size="sm"
								class="h-7 w-7 p-0"
								onclick={() => orderStore.updateQuantity(item.product.id, item.quantity - 1)}
							>
								<Minus class="h-3 w-3" />
							</Button>
							<span class="min-w-[2rem] text-center text-sm font-medium">{item.quantity}</span>
							<Button
								variant="outline"
								size="sm"
								class="h-7 w-7 p-0"
								onclick={() => orderStore.updateQuantity(item.product.id, item.quantity + 1)}
							>
								<Plus class="h-3 w-3" />
							</Button>
						</div>
					</div>
				{/each}
			</div>
		</ScrollArea>

		<!-- Totals -->
		<div class="border-t px-4 py-3">
			<div class="space-y-1 text-sm">
				<div class="flex justify-between">
					<span class="text-muted-foreground">Subtotal</span>
					<span>{formatCurrency(orderStore.subtotalCents, currencySymbol)}</span>
				</div>
				<div class="flex justify-between">
					<span class="text-muted-foreground">Tax</span>
					<span>{formatCurrency(orderStore.taxTotalCents, currencySymbol)}</span>
				</div>
				<Separator />
				<div class="flex justify-between text-base font-semibold">
					<span>Total</span>
					<span>{formatCurrency(orderStore.totalCents, currencySymbol)}</span>
				</div>
			</div>
		</div>

		<!-- Action buttons -->
		<div class="border-t px-4 py-3 space-y-2">
			<Button class="w-full" size="lg" onclick={onPay} disabled={busy || orderStore.items.length === 0}>
				Pay {formatCurrency(orderStore.totalCents, currencySymbol)}
			</Button>
			<div class="flex gap-2">
				<Button variant="outline" class="flex-1" onclick={onSaveDraft} disabled={busy || orderStore.items.length === 0}>
					<FileText class="mr-2 h-4 w-4" />
					{busy ? "Saving..." : "Save Draft"}
				</Button>
				<Button variant="outline" class="flex-1" onclick={onClear}>
					<X class="mr-2 h-4 w-4" />
					Clear
				</Button>
			</div>
		</div>
	{/if}
</div>

<!-- Quick Add Customer Dialog -->
<Dialog bind:open={quickAddOpen}>
	<DialogContent class="sm:max-w-sm">
		<DialogHeader>
			<DialogTitle>New Customer</DialogTitle>
			<DialogDescription>Add a customer to this order.</DialogDescription>
		</DialogHeader>
		<div class="space-y-4 py-2">
			<div class="space-y-2">
				<Label for="quick-add-first">First Name</Label>
				<Input
					id="quick-add-first"
					placeholder="First name"
					bind:value={quickAddFirst}
					disabled={quickAddSaving}
				/>
			</div>
			<div class="space-y-2">
				<Label for="quick-add-last">Last Name</Label>
				<Input
					id="quick-add-last"
					placeholder="Last name"
					bind:value={quickAddLast}
					disabled={quickAddSaving}
				/>
			</div>
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => (quickAddOpen = false)} disabled={quickAddSaving}>
				Cancel
			</Button>
			<Button onclick={handleQuickAddCustomer} disabled={quickAddSaving}>
				{#if quickAddSaving}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
				{/if}
				Save
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
