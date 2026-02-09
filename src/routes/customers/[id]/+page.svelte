<script lang="ts">
	import { onMount } from "svelte";
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import { Users, ArrowLeft, Save, Loader2, ShoppingBag } from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { Textarea } from "$lib/components/ui/textarea/index.js";
	import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from "$lib/components/ui/table/index.js";
	import { getCustomer, createCustomer, updateCustomer } from "$lib/commands/customers.js";
	import { getOrders } from "$lib/commands/orders.js";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import { formatCurrency } from "$lib/utils.js";
	import type { Order } from "$lib/types/index.js";

	const paramId = $derived(page.params.id as string);
	const isNew = $derived(paramId === "new");
	const customerId = $derived(isNew ? null : parseInt(paramId, 10));

	let loading = $state(true);
	let saving = $state(false);
	let error = $state("");
	let orders = $state<Order[]>([]);

	// Form fields
	let firstName = $state("");
	let lastName = $state("");
	let email = $state("");
	let phone = $state("");
	let billingLine1 = $state("");
	let billingLine2 = $state("");
	let billingCity = $state("");
	let billingState = $state("");
	let billingZip = $state("");
	let shippingLine1 = $state("");
	let shippingLine2 = $state("");
	let shippingCity = $state("");
	let shippingState = $state("");
	let shippingZip = $state("");
	let notes = $state("");

	const currencySymbol = $derived(settingsStore.get("currency_symbol") || "$");

	const orderStats = $derived(() => {
		const completed = orders.filter((o) => o.status === "completed");
		const totalSpent = completed.reduce((sum, o) => sum + o.total_cents, 0);
		return {
			count: completed.length,
			totalSpent
		};
	});

	onMount(async () => {
		loading = true;
		try {
			if (!isNew && customerId) {
				const customer = await getCustomer(customerId);
				if (customer) {
					firstName = customer.first_name;
					lastName = customer.last_name;
					email = customer.email ?? "";
					phone = customer.phone ?? "";
					billingLine1 = customer.billing_address_line1 ?? "";
					billingLine2 = customer.billing_address_line2 ?? "";
					billingCity = customer.billing_city ?? "";
					billingState = customer.billing_state ?? "";
					billingZip = customer.billing_zip ?? "";
					shippingLine1 = customer.shipping_address_line1 ?? "";
					shippingLine2 = customer.shipping_address_line2 ?? "";
					shippingCity = customer.shipping_city ?? "";
					shippingState = customer.shipping_state ?? "";
					shippingZip = customer.shipping_zip ?? "";
					notes = customer.notes ?? "";

					// Load order history
					orders = await getOrders({ customerId });
				} else {
					error = "Customer not found.";
				}
			}
		} catch {
			error = "Failed to load customer data.";
		} finally {
			loading = false;
		}
	});

	async function handleSave() {
		error = "";

		if (!firstName.trim()) {
			error = "First name is required.";
			return;
		}
		if (!lastName.trim()) {
			error = "Last name is required.";
			return;
		}

		saving = true;
		try {
			const data = {
				first_name: firstName.trim(),
				last_name: lastName.trim(),
				email: email.trim() || null,
				phone: phone.trim() || null,
				billing_address_line1: billingLine1.trim() || null,
				billing_address_line2: billingLine2.trim() || null,
				billing_city: billingCity.trim() || null,
				billing_state: billingState.trim() || null,
				billing_zip: billingZip.trim() || null,
				shipping_address_line1: shippingLine1.trim() || null,
				shipping_address_line2: shippingLine2.trim() || null,
				shipping_city: shippingCity.trim() || null,
				shipping_state: shippingState.trim() || null,
				shipping_zip: shippingZip.trim() || null,
				notes: notes.trim() || null
			};

			if (isNew) {
				await createCustomer(data);
			} else if (customerId) {
				await updateCustomer(customerId, data);
			}

			goto("/customers");
		} catch {
			error = "Failed to save customer.";
		} finally {
			saving = false;
		}
	}

	function copyBillingToShipping() {
		shippingLine1 = billingLine1;
		shippingLine2 = billingLine2;
		shippingCity = billingCity;
		shippingState = billingState;
		shippingZip = billingZip;
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
			day: "numeric"
		});
	}
</script>

