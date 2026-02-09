// ---- Users ----
export interface User {
	id: number;
	uuid: string;
	name: string;
	email: string | null;
	pin_hash: string;
	role: "admin" | "cashier";
	is_active: boolean;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
}

// ---- Categories ----
export interface Category {
	id: number;
	uuid: string;
	name: string;
	description: string | null;
	color: string;
	icon: string | null;
	sort_order: number;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
}

// ---- Products ----
export interface Product {
	id: number;
	uuid: string;
	name: string;
	description: string | null;
	sku: string | null;
	barcode: string | null;
	category_id: number | null;
	cost_price_cents: number;
	sale_price_cents: number;
	tax_rate_bps: number;
	stock_quantity: number;
	low_stock_threshold: number;
	image_path: string | null;
	is_active: boolean;
	sort_order: number;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
}

// ---- Customers ----
export interface Customer {
	id: number;
	uuid: string;
	first_name: string;
	last_name: string;
	email: string | null;
	phone: string | null;
	billing_address_line1: string | null;
	billing_address_line2: string | null;
	billing_city: string | null;
	billing_state: string | null;
	billing_zip: string | null;
	shipping_address_line1: string | null;
	shipping_address_line2: string | null;
	shipping_city: string | null;
	shipping_state: string | null;
	shipping_zip: string | null;
	notes: string | null;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
}

// ---- Orders ----
export type OrderStatus = "draft" | "completed" | "refunded" | "void";

export interface Order {
	id: number;
	uuid: string;
	order_number: string;
	status: OrderStatus;
	customer_id: number | null;
	user_id: number;
	subtotal_cents: number;
	discount_cents: number;
	tax_total_cents: number;
	total_cents: number;
	notes: string | null;
	completed_at: string | null;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
}

// ---- Order Items ----
export interface OrderItem {
	id: number;
	uuid: string;
	order_id: number;
	product_id: number | null;
	product_name: string;
	product_sku: string | null;
	quantity: number;
	unit_price_cents: number;
	tax_rate_bps: number;
	line_subtotal_cents: number;
	line_tax_cents: number;
	line_total_cents: number;
	notes: string | null;
	created_at: string;
	updated_at: string;
}

// ---- Payments ----
export type PaymentMethod = "cash" | "check" | "credit_card" | "other";

export interface Payment {
	id: number;
	uuid: string;
	order_id: number;
	method: PaymentMethod;
	amount_cents: number;
	change_cents: number;
	reference_number: string | null;
	created_at: string;
	updated_at: string;
}

// ---- Refunds ----
export interface Refund {
	id: number;
	uuid: string;
	order_id: number;
	user_id: number;
	total_refund_cents: number;
	reason: string | null;
	created_at: string;
	updated_at: string;
}

export interface RefundItem {
	id: number;
	uuid: string;
	refund_id: number;
	order_item_id: number;
	quantity: number;
	refund_amount_cents: number;
	restock: boolean;
	created_at: string;
	updated_at: string;
}

// ---- Settings ----
export type SettingValueType = "string" | "integer" | "boolean" | "json";

export interface Setting {
	id: number;
	key: string;
	value: string;
	value_type: SettingValueType;
	group_name: string;
	description: string | null;
	created_at: string;
	updated_at: string;
}

// ---- POS-specific types ----
export interface CartItem {
	product: Product;
	quantity: number;
	notes: string;
}

export interface NavItem {
	label: string;
	href: string;
	icon: string;
}

export interface NavGroup {
	label: string;
	items: NavItem[];
}
