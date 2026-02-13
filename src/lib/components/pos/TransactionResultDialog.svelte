<script lang="ts">
	import { CheckCircle, XCircle, AlertTriangle } from "lucide-svelte";
	import {
		AlertDialog,
		AlertDialogAction,
		AlertDialogContent,
		AlertDialogDescription,
		AlertDialogFooter,
		AlertDialogHeader,
		AlertDialogTitle
	} from "$lib/components/ui/alert-dialog/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { formatCurrency } from "$lib/utils.js";
	import type { SolaTransactionResponse } from "$lib/types/index.js";

	interface Props {
		open: boolean;
		result: SolaTransactionResponse | null;
		amountCents: number;
		currencySymbol: string;
		onClose: () => void;
	}

	let { open = $bindable(), result, amountCents, currencySymbol, onClose }: Props = $props();

	const isApproved = $derived(result?.xResult === "A");
	const isDeclined = $derived(result?.xResult === "D");
	const isError = $derived(result?.xResult === "E");

	const statusIcon = $derived(() => {
		if (isApproved) return CheckCircle;
		if (isDeclined) return XCircle;
		return AlertTriangle;
	});

	const statusColor = $derived(() => {
		if (isApproved) return "text-green-600";
		if (isDeclined) return "text-red-600";
		return "text-yellow-600";
	});

	const lastFour = $derived(() => {
		if (!result?.xMaskedCardNumber) return null;
		// Extract last 4 from format: "************1234"
		return result.xMaskedCardNumber.slice(-4);
	});
</script>

<AlertDialog bind:open>
	<AlertDialogContent>
		<AlertDialogHeader>
			<AlertDialogTitle class="flex items-center gap-2">
				<svelte:component this={statusIcon()} class="h-6 w-6 {statusColor()}" />
				{#if isApproved}
					Transaction Approved
				{:else if isDeclined}
					Transaction Declined
				{:else}
					Transaction Error
				{/if}
			</AlertDialogTitle>
			<AlertDialogDescription>
				{#if isApproved && result}
					Your payment was successfully processed.
				{:else if isDeclined && result}
					{result.xError || "The card was declined. Please try another payment method."}
				{:else if result}
					{result.xError || "An error occurred while processing the transaction."}
				{/if}
			</AlertDialogDescription>
		</AlertDialogHeader>

		{#if result && isApproved}
			<div class="space-y-3 py-4">
				<div class="flex justify-between text-sm">
					<span class="text-muted-foreground">Amount</span>
					<span class="font-semibold">{formatCurrency(amountCents, currencySymbol)}</span>
				</div>

				{#if result.xCardType}
					<div class="flex justify-between text-sm">
						<span class="text-muted-foreground">Card Type</span>
						<Badge variant="outline">{result.xCardType}</Badge>
					</div>
				{/if}

				{#if lastFour()}
					<div class="flex justify-between text-sm">
						<span class="text-muted-foreground">Card Number</span>
						<span class="font-mono">****{lastFour()}</span>
					</div>
				{/if}

				{#if result.xAuthCode}
					<div class="flex justify-between text-sm">
						<span class="text-muted-foreground">Auth Code</span>
						<span class="font-mono">{result.xAuthCode}</span>
					</div>
				{/if}

				{#if result.xEntryMethod}
					<div class="flex justify-between text-sm">
						<span class="text-muted-foreground">Entry Method</span>
						<Badge variant="secondary">{result.xEntryMethod}</Badge>
					</div>
				{/if}

				{#if result.xRefnum}
					<div class="flex justify-between text-sm">
						<span class="text-muted-foreground">Reference</span>
						<span class="font-mono text-xs">{result.xRefnum}</span>
					</div>
				{/if}
			</div>
		{/if}

		<AlertDialogFooter>
			<AlertDialogAction onclick={onClose}>
				{isApproved ? "Continue" : "Close"}
			</AlertDialogAction>
		</AlertDialogFooter>
	</AlertDialogContent>
</AlertDialog>
