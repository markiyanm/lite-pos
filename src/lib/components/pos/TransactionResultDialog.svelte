<script lang="ts">
	import { CheckCircle, XCircle, AlertTriangle, Loader2, Copy, Check, Ban } from "lucide-svelte";
	import {
		AlertDialog,
		AlertDialogAction,
		AlertDialogContent,
		AlertDialogFooter,
		AlertDialogHeader,
		AlertDialogTitle
	} from "$lib/components/ui/alert-dialog/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import { formatCurrency } from "$lib/utils.js";
	import type {
		SolaTransactionResponse,
		SolaRequestInfo,
		SolaTransactionResult
	} from "$lib/types/index.js";

	interface Props {
		open: boolean;
		/** Request info shown immediately when dialog opens */
		requestInfo: SolaRequestInfo | null;
		/** Full result including response — filled when response arrives */
		result: SolaTransactionResult | null;
		/** Error string if the request itself failed (network error, timeout) */
		error: string | null;
		/** Whether we're waiting for the response */
		pending: boolean;
		amountCents: number;
		currencySymbol: string;
		onClose: () => void;
		onCancel: () => void;
		/** Whether a cancel request is in progress */
		cancelling: boolean;
		/** Cancel request info — shown when cancel is sent */
		cancelRequestInfo: SolaRequestInfo | null;
		/** Cancel result — filled when cancel response arrives */
		cancelResult: SolaTransactionResult | null;
		cancelError: string | null;
	}

	let {
		open = $bindable(),
		requestInfo,
		result,
		error,
		pending,
		amountCents,
		currencySymbol,
		onClose,
		onCancel,
		cancelling,
		cancelRequestInfo,
		cancelResult,
		cancelError
	}: Props = $props();

	let copiedField = $state("");

	const response = $derived(result?.response ?? null);
	const isApproved = $derived(response?.xResult === "A");
	const isDeclined = $derived(response?.xResult === "D");
	const isError = $derived(response?.xResult === "E");
	const hasResponse = $derived(response !== null);

	async function copyToClipboard(text: string, field: string) {
		try {
			await navigator.clipboard.writeText(text);
			copiedField = field;
			setTimeout(() => (copiedField = ""), 2000);
		} catch {
			// ignore
		}
	}

	function formatTimestamp(epoch: string): string {
		const secs = parseFloat(epoch);
		if (isNaN(secs)) return epoch;
		const date = new Date(secs * 1000);
		return date.toLocaleString("en-US", {
			dateStyle: "short",
			timeStyle: "medium"
		});
	}
</script>

