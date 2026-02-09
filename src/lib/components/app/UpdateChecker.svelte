<script lang="ts">
	import { onMount } from "svelte";
	import { toast } from "svelte-sonner";

	onMount(async () => {
		try {
			const { check } = await import("@tauri-apps/plugin-updater");
			const update = await check();
			if (update) {
				toast.info(`Update available: v${update.version}`, {
					description: "Restart the app to install the update.",
					duration: 10000,
					action: {
						label: "Install",
						onClick: async () => {
							try {
								await update.downloadAndInstall();
								const { relaunch } = await import("@tauri-apps/plugin-process");
								await relaunch();
							} catch {
								toast.error("Failed to install update");
							}
						}
					}
				});
			}
		} catch {
			// Updater not available (e.g., dev mode) — ignore
		}
	});
</script>
