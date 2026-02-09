import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithElementRef<T, El extends HTMLElement = any> = T & {
	ref?: El | null;
};

export type WithoutChildren<T> = T extends { children?: unknown }
	? Omit<T, "children">
	: T;

export type WithoutChildrenOrChild<T> = T extends { children?: unknown; child?: unknown }
	? Omit<T, "children" | "child">
	: T extends { children?: unknown }
		? Omit<T, "children">
		: T;

export type WithoutChild<T> = T extends { child?: unknown }
	? Omit<T, "child">
	: T;

export function formatCents(cents: number): string {
	return (cents / 100).toFixed(2);
}

export function formatCurrency(cents: number, symbol = "$"): string {
	return `${symbol}${formatCents(cents)}`;
}

export function formatBps(bps: number): string {
	return (bps / 100).toFixed(2);
}
