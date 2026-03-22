import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { select, execute } from "$lib/db/index.js";
import { decryptValue } from "$lib/commands/crypto.js";
import { settingsStore } from "$lib/stores/settings.svelte.js";
import type { Customer, GatewaySyncStatus } from "$lib/types/index.js";

// ---- Types for Rust command results ----

export interface SyncOperationResult {
	success: boolean;
	customer_id: string | null;
	revision: number | null;
	error: string | null;
	error_type: string | null;
}

export interface GatewayCustomerData {
	customer_id: string;
	customer_number: string;
	first_name: string;
	last_name: string;
	email: string;
	phone: string;
	street: string;
	city: string;
	state: string;
	zip: string;
	country: string;
	revision: number;
}

export interface ListGatewayCustomersResult {
	success: boolean;
	customers: GatewayCustomerData[];
	error: string | null;
	error_type: string | null;
}

// ---- Sync event types ----

export interface SyncStartedPayload {
	entity: "customers";
}

export interface SyncProgressPayload {
	entity: "customers";
	processed: number;
	total: number;
}

export interface SyncCompletedPayload {
	entity: "customers";
	created: number;
	updated: number;
	pulled: number;
	errors: number;
}

export interface SyncErrorPayload {
	entity: "customers";
	message: string;
}

// ---- Rust command wrappers ----

export async function acquireSyncLock(): Promise<boolean> {
	return invoke<boolean>("gateway_sync_acquire_lock");
}

export async function releaseSyncLock(): Promise<void> {
	return invoke<void>("gateway_sync_release_lock");
}

export async function isSyncRunning(): Promise<boolean> {
	return invoke<boolean>("gateway_sync_is_running");
}

async function listGatewayCustomers(apiKey: string): Promise<ListGatewayCustomersResult> {
	return invoke<ListGatewayCustomersResult>("gateway_list_customers", { apiKey });
}

async function createGatewayCustomer(
	apiKey: string,
	customer: {
		customer_number: string;
		first_name: string;
		last_name: string;
		email: string;
		phone: string;
		street: string;
		city: string;
		state: string;
		zip: string;
		country: string;
	}
): Promise<SyncOperationResult> {
	return invoke<SyncOperationResult>("gateway_create_customer", { apiKey, customer });
}

async function updateGatewayCustomer(
	apiKey: string,
	customer: {
		customer_id: string;
		customer_number: string;
		first_name: string;
		last_name: string;
		email: string;
		phone: string;
		street: string;
		city: string;
		state: string;
		zip: string;
		country: string;
		revision: number;
	}
): Promise<SyncOperationResult> {
	return invoke<SyncOperationResult>("gateway_update_customer", { apiKey, customer });
}

async function deleteGatewayCustomer(
	apiKey: string,
	customerId: string
): Promise<SyncOperationResult> {
	return invoke<SyncOperationResult>("gateway_delete_customer", { apiKey, customerId });
}

// ---- DB helpers ----

async function getLocalCustomersForSync(): Promise<Customer[]> {
	return select<Customer>("SELECT * FROM customers WHERE deleted_at IS NULL", []);
}

async function getSoftDeletedCustomersWithGatewayId(): Promise<Customer[]> {
	return select<Customer>(
		"SELECT * FROM customers WHERE deleted_at IS NOT NULL AND gateway_customer_id IS NOT NULL AND gateway_sync_status != 'archived'",
		[]
	);
}

async function getOrderCountForCustomer(customerId: number): Promise<number> {
	const rows = await select<{ cnt: number }>(
		"SELECT COUNT(*) as cnt FROM orders WHERE customer_id = ?",
		[customerId]
	);
	return rows[0]?.cnt ?? 0;
}

async function updateCustomerGatewayFields(
	id: number,
	gatewayCustomerId: string | null,
	syncStatus: GatewaySyncStatus,
	revision: number | null
): Promise<void> {
	const now = syncStatus === "synced" ? new Date().toISOString() : null;
	if (now) {
		await execute(
			"UPDATE customers SET gateway_customer_id = ?, gateway_sync_status = ?, gateway_synced_at = ?, gateway_revision = ? WHERE id = ?",
			[gatewayCustomerId, syncStatus, now, revision, id]
		);
	} else {
		await execute(
			"UPDATE customers SET gateway_customer_id = ?, gateway_sync_status = ?, gateway_revision = ? WHERE id = ?",
			[gatewayCustomerId, syncStatus, revision, id]
		);
	}
}

async function createLocalCustomerFromGateway(gw: GatewayCustomerData): Promise<void> {
	const uuid = crypto.randomUUID();
	await execute(
		`INSERT INTO customers (uuid, first_name, last_name, email, phone, billing_address_line1, billing_city, billing_state, billing_zip, gateway_customer_id, gateway_sync_status, gateway_synced_at, gateway_revision)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'synced', datetime('now'), ?)`,
		[
			uuid,
			gw.first_name || "Unknown",
			gw.last_name || "Unknown",
			gw.email || null,
			gw.phone || null,
			gw.street || null,
			gw.city || null,
			gw.state || null,
			gw.zip || null,
			gw.customer_id,
			gw.revision
		]
	);
}

