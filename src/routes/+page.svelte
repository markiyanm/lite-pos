<script lang="ts">
	import { goto } from "$app/navigation";
	import { session } from "$lib/stores/session.svelte.js";
	import { onMount } from "svelte";

	onMount(() => {
		if (session.isAuthenticated) {
			goto("/pos");
		}
	});
</script>

<div class="flex h-full items-center justify-center">
	<p class="text-muted-foreground">Redirecting...</p>
</div>
