<script lang="ts">
	import { goto } from "$app/navigation";
	import { Store, Delete, Loader2 } from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "$lib/components/ui/card/index.js";
	import { login } from "$lib/commands/auth.js";
	import { session } from "$lib/stores/session.svelte.js";
	import { settingsStore } from "$lib/stores/settings.svelte.js";
	import { getAllSettings } from "$lib/commands/settings.js";

	let pin = $state("");
	let error = $state("");
	let loading = $state(false);

	const maskedPin = $derived("●".repeat(pin.length));

	function appendDigit(digit: string) {
		if (pin.length < 6) {
			pin += digit;
			error = "";
		}
	}

	function deleteDigit() {
		pin = pin.slice(0, -1);
		error = "";
	}

	function clearPin() {
		pin = "";
		error = "";
	}

	async function handleLogin() {
		if (pin.length === 0) return;
		loading = true;
		error = "";

		try {
			const user = await login(pin);
			if (user) {
				session.login(user);

				// Load settings after login
				if (!settingsStore.loaded) {
					const settings = await getAllSettings();
					settingsStore.load(settings);
				}

				goto("/pos");
			} else {
				error = "Invalid PIN. Please try again.";
				pin = "";
			}
		} catch {
			error = "Login failed. Please try again.";
			pin = "";
		} finally {
			loading = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (loading) return;
		if (e.key >= "0" && e.key <= "9") {
			appendDigit(e.key);
		} else if (e.key === "Backspace") {
			deleteDigit();
		} else if (e.key === "Escape") {
			clearPin();
		} else if (e.key === "Enter") {
			handleLogin();
		}
	}

	const numpadKeys = [
		["1", "2", "3"],
		["4", "5", "6"],
		["7", "8", "9"],
		["clear", "0", "backspace"]
	];
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex h-screen items-center justify-center bg-muted/30">
	<Card class="w-full max-w-sm">
		<CardHeader class="text-center">
			<div class="mx-auto mb-2 flex h-12 w-12 items-center justify-center rounded-xl bg-primary text-primary-foreground">
				<Store class="h-6 w-6" />
			</div>
			<CardTitle class="text-2xl">Lite POS</CardTitle>
			<CardDescription>Enter your PIN to sign in</CardDescription>
		</CardHeader>
		<CardContent>
			<div class="space-y-4">
				<!-- PIN display -->
				<div
					class="flex h-14 items-center justify-center rounded-md border bg-muted/50 text-2xl tracking-[0.5em]"
					class:border-destructive={error}
				>
					{#if pin.length > 0}
						{maskedPin}
					{:else}
						<span class="text-muted-foreground text-base tracking-normal">Enter PIN</span>
					{/if}
				</div>

				<!-- Error message -->
				{#if error}
					<p class="text-center text-sm text-destructive">{error}</p>
				{/if}

				<!-- Numpad -->
				<div class="grid grid-cols-3 gap-2">
					{#each numpadKeys as row}
						{#each row as key}
							{#if key === "clear"}
								<Button
									variant="outline"
									class="h-14 text-sm"
									onclick={clearPin}
									disabled={loading}
								>
									Clear
								</Button>
							{:else if key === "backspace"}
								<Button
									variant="outline"
									class="h-14"
									onclick={deleteDigit}
									disabled={loading}
								>
									<Delete class="h-5 w-5" />
								</Button>
							{:else}
								<Button
									variant="outline"
									class="h-14 text-xl font-medium"
									onclick={() => appendDigit(key)}
									disabled={loading}
								>
									{key}
								</Button>
							{/if}
						{/each}
					{/each}
				</div>

				<!-- Sign in button -->
				<Button
					class="w-full"
					size="lg"
					onclick={handleLogin}
					disabled={pin.length === 0 || loading}
				>
					{#if loading}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						Signing in...
					{:else}
						Sign In
					{/if}
				</Button>
			</div>
		</CardContent>
	</Card>
</div>
