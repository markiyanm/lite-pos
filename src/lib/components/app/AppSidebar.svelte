<script lang="ts">
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import {
		ShoppingCart,
		ClipboardList,
		Package,
		Tags,
		Users,
		Settings,
		LogOut,
		Store
	} from "lucide-svelte";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { session } from "$lib/stores/session.svelte.js";

	const navGroups = [
		{
			label: "Point of Sale",
			items: [
				{ label: "POS", href: "/pos", icon: ShoppingCart },
				{ label: "Orders", href: "/orders", icon: ClipboardList }
			]
		},
		{
			label: "Catalog",
			items: [
				{ label: "Products", href: "/products", icon: Package },
				{ label: "Categories", href: "/categories", icon: Tags },
				{ label: "Customers", href: "/customers", icon: Users }
			]
		},
		{
			label: "Administration",
			items: [
				{ label: "Settings", href: "/settings", icon: Settings }
			]
		}
	];

	function isActive(href: string): boolean {
		return page.url.pathname === href || page.url.pathname.startsWith(href + "/");
	}

	function handleLogout() {
		session.logout();
		goto("/login");
	}
</script>

<aside class="flex h-screen w-60 flex-col border-r border-sidebar-border bg-sidebar">
	<!-- Logo / Brand -->
	<div class="flex items-center gap-2 px-4 py-5">
		<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
			<Store class="h-4 w-4" />
		</div>
		<span class="text-lg font-semibold text-sidebar-foreground">Lite POS</span>
	</div>

	<Separator />

	<!-- Navigation -->
	<nav class="flex-1 overflow-y-auto px-3 py-4">
		{#each navGroups as group}
			<div class="mb-4">
				<p class="mb-1 px-2 text-xs font-medium uppercase tracking-wider text-sidebar-foreground/50">
					{group.label}
				</p>
				{#each group.items as item}
					{@const active = isActive(item.href)}
					<a
						href={item.href}
						class="mb-0.5 flex items-center gap-3 rounded-md px-2 py-2 text-sm transition-colors
							{active
								? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium'
								: 'text-sidebar-foreground hover:bg-sidebar-accent/50 hover:text-sidebar-accent-foreground'}"
					>
						<item.icon class="h-4 w-4" />
						{item.label}
					</a>
				{/each}
			</div>
		{/each}
	</nav>

	<!-- Footer -->
	<div class="border-t border-sidebar-border px-3 py-3">
		{#if session.user}
			<div class="mb-2 flex items-center gap-2 px-2">
				<div class="flex h-8 w-8 items-center justify-center rounded-full bg-sidebar-accent text-xs font-medium text-sidebar-accent-foreground">
					{session.user.name.charAt(0).toUpperCase()}
				</div>
				<div class="flex-1 truncate">
					<p class="truncate text-sm font-medium text-sidebar-foreground">{session.user.name}</p>
					<p class="text-xs text-sidebar-foreground/50 capitalize">{session.user.role}</p>
				</div>
			</div>
		{/if}
		<Button variant="ghost" class="w-full justify-start gap-2 text-sidebar-foreground/70" onclick={handleLogout}>
			<LogOut class="h-4 w-4" />
			Logout
		</Button>
	</div>
</aside>
