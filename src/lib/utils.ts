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

export function resizeImage(dataUrl: string, maxWidth: number): Promise<string> {
	return new Promise((resolve) => {
		const img = new Image();
		img.onload = () => {
			let { width, height } = img;
			if (width > maxWidth) {
				height = Math.round((height * maxWidth) / width);
				width = maxWidth;
			}
			const canvas = document.createElement("canvas");
			canvas.width = width;
			canvas.height = height;
			const ctx = canvas.getContext("2d")!;
			ctx.drawImage(img, 0, 0, width, height);
			resolve(canvas.toDataURL("image/jpeg", 0.85));
		};
		img.src = dataUrl;
	});
}

export function readFileAsDataUrl(file: File, maxWidth: number): Promise<string> {
	return new Promise((resolve, reject) => {
		const reader = new FileReader();
		reader.onload = async () => {
			try {
				const result = await resizeImage(reader.result as string, maxWidth);
				resolve(result);
			} catch (e) {
				reject(e);
			}
		};
		reader.onerror = () => reject(reader.error);
		reader.readAsDataURL(file);
	});
}
