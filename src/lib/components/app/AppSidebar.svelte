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
		Store,
		PanelLeftClose,
		PanelLeftOpen,
		BarChart3
	} from "lucide-svelte";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { session } from "$lib/stores/session.svelte.js";
	import { settingsStore } from "$lib/stores/settings.svelte.js";

	let collapsed = $state(
		typeof localStorage !== "undefined" && localStorage.getItem("sidebar-collapsed") === "true"
	);

	function toggleCollapsed() {
		collapsed = !collapsed;
		localStorage.setItem("sidebar-collapsed", String(collapsed));
	}

	const storeName = $derived(settingsStore.get("store_name") || "Lite POS");
	const storeLogo = $derived(settingsStore.get("store_logo"));

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
				{ label: "Reports", href: "/reports", icon: BarChart3 },
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

<aside
	class="flex h-screen flex-col border-r border-sidebar-border bg-sidebar transition-[width] duration-200 ease-in-out {collapsed ? 'w-16' : 'w-60'}"
>
	<!-- Logo / Brand -->
	<div class="flex items-center {collapsed ? 'justify-center px-2' : 'gap-2 px-4'} py-5">
		{#if storeLogo}
			<img
				src={storeLogo}
				alt="Logo"
				class="h-8 w-8 shrink-0 rounded-lg object-contain"
			/>
		{:else}
			<div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground">
				<Store class="h-4 w-4" />
			</div>
		{/if}
		{#if !collapsed}
			<span class="truncate text-lg font-semibold text-sidebar-foreground">{storeName}</span>
		{/if}
	</div>

	<Separator />

	<!-- Toggle + Navigation -->
	<div class="flex items-center {collapsed ? 'justify-center' : 'justify-end'} px-2 pt-2">
		<Button
			variant="ghost"
			size="icon"
			class="h-7 w-7 text-sidebar-foreground/50 hover:text-sidebar-foreground"
			onclick={toggleCollapsed}
		>
			{#if collapsed}
				<PanelLeftOpen class="h-4 w-4" />
			{:else}
				<PanelLeftClose class="h-4 w-4" />
			{/if}
		</Button>
	</div>

	<nav class="flex-1 overflow-y-auto px-3 py-2">
		{#each navGroups as group}
			<div class="mb-4">
				{#if !collapsed}
					<p class="mb-1 px-2 text-xs font-medium uppercase tracking-wider text-sidebar-foreground/50">
						{group.label}
					</p>
				{/if}
				{#each group.items as item}
					{@const active = isActive(item.href)}
					<a
						href={item.href}
						class="mb-0.5 flex items-center rounded-md py-2 text-sm transition-colors
							{collapsed ? 'justify-center px-0' : 'gap-3 px-2'}
							{active
								? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium'
								: 'text-sidebar-foreground hover:bg-sidebar-accent/50 hover:text-sidebar-accent-foreground'}"
						title={collapsed ? item.label : undefined}
					>
						<item.icon class="h-4 w-4 shrink-0" />
						{#if !collapsed}
							{item.label}
						{/if}
					</a>
				{/each}
			</div>
		{/each}
	</nav>

	<!-- Footer -->
	<div class="border-t border-sidebar-border px-3 py-3">
		{#if session.user}
			<div class="mb-2 flex items-center {collapsed ? 'justify-center' : 'gap-2 px-2'}">
				<div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-sidebar-accent text-xs font-medium text-sidebar-accent-foreground">
					{session.user.name.charAt(0).toUpperCase()}
				</div>
				{#if !collapsed}
					<div class="flex-1 truncate">
						<p class="truncate text-sm font-medium text-sidebar-foreground">{session.user.name}</p>
						<p class="text-xs text-sidebar-foreground/50 capitalize">{session.user.role}</p>
					</div>
				{/if}
			</div>
		{/if}
		<Button
			variant="ghost"
			class="{collapsed ? 'w-full justify-center px-0' : 'w-full justify-start gap-2'} text-sidebar-foreground/70"
			onclick={handleLogout}
			title={collapsed ? "Logout" : undefined}
		>
			<LogOut class="h-4 w-4 shrink-0" />
			{#if !collapsed}
				Logout
			{/if}
		</Button>
	</div>
</aside>