<AlertDialog bind:open>
	<AlertDialogContent class="max-w-xl max-h-[85vh] overflow-y-auto">
		<AlertDialogHeader>
			<AlertDialogTitle class="flex items-center gap-2">
				{#if pending}
					<Loader2 class="h-5 w-5 animate-spin text-primary" />
					Sending to Device...
				{:else if error}
					<AlertTriangle class="h-5 w-5 text-destructive" />
					Request Failed
				{:else if isApproved}
					<CheckCircle class="h-5 w-5 text-green-600" />
					Transaction Approved
				{:else if isDeclined}
					<XCircle class="h-5 w-5 text-red-600" />
					Transaction Declined
				{:else if isError}
					<AlertTriangle class="h-5 w-5 text-yellow-600" />
					Transaction Error
				{:else}
					<Loader2 class="h-5 w-5 animate-spin" />
					Processing...
				{/if}
			</AlertDialogTitle>
		</AlertDialogHeader>

		<div class="space-y-3">
			<!-- REQUEST PANE — shown immediately -->
			{#if requestInfo}
				<div class="rounded-md border bg-muted/30 p-3 space-y-2">
					<div class="flex items-center justify-between">
						<span class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Request</span>
						<Badge variant="outline" class="text-xs font-mono">{requestInfo.command}</Badge>
					</div>
					<div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
						<span class="text-muted-foreground">URL</span>
						<span class="font-mono truncate">{requestInfo.url}</span>

						<span class="text-muted-foreground">xKey</span>
						<span class="font-mono">{requestInfo.masked_key}</span>

						<span class="text-muted-foreground">xCommand</span>
						<span class="font-mono">{requestInfo.command}</span>

						<span class="text-muted-foreground">xAmount</span>
						<span class="font-mono">{requestInfo.amount}</span>

						<span class="text-muted-foreground">xDeviceId</span>
						<div class="flex items-center gap-1">
							<span class="font-mono truncate">{requestInfo.device_id}</span>
							<button
								type="button"
								class="shrink-0 text-muted-foreground hover:text-foreground"
								onclick={() => copyToClipboard(requestInfo!.device_id, "req-device")}
							>
								{#if copiedField === "req-device"}
									<Check class="h-3 w-3 text-green-500" />
								{:else}
									<Copy class="h-3 w-3" />
								{/if}
							</button>
						</div>

						<span class="text-muted-foreground">xRequestId</span>
						<div class="flex items-center gap-1">
							<span class="font-mono truncate">{requestInfo.request_id}</span>
							<button
								type="button"
								class="shrink-0 text-muted-foreground hover:text-foreground"
								onclick={() => copyToClipboard(requestInfo!.request_id, "req-id")}
							>
								{#if copiedField === "req-id"}
									<Check class="h-3 w-3 text-green-500" />
								{:else}
									<Copy class="h-3 w-3" />
								{/if}
							</button>
						</div>

						{#if requestInfo.invoice}
							<span class="text-muted-foreground">xInvoice</span>
							<span class="font-mono">{requestInfo.invoice}</span>
						{/if}

						<span class="text-muted-foreground">Sent At</span>
						<span class="font-mono">{formatTimestamp(requestInfo.timestamp)}</span>
					</div>
				</div>
			{/if}

			<!-- PENDING INDICATOR -->
			{#if pending}
				<div class="flex items-center gap-3 rounded-md border border-dashed p-4">
					<Loader2 class="h-5 w-5 animate-spin text-primary" />
					<div>
						<p class="text-sm font-medium">Waiting for device response...</p>
						<p class="text-xs text-muted-foreground">Present card on the terminal to continue.</p>
					</div>
				</div>
			{/if}

			<!-- NETWORK ERROR -->
			{#if error}
				<div class="rounded-md border border-destructive/50 bg-destructive/5 p-3 space-y-1">
					<span class="text-xs font-semibold uppercase tracking-wide text-destructive">Error</span>
					<p class="text-sm">{error}</p>
				</div>
			{/if}

			<!-- RESPONSE PANE — shown once response arrives -->
			{#if hasResponse && result}
				<div class="rounded-md border p-3 space-y-2 {isApproved ? 'bg-green-500/5 border-green-500/30' : isDeclined ? 'bg-red-500/5 border-red-500/30' : 'bg-yellow-500/5 border-yellow-500/30'}">
					<div class="flex items-center justify-between">
						<span class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Response</span>
						<div class="flex items-center gap-2">
							<Badge variant={isApproved ? "default" : "destructive"} class="text-xs">
								{response?.xResult === "A" ? "Approved" : response?.xResult === "D" ? "Declined" : "Error"}
							</Badge>
							<span class="text-xs text-muted-foreground">
								HTTP {result.http_status} &middot; {result.duration_ms}ms
							</span>
						</div>
					</div>

					<div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
						<span class="text-muted-foreground">xResult</span>
						<span class="font-mono font-semibold">{response?.xResult}</span>

						<span class="text-muted-foreground">xStatus</span>
						<span class="font-mono">{response?.xStatus}</span>

						{#if response?.xError}
							<span class="text-muted-foreground">xError</span>
							<span class="font-mono text-destructive">{response.xError}</span>
						{/if}

						<span class="text-muted-foreground">xRefnum</span>
						<div class="flex items-center gap-1">
							<span class="font-mono truncate">{response?.xRefnum}</span>
							{#if response?.xRefnum}
								<button
									type="button"
									class="shrink-0 text-muted-foreground hover:text-foreground"
									onclick={() => copyToClipboard(response!.xRefnum, "res-ref")}
								>
									{#if copiedField === "res-ref"}
										<Check class="h-3 w-3 text-green-500" />
									{:else}
										<Copy class="h-3 w-3" />
									{/if}
								</button>
							{/if}
						</div>

						{#if response?.xAuthCode}
							<span class="text-muted-foreground">xAuthCode</span>
							<span class="font-mono">{response.xAuthCode}</span>
						{/if}

						{#if response?.xMaskedCardNumber}
							<span class="text-muted-foreground">xMaskedCard</span>
							<span class="font-mono">{response.xMaskedCardNumber}</span>
						{/if}

						{#if response?.xCardType}
							<span class="text-muted-foreground">xCardType</span>
							<span class="font-mono">{response.xCardType}</span>
						{/if}

						{#if response?.xName}
							<span class="text-muted-foreground">xName</span>
							<span class="font-mono">{response.xName}</span>
						{/if}

						{#if response?.xEntryMethod}
							<span class="text-muted-foreground">xEntryMethod</span>
							<span class="font-mono">{response.xEntryMethod}</span>
						{/if}

						{#if response?.xAmt}
							<span class="text-muted-foreground">xAmt</span>
							<span class="font-mono">{response.xAmt}</span>
						{/if}

						{#if response?.xAvsResult}
							<span class="text-muted-foreground">xAvsResult</span>
							<span class="font-mono">{response.xAvsResult}</span>
						{/if}

						{#if response?.xCvvResult}
							<span class="text-muted-foreground">xCvvResult</span>
							<span class="font-mono">{response.xCvvResult}</span>
						{/if}
					</div>
				</div>
			{/if}

			<!-- CANCEL SECTION -->
			{#if cancelRequestInfo || cancelResult || cancelError}
				<Separator />

				<!-- Cancel Request -->
				{#if cancelRequestInfo}
					<div class="rounded-md border bg-muted/30 p-3 space-y-2">
						<div class="flex items-center justify-between">
							<span class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Cancel Request</span>
							<Badge variant="outline" class="text-xs font-mono">{cancelRequestInfo.command}</Badge>
						</div>
						<div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
							<span class="text-muted-foreground">URL</span>
							<span class="font-mono truncate">{cancelRequestInfo.url}</span>

							<span class="text-muted-foreground">xCommand</span>
							<span class="font-mono">{cancelRequestInfo.command}</span>

							<span class="text-muted-foreground">xDeviceId</span>
							<span class="font-mono truncate">{cancelRequestInfo.device_id}</span>

							<span class="text-muted-foreground">xRequestId</span>
							<span class="font-mono truncate">{cancelRequestInfo.request_id}</span>

							<span class="text-muted-foreground">Sent At</span>
							<span class="font-mono">{formatTimestamp(cancelRequestInfo.timestamp)}</span>
						</div>
					</div>
				{/if}

				<!-- Cancel pending -->
				{#if cancelling && !cancelResult && !cancelError}
					<div class="flex items-center gap-3 rounded-md border border-dashed p-3">
						<Loader2 class="h-4 w-4 animate-spin text-muted-foreground" />
						<p class="text-sm text-muted-foreground">Sending cancel to device...</p>
					</div>
				{/if}

				<!-- Cancel error -->
				{#if cancelError}
					<div class="rounded-md border border-destructive/50 bg-destructive/5 p-3 space-y-1">
						<span class="text-xs font-semibold uppercase tracking-wide text-destructive">Cancel Error</span>
						<p class="text-sm">{cancelError}</p>
					</div>
				{/if}

				<!-- Cancel response -->
				{#if cancelResult}
					<div class="rounded-md border bg-muted/30 p-3 space-y-2">
						<div class="flex items-center justify-between">
							<span class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Cancel Response</span>
							<span class="text-xs text-muted-foreground">
								HTTP {cancelResult.http_status} &middot; {cancelResult.duration_ms}ms
							</span>
						</div>
						<div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
							<span class="text-muted-foreground">xResult</span>
							<span class="font-mono font-semibold">{cancelResult.response.xResult}</span>

							<span class="text-muted-foreground">xStatus</span>
							<span class="font-mono">{cancelResult.response.xStatus}</span>

							{#if cancelResult.response.xError}
								<span class="text-muted-foreground">xError</span>
								<span class="font-mono text-destructive">{cancelResult.response.xError}</span>
							{/if}

							<span class="text-muted-foreground">xRefnum</span>
							<span class="font-mono">{cancelResult.response.xRefnum}</span>
						</div>
					</div>
				{/if}
			{/if}
		</div>

		<AlertDialogFooter class="gap-2 sm:gap-0">
			<!-- Cancel button — only when pending (waiting for device) -->
			{#if pending && !cancelling}
				<Button variant="destructive" onclick={onCancel}>
					<Ban class="h-4 w-4 mr-2" />
					Cancel Transaction
				</Button>
			{/if}

			{#if !pending}
				<AlertDialogAction onclick={onClose}>
					{isApproved ? "Continue" : "Close"}
				</AlertDialogAction>
			{/if}
		</AlertDialogFooter>
	</AlertDialogContent>
</AlertDialog>
