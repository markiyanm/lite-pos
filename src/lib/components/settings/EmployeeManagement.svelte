<script lang="ts">
	import { onMount } from "svelte";
	import { Plus, Pencil, Loader2, UserCheck, UserX } from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "$lib/components/ui/card/index.js";
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle,
		DialogDescription,
		DialogFooter
	} from "$lib/components/ui/dialog/index.js";
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
	import { getUsers, createUser, updateUser } from "$lib/commands/auth.js";
	import type { User } from "$lib/types/index.js";

	let users = $state<User[]>([]);
	let loadingUsers = $state(true);
	let dialogOpen = $state(false);
	let saving = $state(false);
	let editingUser = $state<User | null>(null);

	// Form state
	let formName = $state("");
	let formEmail = $state("");
	let formPin = $state("");
	let formRole = $state<"admin" | "cashier">("cashier");
	let formError = $state("");

	onMount(loadUsers);

	async function loadUsers() {
		loadingUsers = true;
		try {
			users = await getUsers();
		} catch {
			users = [];
		} finally {
			loadingUsers = false;
		}
	}

	function openNewDialog() {
		editingUser = null;
		formName = "";
		formEmail = "";
		formPin = "";
		formRole = "cashier";
		formError = "";
		dialogOpen = true;
	}

	function openEditDialog(user: User) {
		editingUser = user;
		formName = user.name;
		formEmail = user.email ?? "";
		formPin = "";
		formRole = user.role;
		formError = "";
		dialogOpen = true;
	}

	async function handleSave() {
		formError = "";

		if (!formName.trim()) {
			formError = "Name is required.";
			return;
		}
		if (!editingUser && !formPin.trim()) {
			formError = "PIN is required for new employees.";
			return;
		}
		if (formPin && (formPin.length < 4 || formPin.length > 6)) {
			formError = "PIN must be 4-6 digits.";
			return;
		}
		if (formPin && !/^\d+$/.test(formPin)) {
			formError = "PIN must contain only digits.";
			return;
		}

		saving = true;
		try {
			if (editingUser) {
				const updates: { name?: string; email?: string; pin?: string; role?: string } = {
					name: formName.trim(),
					email: formEmail.trim() || undefined,
					role: formRole
				};
				if (formPin) {
					updates.pin = formPin;
				}
				await updateUser(editingUser.id, updates);
			} else {
				await createUser({
					name: formName.trim(),
					email: formEmail.trim() || undefined,
					pin: formPin,
					role: formRole
				});
			}
			dialogOpen = false;
			await loadUsers();
		} catch {
			formError = "Failed to save employee.";
		} finally {
			saving = false;
		}
	}

	async function toggleActive(user: User) {
		await updateUser(user.id, { is_active: !user.is_active });
		await loadUsers();
	}
</script>

<Card>
	<CardHeader>
		<div class="flex items-center justify-between">
			<div>
				<CardTitle>Employee Management</CardTitle>
				<CardDescription>Manage employee accounts and PINs. Only admins can access this section.</CardDescription>
			</div>
			<Button onclick={openNewDialog}>
				<Plus class="mr-2 h-4 w-4" />
				Add Employee
			</Button>
		</div>
	</CardHeader>
	<CardContent>
		{#if loadingUsers}
			<div class="flex justify-center py-8">
				<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
			</div>
		{:else if users.length === 0}
			<p class="py-8 text-center text-muted-foreground">No employees found.</p>
		{:else}
			<Table>
				<TableHeader>
					<TableRow>
						<TableHead>Name</TableHead>
						<TableHead>Email</TableHead>
						<TableHead>Role</TableHead>
						<TableHead>Status</TableHead>
						<TableHead class="text-right">Actions</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{#each users as user}
						<TableRow>
							<TableCell class="font-medium">{user.name}</TableCell>
							<TableCell>{user.email ?? "—"}</TableCell>
							<TableCell>
								<Badge variant={user.role === "admin" ? "default" : "secondary"}>
									{user.role}
								</Badge>
							</TableCell>
							<TableCell>
								{#if user.is_active}
									<Badge variant="outline" class="border-green-500 text-green-600">Active</Badge>
								{:else}
									<Badge variant="outline" class="border-red-500 text-red-600">Inactive</Badge>
								{/if}
							</TableCell>
							<TableCell class="text-right">
								<div class="flex justify-end gap-1">
									<Button variant="ghost" size="sm" onclick={() => openEditDialog(user)}>
										<Pencil class="h-4 w-4" />
									</Button>
									<Button
										variant="ghost"
										size="sm"
										onclick={() => toggleActive(user)}
									>
										{#if user.is_active}
											<UserX class="h-4 w-4 text-red-500" />
										{:else}
											<UserCheck class="h-4 w-4 text-green-500" />
										{/if}
									</Button>
								</div>
							</TableCell>
						</TableRow>
					{/each}
				</TableBody>
			</Table>
		{/if}
	</CardContent>
</Card>

<!-- Add/Edit Employee Dialog -->
<Dialog bind:open={dialogOpen}>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>{editingUser ? "Edit Employee" : "Add Employee"}</DialogTitle>
			<DialogDescription>
				{editingUser ? "Update employee details. Leave PIN empty to keep the current one." : "Create a new employee account."}
			</DialogDescription>
		</DialogHeader>

		<div class="space-y-4 py-4">
			{#if formError}
				<p class="text-sm text-destructive">{formError}</p>
			{/if}

			<div class="grid gap-2">
				<Label>Name</Label>
				<Input
					bind:value={formName}
					placeholder="Employee name"
				/>
			</div>

			<div class="grid gap-2">
				<Label>Email (optional)</Label>
				<Input
					type="email"
					bind:value={formEmail}
					placeholder="employee@example.com"
				/>
			</div>

			<div class="grid gap-2">
				<Label>{editingUser ? "New PIN (leave empty to keep current)" : "PIN"}</Label>
				<Input
					type="password"
					bind:value={formPin}
					placeholder="4-6 digit PIN"
					maxlength={6}
				/>
			</div>

			<div class="grid gap-2">
				<Label>Role</Label>
				<Select
					type="single"
					value={formRole}
					onValueChange={(v) => { if (v) formRole = v as "admin" | "cashier"; }}
				>
					<SelectTrigger>{formRole === "admin" ? "Admin" : "Cashier"}</SelectTrigger>
					<SelectContent>
						<SelectItem value="cashier">Cashier</SelectItem>
						<SelectItem value="admin">Admin</SelectItem>
					</SelectContent>
				</Select>
			</div>
		</div>

		<DialogFooter>
			<Button variant="outline" onclick={() => (dialogOpen = false)}>Cancel</Button>
			<Button onclick={handleSave} disabled={saving}>
				{#if saving}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
					Saving...
				{:else}
					{editingUser ? "Update" : "Create"}
				{/if}
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
