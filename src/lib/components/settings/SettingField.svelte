<script lang="ts">
	import { Label } from "$lib/components/ui/label/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Textarea } from "$lib/components/ui/textarea/index.js";

	interface Props {
		label: string;
		description?: string;
		value: string;
		type?: "text" | "number" | "textarea";
		placeholder?: string;
		onchange: (value: string) => void;
	}

	let { label, description, value, type = "text", placeholder = "", onchange }: Props = $props();
</script>

<div class="grid gap-2">
	<Label>{label}</Label>
	{#if description}
		<p class="text-sm text-muted-foreground">{description}</p>
	{/if}
	{#if type === "textarea"}
		<Textarea
			{value}
			{placeholder}
			oninput={(e) => onchange(e.currentTarget.value)}
		/>
	{:else}
		<Input
			{type}
			{value}
			{placeholder}
			oninput={(e) => onchange(e.currentTarget.value)}
		/>
	{/if}
</div>
