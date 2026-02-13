import { getTheme, defaultThemeId, type Theme } from "$lib/themes.js";
import { settingsStore } from "./settings.svelte.js";

function createThemeStore() {
	let currentThemeId = $state(defaultThemeId);
	let currentTheme = $state<Theme>(getTheme(defaultThemeId));

	const store = {
		get themeId() {
			return currentThemeId;
		},
		get theme() {
			return currentTheme;
		},
		setTheme(themeId: string) {
			currentThemeId = themeId;
			currentTheme = getTheme(themeId);
			applyTheme(currentTheme);
		},
		loadFromSettings() {
			const savedThemeId = settingsStore.get("color_theme");
			if (savedThemeId) {
				store.setTheme(savedThemeId);
			} else {
				store.setTheme(defaultThemeId);
			}
		}
	};

	return store;
}

function applyTheme(theme: Theme) {
	if (typeof document === "undefined") return;

	const root = document.documentElement;
	const isDark = root.classList.contains("dark");
	const colors = isDark ? theme.dark : theme.light;

	// Apply all CSS variables
	root.style.setProperty("--background", colors.background);
	root.style.setProperty("--foreground", colors.foreground);
	root.style.setProperty("--muted", colors.muted);
	root.style.setProperty("--muted-foreground", colors.mutedForeground);
	root.style.setProperty("--popover", colors.popover);
	root.style.setProperty("--popover-foreground", colors.popoverForeground);
	root.style.setProperty("--card", colors.card);
	root.style.setProperty("--card-foreground", colors.cardForeground);
	root.style.setProperty("--border", colors.border);
	root.style.setProperty("--input", colors.input);
	root.style.setProperty("--primary", colors.primary);
	root.style.setProperty("--primary-foreground", colors.primaryForeground);
	root.style.setProperty("--secondary", colors.secondary);
	root.style.setProperty("--secondary-foreground", colors.secondaryForeground);
	root.style.setProperty("--accent", colors.accent);
	root.style.setProperty("--accent-foreground", colors.accentForeground);
	root.style.setProperty("--destructive", colors.destructive);
	root.style.setProperty("--destructive-foreground", colors.destructiveForeground);
	root.style.setProperty("--ring", colors.ring);
	root.style.setProperty("--sidebar", colors.sidebar);
	root.style.setProperty("--sidebar-foreground", colors.sidebarForeground);
	root.style.setProperty("--sidebar-primary", colors.sidebarPrimary);
	root.style.setProperty("--sidebar-primary-foreground", colors.sidebarPrimaryForeground);
	root.style.setProperty("--sidebar-accent", colors.sidebarAccent);
	root.style.setProperty("--sidebar-accent-foreground", colors.sidebarAccentForeground);
	root.style.setProperty("--sidebar-border", colors.sidebarBorder);
	root.style.setProperty("--sidebar-ring", colors.sidebarRing);
}

export const themeStore = createThemeStore();

// React to mode-watcher dark mode changes
if (typeof document !== "undefined") {
	const observer = new MutationObserver(() => {
		// Get the current theme without creating reactive dependencies
		const theme = getTheme(themeStore.themeId);
		applyTheme(theme);
	});

	observer.observe(document.documentElement, {
		attributes: true,
		attributeFilter: ["class"]
	});
}
