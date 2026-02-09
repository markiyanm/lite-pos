import type { CartItem, Customer, Product } from "$lib/types/index.js";

let items = $state<CartItem[]>([]);
let customer = $state<Customer | null>(null);
let draftId = $state<number | null>(null);
let notes = $state("");

export function getOrderStore() {
	const subtotalCents = $derived(
		items.reduce((sum, item) => sum + item.product.sale_price_cents * item.quantity, 0)
	);

	const taxTotalCents = $derived(
		items.reduce((sum, item) => {
			const lineSubtotal = item.product.sale_price_cents * item.quantity;
			return sum + Math.round((lineSubtotal * item.product.tax_rate_bps) / 10000);
		}, 0)
	);

	const totalCents = $derived(subtotalCents + taxTotalCents);
	const itemCount = $derived(items.reduce((sum, item) => sum + item.quantity, 0));

	return {
		get items() { return items; },
		get customer() { return customer; },
		get draftId() { return draftId; },
		get notes() { return notes; },
		get subtotalCents() { return subtotalCents; },
		get taxTotalCents() { return taxTotalCents; },
		get totalCents() { return totalCents; },
		get itemCount() { return itemCount; },

		addItem(product: Product) {
			const existing = items.find((item) => item.product.id === product.id);
			if (existing) {
				existing.quantity += 1;
			} else {
				items.push({ product, quantity: 1, notes: "" });
			}
		},

		removeItem(productId: number) {
			items = items.filter((item) => item.product.id !== productId);
		},

		updateQuantity(productId: number, quantity: number) {
			const item = items.find((i) => i.product.id === productId);
			if (item) {
				if (quantity <= 0) {
					items = items.filter((i) => i.product.id !== productId);
				} else {
					item.quantity = quantity;
				}
			}
		},

		setCustomer(c: Customer | null) {
			customer = c;
		},

		setNotes(n: string) {
			notes = n;
		},

		setDraftId(id: number | null) {
			draftId = id;
		},

		clear() {
			items = [];
			customer = null;
			draftId = null;
			notes = "";
		}
	};
}

export const orderStore = getOrderStore();
