<script lang="ts">
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle
	} from "$lib/components/ui/dialog/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import { formatCurrency } from "$lib/utils.js";
	import { CreditCard } from "lucide-svelte";
	import type { Payment, SolaTransactionResponse } from "$lib/types/index.js";

	interface Props {
		open: boolean;
		payment: Payment | null;
		currencySymbol: string;
		onClose: () => void;
	}

	let { open = $bindable(), payment, currencySymbol, onClose }: Props = $props();

	const gatewayResponse = $derived.by<SolaTransactionResponse | null>(() => {
		if (!payment || !payment.gateway_response) return null;
		try {
			return JSON.parse(payment.gateway_response) as SolaTransactionResponse;
		} catch {
			return null;
		}
	});

	function formatCardNumber(masked: string | undefined): string {
		if (!masked) return "";
		// Format ************1234 as **** **** **** 1234
		if (masked.length === 16) {
			return masked.match(/.{1,4}/g)?.join(" ") || masked;
		}
		return masked;
	}

	function formatDateTime(dateStr: string): string {
		const date = new Date(dateStr);
		return date.toLocaleString("en-US", {
			dateStyle: "medium",
			timeStyle: "short"
		});
	}

	function formatExpDate(exp: string | undefined): string {
		if (!exp) return "";
		// If format is MMYY, convert to MM/YY
		if (exp.length === 4) {
			return `${exp.substring(0, 2)}/${exp.substring(2)}`;
		}
		return exp;
	}
</script>

<Dialog bind:open>
	<DialogContent class="max-w-md">
		<DialogHeader>
			<DialogTitle class="flex items-center gap-2">
				<CreditCard class="h-5 w-5" />
				Card Transaction Details
			</DialogTitle>
		</DialogHeader>

		{#if payment}
			<div class="space-y-4">
				<!-- Amount -->
				<div class="text-center">
					<p class="text-3xl font-bold">{formatCurrency(payment.amount_cents, currencySymbol)}</p>
				</div>

				<!-- Trans date -->
				<div class="text-center">
					<p class="text-sm text-muted-foreground">{formatDateTime(payment.created_at)}</p>
				</div>

				<Separator />

				<!-- Transaction details -->
				<div class="space-y-3">
					<!-- Result -->
					{#if gatewayResponse?.xStatus}
						<div class="flex justify-between text-sm">
							<span class="text-muted-foreground">Result</span>
							<span class="font-medium">{gatewayResponse.xStatus}</span>
						</div>
					{/if}

					<!-- Cardholder -->
					{#if gatewayResponse?.xName}
						<div class="flex justify-between text-sm">
							<span class="text-muted-foreground">Cardholder</span>
							<span class="font-medium">{gatewayResponse.xName}</span>
						</div>
					{/if}

					<!-- Card Type -->
					{#if payment.card_type}
						<div class="flex justify-between text-sm">
							<span class="text-muted-foreground">Card Type</span>
							<span class="font-medium">{payment.card_type}</span>
						</div>
					{/if}

					<!-- Card Number -->
					{#if gatewayResponse?.xMaskedCardNumber}
						<div class="flex justify-between text-sm">
							<span class="text-muted-foreground">Card Number</span>
							<span class="font-mono text-xs">{formatCardNumber(gatewayResponse.xMaskedCardNumber)}</span>
						</div>
					{/if}

					<!-- Exp Date -->
					{#if gatewayResponse?.xExp}
						<div class="flex justify-between text-sm">
							<span class="text-muted-foreground">Exp Date</span>
							<span class="font-medium">{formatExpDate(gatewayResponse.xExp)}</span>
						</div>
					{/if}

					<!-- Entry Mode -->
					{#if gatewayResponse?.xEntryMethod}
						<div class="flex justify-between text-sm">
							<span class="text-muted-foreground">Entry Mode</span>
							<span class="font-medium">{gatewayResponse.xEntryMethod}</span>
						</div>
					{/if}

					<!-- Auth Code -->
					{#if payment.card_auth_code}
						<div class="flex justify-between text-sm">
							<span class="text-muted-foreground">Auth Code</span>
							<span class="font-mono">{payment.card_auth_code}</span>
						</div>
					{/if}

					<!-- Gateway Reference # -->
					{#if gatewayResponse?.xRefnum}
						<div class="flex justify-between text-sm">
							<span class="text-muted-foreground">Gateway Reference #</span>
							<span class="font-mono text-xs">{gatewayResponse.xRefnum}</span>
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</DialogContent>
</Dialog>
