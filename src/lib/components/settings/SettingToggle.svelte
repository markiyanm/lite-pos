<script lang="ts">
	import { Label } from "$lib/components/ui/label/index.js";
	import { Switch } from "$lib/components/ui/switch/index.js";

	interface Props {
		label: string;
		description?: string;
		checked: boolean;
		onchange: (value: boolean) => void;
		disabled?: boolean;
	}

	let { label, description, checked, onchange, disabled = false }: Props = $props();
</script>

<div class="flex items-center justify-between gap-4 rounded-lg border p-4" class:opacity-50={disabled}>
	<div class="space-y-0.5">
		<Label>{label}</Label>
		{#if description}
			<p class="text-sm text-muted-foreground">{description}</p>
		{/if}
	</div>
	<Switch
		{checked}
		{disabled}
		onCheckedChange={onchange}
	/>
</div>