<div class="p-6">
	<!-- Header -->
	<div class="mb-6 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<Button variant="ghost" size="sm" onclick={() => goto("/customers")}>
				<ArrowLeft class="h-4 w-4" />
			</Button>
			<Users class="h-6 w-6" />
			<h1 class="text-2xl font-semibold">
				{isNew ? "New Customer" : "Edit Customer"}
			</h1>
		</div>
		<Button onclick={handleSave} disabled={saving || loading}>
			{#if saving}
				<Loader2 class="mr-2 h-4 w-4 animate-spin" />
				Saving...
			{:else}
				<Save class="mr-2 h-4 w-4" />
				{isNew ? "Create Customer" : "Save Changes"}
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
			<!-- Main form (2 cols) -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Contact Info -->
				<Card>
					<CardHeader>
						<CardTitle>Contact Information</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid grid-cols-2 gap-4">
							<div class="grid gap-2">
								<Label>First Name *</Label>
								<Input bind:value={firstName} placeholder="John" />
							</div>
							<div class="grid gap-2">
								<Label>Last Name *</Label>
								<Input bind:value={lastName} placeholder="Doe" />
							</div>
						</div>
						<div class="grid grid-cols-2 gap-4">
							<div class="grid gap-2">
								<Label>Email</Label>
								<Input type="email" bind:value={email} placeholder="john@example.com" />
							</div>
							<div class="grid gap-2">
								<Label>Phone</Label>
								<Input type="tel" bind:value={phone} placeholder="(555) 123-4567" />
							</div>
						</div>
					</CardContent>
				</Card>

				<!-- Billing Address -->
				<Card>
					<CardHeader>
						<CardTitle>Billing Address</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid gap-2">
							<Label>Address Line 1</Label>
							<Input bind:value={billingLine1} placeholder="123 Main St" />
						</div>
						<div class="grid gap-2">
							<Label>Address Line 2</Label>
							<Input bind:value={billingLine2} placeholder="Apt 4B" />
						</div>
						<div class="grid grid-cols-3 gap-4">
							<div class="grid gap-2">
								<Label>City</Label>
								<Input bind:value={billingCity} placeholder="New York" />
							</div>
							<div class="grid gap-2">
								<Label>State</Label>
								<Input bind:value={billingState} placeholder="NY" />
							</div>
							<div class="grid gap-2">
								<Label>ZIP Code</Label>
								<Input bind:value={billingZip} placeholder="10001" />
							</div>
						</div>
					</CardContent>
				</Card>

				<!-- Shipping Address -->
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle>Shipping Address</CardTitle>
							<Button variant="outline" size="sm" onclick={copyBillingToShipping}>
								Copy from Billing
							</Button>
						</div>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid gap-2">
							<Label>Address Line 1</Label>
							<Input bind:value={shippingLine1} placeholder="123 Main St" />
						</div>
						<div class="grid gap-2">
							<Label>Address Line 2</Label>
							<Input bind:value={shippingLine2} placeholder="Apt 4B" />
						</div>
						<div class="grid grid-cols-3 gap-4">
							<div class="grid gap-2">
								<Label>City</Label>
								<Input bind:value={shippingCity} placeholder="New York" />
							</div>
							<div class="grid gap-2">
								<Label>State</Label>
								<Input bind:value={shippingState} placeholder="NY" />
							</div>
							<div class="grid gap-2">
								<Label>ZIP Code</Label>
								<Input bind:value={shippingZip} placeholder="10001" />
							</div>
						</div>
					</CardContent>
				</Card>
			</div>

			<!-- Sidebar (1 col) -->
			<div class="space-y-6">
				<!-- Notes -->
				<Card>
					<CardHeader>
						<CardTitle>Notes</CardTitle>
					</CardHeader>
					<CardContent>
						<Textarea bind:value={notes} placeholder="Internal notes about this customer..." rows={4} />
					</CardContent>
				</Card>

				<!-- Order summary (existing customers only) -->
				{#if !isNew}
					<Card>
						<CardHeader>
							<CardTitle>Summary</CardTitle>
						</CardHeader>
						<CardContent class="space-y-3">
							<div class="flex justify-between text-sm">
								<span class="text-muted-foreground">Total Orders</span>
								<span class="font-medium">{orderStats().count}</span>
							</div>
							<Separator />
							<div class="flex justify-between text-sm">
								<span class="text-muted-foreground">Total Spent</span>
								<span class="font-medium">{formatCurrency(orderStats().totalSpent, currencySymbol)}</span>
							</div>
						</CardContent>
					</Card>
				{/if}
			</div>
		</div>

		<!-- Order History (existing customers only) -->
		{#if !isNew}
			<div class="mt-6">
				<Card>
					<CardHeader>
						<div class="flex items-center gap-2">
							<ShoppingBag class="h-5 w-5" />
							<CardTitle>Order History</CardTitle>
							<Badge variant="secondary">{orders.length}</Badge>
						</div>
					</CardHeader>
					<CardContent>
						{#if orders.length === 0}
							<p class="py-8 text-center text-sm text-muted-foreground">
								No orders yet for this customer.
							</p>
						{:else}
							<Table>
								<TableHeader>
									<TableRow>
										<TableHead>Order #</TableHead>
										<TableHead>Date</TableHead>
										<TableHead>Status</TableHead>
										<TableHead class="text-right">Total</TableHead>
									</TableRow>
								</TableHeader>
								<TableBody>
									{#each orders as order (order.id)}
										<TableRow
											class="cursor-pointer"
											onclick={() => goto(`/orders/${order.id}`)}
										>
											<TableCell class="font-mono text-sm">{order.order_number}</TableCell>
											<TableCell>{formatDate(order.created_at)}</TableCell>
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
						{/if}
					</CardContent>
				</Card>
			</div>
		{/if}
	{/if}
</div>