async function logSyncOperation(
	entityId: number,
	direction: "push" | "pull",
	action: "create" | "update" | "delete",
	status: "success" | "error",
	errorMessage?: string
): Promise<void> {
	await execute(
		"INSERT INTO sync_log (entity_type, entity_id, direction, action, status, error_message) VALUES ('customer', ?, ?, ?, ?, ?)",
		[entityId, direction, action, status, errorMessage ?? null]
	);
}

// ---- Main sync orchestrator ----

export interface SyncResult {
	success: boolean;
	created: number;
	updated: number;
	pulled: number;
	errors: number;
	errorMessage?: string;
}

/**
 * Get the decrypted API key from settings. Returns null if not configured.
 */
export async function getDecryptedApiKey(): Promise<string | null> {
	const encryptedKey = settingsStore.get("sola_gateway_api_key");
	if (!encryptedKey) return null;
	try {
		const key = await decryptValue(encryptedKey);
		return key && key.trim() ? key.trim() : null;
	} catch {
		return null;
	}
}

/**
 * Run a full bidirectional customer sync with the gateway.
 * This is the main entry point called from the UI and background timer.
 */
export async function syncCustomersWithGateway(
	apiKey: string,
	onProgress?: (processed: number, total: number) => void
): Promise<SyncResult> {
	const result: SyncResult = { success: true, created: 0, updated: 0, pulled: 0, errors: 0 };

	// Step 1: List gateway customers
	const gwResult = await listGatewayCustomers(apiKey);
	if (!gwResult.success) {
		return {
			success: false,
			created: 0,
			updated: 0,
			pulled: 0,
			errors: 1,
			errorMessage: gwResult.error ?? "Failed to list gateway customers"
		};
	}

	const gwCustomers = gwResult.customers;

	// Step 2: Get all local customers
	const localCustomers = await getLocalCustomersForSync();

	// Build lookup maps
	const gwByCustomerNumber = new Map<string, GatewayCustomerData>();
	const gwByCustomerId = new Map<string, GatewayCustomerData>();
	for (const gw of gwCustomers) {
		if (gw.customer_number) {
			gwByCustomerNumber.set(gw.customer_number, gw);
		}
		gwByCustomerId.set(gw.customer_id, gw);
	}

	const localByGatewayId = new Map<string, Customer>();
	const localByUuid = new Map<string, Customer>();
	for (const lc of localCustomers) {
		if (lc.gateway_customer_id) {
			localByGatewayId.set(lc.gateway_customer_id, lc);
		}
		localByUuid.set(lc.uuid, lc);
	}

	const totalWork = localCustomers.length + gwCustomers.length;
	let processed = 0;

	// Step 3: Process local customers - push to gateway
	for (const local of localCustomers) {
		try {
			if (local.gateway_customer_id) {
				// Already linked to gateway - check if local is newer and needs push
				const gw = gwByCustomerId.get(local.gateway_customer_id);
				if (gw) {
					// Check if local was updated after last sync
					const localUpdated = local.updated_at;
					const lastSynced = local.gateway_synced_at;

					if (lastSynced && localUpdated > lastSynced) {
						// Local is newer, push update
						const updateResult = await updateGatewayCustomer(apiKey, {
							customer_id: local.gateway_customer_id,
							customer_number: local.uuid,
							first_name: local.first_name,
							last_name: local.last_name,
							email: local.email ?? "",
							phone: local.phone ?? "",
							street: local.billing_address_line1 ?? "",
							city: local.billing_city ?? "",
							state: local.billing_state ?? "",
							zip: local.billing_zip ?? "",
							country: "US",
							revision: local.gateway_revision ?? gw.revision
						});

						if (updateResult.success) {
							await updateCustomerGatewayFields(
								local.id,
								local.gateway_customer_id,
								"synced",
								updateResult.revision
							);
							await logSyncOperation(local.id, "push", "update", "success");
							result.updated++;
						} else {
							await updateCustomerGatewayFields(local.id, local.gateway_customer_id, "error", local.gateway_revision);
							await logSyncOperation(local.id, "push", "update", "error", updateResult.error ?? undefined);
							result.errors++;
						}
					} else if (gw.revision > (local.gateway_revision ?? 0)) {
						// Gateway is newer, pull update
						await execute(
							`UPDATE customers SET first_name = ?, last_name = ?, email = ?, phone = ?, billing_address_line1 = ?, billing_city = ?, billing_state = ?, billing_zip = ?, gateway_sync_status = 'synced', gateway_synced_at = datetime('now'), gateway_revision = ? WHERE id = ?`,
							[
								gw.first_name || local.first_name,
								gw.last_name || local.last_name,
								gw.email || local.email,
								gw.phone || local.phone,
								gw.street || local.billing_address_line1,
								gw.city || local.billing_city,
								gw.state || local.billing_state,
								gw.zip || local.billing_zip,
								gw.revision,
								local.id
							]
						);
						await logSyncOperation(local.id, "pull", "update", "success");
						result.pulled++;
					} else {
						// Already in sync, just mark as synced
						if (local.gateway_sync_status !== "synced") {
							await updateCustomerGatewayFields(local.id, local.gateway_customer_id, "synced", local.gateway_revision);
						}
					}
				}
			} else {
				// No gateway ID - check if gateway has a customer with our UUID as CustomerNumber
				const gwMatch = gwByCustomerNumber.get(local.uuid);
				if (gwMatch) {
					// Found a match by UUID - link them
					await updateCustomerGatewayFields(local.id, gwMatch.customer_id, "synced", gwMatch.revision);
					await logSyncOperation(local.id, "pull", "update", "success");
					result.pulled++;
				} else {
					// New local customer - push to gateway
					const createResult = await createGatewayCustomer(apiKey, {
						customer_number: local.uuid,
						first_name: local.first_name,
						last_name: local.last_name,
						email: local.email ?? "",
						phone: local.phone ?? "",
						street: local.billing_address_line1 ?? "",
						city: local.billing_city ?? "",
						state: local.billing_state ?? "",
						zip: local.billing_zip ?? "",
						country: "US"
					});

					if (createResult.success) {
						await updateCustomerGatewayFields(
							local.id,
							createResult.customer_id,
							"synced",
							createResult.revision
						);
						await logSyncOperation(local.id, "push", "create", "success");
						result.created++;
					} else {
						await updateCustomerGatewayFields(local.id, null, "error", null);
						await logSyncOperation(local.id, "push", "create", "error", createResult.error ?? undefined);
						result.errors++;
					}
				}
			}
		} catch (err) {
			result.errors++;
			await logSyncOperation(local.id, "push", "update", "error", String(err));
		}

		processed++;
		onProgress?.(processed, totalWork);
	}

	// Step 4: Process gateway customers not linked to any local customer - pull
	for (const gw of gwCustomers) {
		if (!localByGatewayId.has(gw.customer_id) && !localByUuid.has(gw.customer_number)) {
			try {
				await createLocalCustomerFromGateway(gw);
				// Get the ID of the newly created customer for logging
				const newRows = await select<{ id: number }>(
					"SELECT id FROM customers WHERE gateway_customer_id = ?",
					[gw.customer_id]
				);
				if (newRows[0]) {
					await logSyncOperation(newRows[0].id, "pull", "create", "success");
				}
				result.pulled++;
			} catch (err) {
				result.errors++;
			}
		}

		processed++;
		onProgress?.(processed, totalWork);
	}

	// Step 5: Handle soft-deleted local customers
	try {
		const deletedCustomers = await getSoftDeletedCustomersWithGatewayId();
		for (const dc of deletedCustomers) {
			const orderCount = await getOrderCountForCustomer(dc.id);
			if (orderCount > 0) {
				// Has orders - preserve on gateway, mark as archived locally
				await execute(
					"UPDATE customers SET gateway_sync_status = 'archived' WHERE id = ?",
					[dc.id]
				);
			} else {
				// No orders - delete from gateway
				if (dc.gateway_customer_id) {
					const deleteResult = await deleteGatewayCustomer(apiKey, dc.gateway_customer_id);
					if (deleteResult.success) {
						await logSyncOperation(dc.id, "push", "delete", "success");
					} else {
						// Gateway delete failed (e.g., active schedules) - just log, keep local state
						await logSyncOperation(dc.id, "push", "delete", "error", deleteResult.error ?? undefined);
					}
				}
			}
		}
	} catch {
		// Non-fatal: delete handling errors don't affect overall sync result
	}

	return result;
}

// ---- Event listeners ----

export function onSyncStarted(callback: (payload: SyncStartedPayload) => void): Promise<UnlistenFn> {
	return listen<SyncStartedPayload>("gateway-sync-started", (event) => callback(event.payload));
}

export function onSyncProgress(callback: (payload: SyncProgressPayload) => void): Promise<UnlistenFn> {
	return listen<SyncProgressPayload>("gateway-sync-progress", (event) => callback(event.payload));
}

export function onSyncCompleted(callback: (payload: SyncCompletedPayload) => void): Promise<UnlistenFn> {
	return listen<SyncCompletedPayload>("gateway-sync-completed", (event) => callback(event.payload));
}

export function onSyncError(callback: (payload: SyncErrorPayload) => void): Promise<UnlistenFn> {
	return listen<SyncErrorPayload>("gateway-sync-error", (event) => callback(event.payload));
}
