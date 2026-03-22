<script lang="ts">
	import { Wifi, WifiOff } from "lucide-svelte";
	import {
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger
	} from "$lib/components/ui/select/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import type { SolaDevice } from "$lib/types/index.js";

	interface Props {
		devices: SolaDevice[];
		selectedDeviceId: string;
		onSelect: (deviceId: string) => void;
		disabled?: boolean;
	}

	let { devices = $bindable(), selectedDeviceId = $bindable(), onSelect, disabled = false }: Props = $props();

	const selectedDevice = $derived(devices.find((d) => d.xDeviceId === selectedDeviceId) || null);

	function handleValueChange(value: string) {
		if (value) {
			selectedDeviceId = value;
			onSelect(value);
		}
	}

	function getStatusBadge(status: SolaDevice["xDeviceStatus"]) {
		switch (status) {
			case "CONNECTED":
				return { variant: "default" as const, text: "Connected", icon: Wifi };
			case "DISCONNECTED":
				return { variant: "secondary" as const, text: "Disconnected", icon: WifiOff };
			case "BUSY":
				return { variant: "secondary" as const, text: "Busy", icon: Wifi };
			default:
				return { variant: "secondary" as const, text: status, icon: WifiOff };
		}
	}
</script>

<div class="space-y-2">
	<Select type="single" value={selectedDeviceId} onValueChange={handleValueChange} {disabled}>
		<SelectTrigger class="w-full">
			{#if selectedDevice}
				<span class="flex items-center gap-2">
					{selectedDevice.xDeviceFriendlyName}
					{#if selectedDevice.xDeviceStatus === "CONNECTED"}
						<Wifi class="h-4 w-4 text-green-600" />
					{:else}
						<WifiOff class="h-4 w-4 text-muted-foreground" />
					{/if}
				</span>
			{:else}
				<span class="text-muted-foreground">Select device...</span>
			{/if}
		</SelectTrigger>
		<SelectContent>
			{#each devices as device (device.xDeviceId)}
				{@const badge = getStatusBadge(device.xDeviceStatus)}
				{@const Icon = badge.icon}
				<SelectItem value={device.xDeviceId}>
					<div class="flex items-center justify-between w-full gap-3">
						<div class="flex flex-col items-start">
							<span>{device.xDeviceFriendlyName}</span>
							<span class="text-xs text-muted-foreground">
								S/N: {device.xDeviceSerialNumber}
							</span>
						</div>
						<Badge variant={badge.variant} class="shrink-0">
							<Icon class="h-3 w-3 mr-1" />
							{badge.text}
						</Badge>
					</div>
				</SelectItem>
			{/each}
		</SelectContent>
	</Select>

	{#if selectedDevice && selectedDevice.xDeviceStatus !== "CONNECTED"}
		<p class="text-xs text-yellow-600">
			Warning: Device is not connected. Transaction may fail.
		</p>
	{/if}
</div>
