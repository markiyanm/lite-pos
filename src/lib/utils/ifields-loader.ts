/**
 * iFields CDN script loader - singleton pattern.
 * Loads the Cardknox iFields JS SDK once per app session.
 */

const IFIELDS_CDN_URL = "https://cdn.cardknox.com/ifields/3.4.2602.2001/ifields.min.js";
export const IFIELDS_IFRAME_URL = "https://cdn.cardknox.com/ifields/3.4.2602.2001/ifield.htm";

let loadPromise: Promise<void> | null = null;
let loaded = false;
let loadError: string | null = null;

/**
 * Load the iFields script from CDN. Returns a promise that resolves when ready.
 * Subsequent calls return the same promise (singleton).
 */
export function loadIfields(): Promise<void> {
	if (loadPromise) return loadPromise;

	loadPromise = new Promise<void>((resolve, reject) => {
		// Check if already loaded (e.g. from a previous session in dev)
		if (typeof (window as any).setAccount === "function") {
			loaded = true;
			resolve();
			return;
		}

		const script = document.createElement("script");
		script.src = IFIELDS_CDN_URL;
		script.async = true;

		script.onload = () => {
			loaded = true;
			loadError = null;
			resolve();
		};

		script.onerror = () => {
			loadError = "Failed to load card entry form. Please check your internet connection.";
			loadPromise = null; // Allow retry
			reject(new Error(loadError));
		};

		document.head.appendChild(script);
	});

	return loadPromise;
}

/**
 * Whether the iFields script has been loaded successfully.
 */
export function isIfieldsLoaded(): boolean {
	return loaded;
}

/**
 * Get the last load error message, if any.
 */
export function getIfieldsLoadError(): string | null {
	return loadError;
}

/**
 * Reset loader state (for testing or retry after error).
 */
export function resetIfieldsLoader(): void {
	loadPromise = null;
	loaded = false;
	loadError = null;
}

// Declare global iFields functions for TypeScript
declare global {
	function setAccount(ifieldKey: string, softwareName: string, softwareVersion: string): void;
	function setIfieldStyle(ifieldId: string, style: string): void;
	function enableAutoFormatting(separator: string): void;
	function addIfieldKeyPressCallback(callback: (data: IfieldsKeyPressData) => void): void;
	function getTokens(
		onSuccess: () => void,
		onError: (error: IfieldsError) => void,
		timeoutMs: number
	): void;
	function focusIfield(ifieldId: string): void;
	function clearIfield(ifieldId: string): void;

	interface IfieldsKeyPressData {
		cardNumberFormattedLength: number;
		cardNumberIsValid: boolean;
		cvvIsValid: boolean;
		issuer: string;
		ifieldValueChanged: boolean;
		lastIfieldChanged: string;
	}

	interface IfieldsError {
		xTokenType: string;
		errorMessage: string;
	}
}
