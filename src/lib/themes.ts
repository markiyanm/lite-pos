export interface ThemeColors {
	background: string;
	foreground: string;
	muted: string;
	mutedForeground: string;
	popover: string;
	popoverForeground: string;
	card: string;
	cardForeground: string;
	border: string;
	input: string;
	primary: string;
	primaryForeground: string;
	secondary: string;
	secondaryForeground: string;
	accent: string;
	accentForeground: string;
	destructive: string;
	destructiveForeground: string;
	ring: string;
	sidebar: string;
	sidebarForeground: string;
	sidebarPrimary: string;
	sidebarPrimaryForeground: string;
	sidebarAccent: string;
	sidebarAccentForeground: string;
	sidebarBorder: string;
	sidebarRing: string;
}

export interface Theme {
	id: string;
	name: string;
	description: string;
	light: ThemeColors;
	dark: ThemeColors;
}

export const themes: Record<string, Theme> = {
	blue: {
		id: "blue",
		name: "Blue",
		description: "Cool blue and slate tones",
		light: {
			background: "hsl(0 0% 100%)",
			foreground: "hsl(222.2 84% 4.9%)",
			muted: "hsl(210 40% 96.1%)",
			mutedForeground: "hsl(215.4 16.3% 46.9%)",
			popover: "hsl(0 0% 100%)",
			popoverForeground: "hsl(222.2 84% 4.9%)",
			card: "hsl(0 0% 100%)",
			cardForeground: "hsl(222.2 84% 4.9%)",
			border: "hsl(214.3 31.8% 91.4%)",
			input: "hsl(214.3 31.8% 91.4%)",
			primary: "hsl(221.2 83.2% 53.3%)", // Vibrant blue
			primaryForeground: "hsl(210 40% 98%)",
			secondary: "hsl(210 40% 96.1%)",
			secondaryForeground: "hsl(222.2 47.4% 11.2%)",
			accent: "hsl(210 40% 96.1%)",
			accentForeground: "hsl(222.2 47.4% 11.2%)",
			destructive: "hsl(0 84.2% 60.2%)",
			destructiveForeground: "hsl(210 40% 98%)",
			ring: "hsl(221.2 83.2% 53.3%)",
			sidebar: "hsl(0 0% 98%)",
			sidebarForeground: "hsl(222.2 84% 4.9%)",
			sidebarPrimary: "hsl(221.2 83.2% 53.3%)",
			sidebarPrimaryForeground: "hsl(210 40% 98%)",
			sidebarAccent: "hsl(210 40% 96.1%)",
			sidebarAccentForeground: "hsl(222.2 47.4% 11.2%)",
			sidebarBorder: "hsl(214.3 31.8% 91.4%)",
			sidebarRing: "hsl(221.2 83.2% 53.3%)"
		},
		dark: {
			background: "hsl(222.2 84% 4.9%)",
			foreground: "hsl(210 40% 98%)",
			muted: "hsl(217.2 32.6% 17.5%)",
			mutedForeground: "hsl(215 20.2% 65.1%)",
			popover: "hsl(222.2 84% 4.9%)",
			popoverForeground: "hsl(210 40% 98%)",
			card: "hsl(222.2 84% 4.9%)",
			cardForeground: "hsl(210 40% 98%)",
			border: "hsl(217.2 32.6% 17.5%)",
			input: "hsl(217.2 32.6% 17.5%)",
			primary: "hsl(217.2 91.2% 59.8%)", // Lighter blue for dark mode
			primaryForeground: "hsl(222.2 47.4% 11.2%)",
			secondary: "hsl(217.2 32.6% 17.5%)",
			secondaryForeground: "hsl(210 40% 98%)",
			accent: "hsl(217.2 32.6% 17.5%)",
			accentForeground: "hsl(210 40% 98%)",
			destructive: "hsl(0 62.8% 30.6%)",
			destructiveForeground: "hsl(210 40% 98%)",
			ring: "hsl(224.3 76.3% 48%)",
			sidebar: "hsl(222.2 84% 4.9%)",
			sidebarForeground: "hsl(210 40% 98%)",
			sidebarPrimary: "hsl(217.2 91.2% 59.8%)",
			sidebarPrimaryForeground: "hsl(222.2 47.4% 11.2%)",
			sidebarAccent: "hsl(217.2 32.6% 17.5%)",
			sidebarAccentForeground: "hsl(210 40% 98%)",
			sidebarBorder: "hsl(217.2 32.6% 17.5%)",
			sidebarRing: "hsl(217.2 91.2% 59.8%)"
		}
	},
	bronze: {
		id: "bronze",
		name: "Bronze",
		description: "Warm bronze and copper tones",
		light: {
			background: "hsl(0 0% 100%)",
			foreground: "hsl(216 28% 11%)",
			muted: "hsl(220 14% 96%)",
			mutedForeground: "hsl(220 9% 46%)",
			popover: "hsl(0 0% 100%)",
			popoverForeground: "hsl(216 28% 11%)",
			card: "hsl(0 0% 100%)",
			cardForeground: "hsl(216 28% 11%)",
			border: "hsl(220 13% 91%)",
			input: "hsl(220 13% 91%)",
			primary: "hsl(18 67% 55%)", // #d87943 bronze/copper
			primaryForeground: "hsl(0 0% 100%)",
			secondary: "hsl(180 18% 32%)", // #527575 teal
			secondaryForeground: "hsl(0 0% 100%)",
			accent: "hsl(0 0% 93%)",
			accentForeground: "hsl(216 28% 11%)",
			destructive: "hsl(0 84% 60%)",
			destructiveForeground: "hsl(0 0% 98%)",
			ring: "hsl(18 67% 55%)",
			sidebar: "hsl(0 0% 98%)",
			sidebarForeground: "hsl(216 28% 11%)",
			sidebarPrimary: "hsl(18 67% 55%)",
			sidebarPrimaryForeground: "hsl(0 0% 98%)",
			sidebarAccent: "hsl(0 0% 93%)",
			sidebarAccentForeground: "hsl(216 28% 11%)",
			sidebarBorder: "hsl(220 13% 91%)",
			sidebarRing: "hsl(217.2 91.2% 59.8%)"
		},
		dark: {
			background: "hsl(240 10% 7%)",
			foreground: "hsl(0 0% 76%)",
			muted: "hsl(0 0% 13%)",
			mutedForeground: "hsl(0 0% 64%)",
			popover: "hsl(240 10% 7%)",
			popoverForeground: "hsl(0 0% 76%)",
			card: "hsl(240 10% 7%)",
			cardForeground: "hsl(0 0% 76%)",
			border: "hsl(0 0% 13%)",
			input: "hsl(0 0% 13%)",
			primary: "hsl(21 75% 60%)", // #e78a53 lighter bronze
			primaryForeground: "hsl(0 0% 100%)",
			secondary: "hsl(180 19% 44%)", // #5f8787 lighter teal
			secondaryForeground: "hsl(0 0% 100%)",
			accent: "hsl(0 0% 20%)",
			accentForeground: "hsl(0 0% 76%)",
			destructive: "hsl(180 19% 44%)",
			destructiveForeground: "hsl(0 0% 98%)",
			ring: "hsl(21 75% 60%)",
			sidebar: "hsl(240 5.9% 10%)",
			sidebarForeground: "hsl(0 0% 76%)",
			sidebarPrimary: "hsl(21 75% 60%)",
			sidebarPrimaryForeground: "hsl(0 0% 100%)",
			sidebarAccent: "hsl(0 0% 20%)",
			sidebarAccentForeground: "hsl(0 0% 76%)",
			sidebarBorder: "hsl(0 0% 13%)",
			sidebarRing: "hsl(217.2 91.2% 59.8%)"
		}
	},
	blueYellow: {
		id: "blueYellow",
		name: "Blue & Yellow",
		description: "Vibrant yellow and cyan accent colors",
		light: {
			background: "hsl(0 0% 100%)",
			foreground: "hsl(0 0% 4%)",
			muted: "hsl(240 5% 96%)",
			mutedForeground: "hsl(240 4% 46%)",
			popover: "hsl(0 0% 100%)",
			popoverForeground: "hsl(0 0% 4%)",
			card: "hsl(0 0% 100%)",
			cardForeground: "hsl(0 0% 4%)",
			border: "hsl(0 0% 90%)", // #e5e5e5
			input: "hsl(0 0% 90%)",
			primary: "hsl(48 100% 50%)", // #fdc700
			primaryForeground: "hsl(54 95% 96%)", // #fefce8
			secondary: "hsl(194 100% 50%)", // #00bcff
			secondaryForeground: "hsl(201 100% 97%)", // #f0f9ff
			accent: "hsl(0 0% 96%)", // #f5f5f5
			accentForeground: "hsl(0 0% 4%)",
			destructive: "hsl(359 100% 45%)", // #e7000b
			destructiveForeground: "hsl(0 0% 98%)",
			ring: "hsl(0 0% 63%)", // #a1a1a1
			sidebar: "hsl(0 0% 98%)",
			sidebarForeground: "hsl(0 0% 4%)",
			sidebarPrimary: "hsl(48 100% 50%)",
			sidebarPrimaryForeground: "hsl(54 95% 96%)",
			sidebarAccent: "hsl(0 0% 96%)",
			sidebarAccentForeground: "hsl(0 0% 4%)",
			sidebarBorder: "hsl(0 0% 90%)",
			sidebarRing: "hsl(194 100% 50%)"
		},
		dark: {
			background: "hsl(0 0% 4%)",
			foreground: "hsl(0 0% 98%)",
			muted: "hsl(240 4% 16%)",
			mutedForeground: "hsl(240 5% 65%)",
			popover: "hsl(0 0% 4%)",
			popoverForeground: "hsl(0 0% 98%)",
			card: "hsl(0 0% 4%)",
			cardForeground: "hsl(0 0% 98%)",
			border: "hsl(0 0% 16%)", // #282828
			input: "hsl(0 0% 16%)",
			primary: "hsl(49 100% 56%)", // #ffdf20
			primaryForeground: "hsl(33 90% 14%)", // #432004
			secondary: "hsl(197 100% 73%)", // #74d4ff
			secondaryForeground: "hsl(201 88% 15%)", // #052f4a
			accent: "hsl(0 0% 25%)", // #404040
			accentForeground: "hsl(0 0% 98%)",
			destructive: "hsl(359 100% 69%)", // #ff6467
			destructiveForeground: "hsl(0 0% 98%)",
			ring: "hsl(0 0% 84%)",
			sidebar: "hsl(240 5.9% 10%)",
			sidebarForeground: "hsl(0 0% 98%)",
			sidebarPrimary: "hsl(49 100% 56%)",
			sidebarPrimaryForeground: "hsl(33 90% 14%)",
			sidebarAccent: "hsl(0 0% 25%)",
			sidebarAccentForeground: "hsl(0 0% 98%)",
			sidebarBorder: "hsl(0 0% 16%)",
			sidebarRing: "hsl(197 100% 73%)"
		}
	},
	goldBrown: {
		id: "goldBrown",
		name: "Gold & Brown",
		description: "Warm earthy tones with gold accents",
		light: {
			background: "hsl(34 50% 98%)", // #faf8f6
			foreground: "hsl(27 44% 8%)", // #1a1410
			muted: "hsl(35 21% 93%)", // #f0ebe4
			mutedForeground: "hsl(40 26% 25%)",
			popover: "hsl(34 50% 98%)",
			popoverForeground: "hsl(27 44% 8%)",
			card: "hsl(34 50% 98%)",
			cardForeground: "hsl(27 44% 8%)",
			border: "hsl(32 23% 82%)", // #d6cfc6
			input: "hsl(32 23% 82%)",
			primary: "hsl(42 94% 33%)", // #a16207
			primaryForeground: "hsl(54 95% 96%)", // #fefce8
			secondary: "hsl(35 21% 93%)", // #f0ebe4
			secondaryForeground: "hsl(40 53% 22%)", // #5c4620
			accent: "hsl(47 89% 91%)", // #fef3c7
			accentForeground: "hsl(33 86% 31%)", // #92400e
			destructive: "hsl(0 68% 42%)", // #b91c1c
			destructiveForeground: "hsl(0 86% 97%)", // #fef2f2
			ring: "hsl(42 94% 33%)",
			sidebar: "hsl(34 31% 95%)", // #f5f0eb
			sidebarForeground: "hsl(27 44% 8%)",
			sidebarPrimary: "hsl(42 94% 33%)",
			sidebarPrimaryForeground: "hsl(54 95% 96%)",
			sidebarAccent: "hsl(47 89% 91%)",
			sidebarAccentForeground: "hsl(33 86% 31%)",
			sidebarBorder: "hsl(32 23% 82%)",
			sidebarRing: "hsl(42 94% 33%)"
		},
		dark: {
			background: "hsl(30 57% 2%)", // #080604
			foreground: "hsl(36 30% 87%)", // #e8e0d6
			muted: "hsl(33 30% 13%)", // #211c14
			mutedForeground: "hsl(35 16% 65%)",
			popover: "hsl(30 57% 2%)",
			popoverForeground: "hsl(36 30% 87%)",
			card: "hsl(30 57% 2%)",
			cardForeground: "hsl(36 30% 87%)",
			border: "hsl(32 27% 14%)", // #2a231a
			input: "hsl(32 27% 14%)",
			primary: "hsl(36 92% 49%)", // #d97706
			primaryForeground: "hsl(33 89% 6%)", // #1c1004
			secondary: "hsl(33 30% 13%)", // #211c14
			secondaryForeground: "hsl(35 18% 64%)", // #b8a88e
			accent: "hsl(40 43% 13%)", // #2c2010
			accentForeground: "hsl(45 97% 57%)", // #fbbf24
			destructive: "hsl(0 84% 60%)", // #ef4444
			destructiveForeground: "hsl(0 90% 5%)", // #1a0606
			ring: "hsl(36 92% 49%)",
			sidebar: "hsl(30 43% 3%)", // #0a0806
			sidebarForeground: "hsl(36 30% 87%)",
			sidebarPrimary: "hsl(36 92% 49%)",
			sidebarPrimaryForeground: "hsl(33 89% 6%)",
			sidebarAccent: "hsl(40 43% 13%)",
			sidebarAccentForeground: "hsl(45 97% 57%)",
			sidebarBorder: "hsl(32 27% 14%)",
			sidebarRing: "hsl(36 92% 49%)"
		}
	},
	default: {
		id: "default",
		name: "Dark",
		description: "Classic neutral gray theme (original)",
		light: {
			background: "hsl(0 0% 100%)",
			foreground: "hsl(240 10% 3.9%)",
			muted: "hsl(240 4.8% 95.9%)",
			mutedForeground: "hsl(240 3.8% 46.1%)",
			popover: "hsl(0 0% 100%)",
			popoverForeground: "hsl(240 10% 3.9%)",
			card: "hsl(0 0% 100%)",
			cardForeground: "hsl(240 10% 3.9%)",
			border: "hsl(240 5.9% 90%)",
			input: "hsl(240 5.9% 90%)",
			primary: "hsl(240 5.9% 10%)",
			primaryForeground: "hsl(0 0% 98%)",
			secondary: "hsl(240 4.8% 95.9%)",
			secondaryForeground: "hsl(240 5.9% 10%)",
			accent: "hsl(240 4.8% 95.9%)",
			accentForeground: "hsl(240 5.9% 10%)",
			destructive: "hsl(0 72.2% 50.6%)",
			destructiveForeground: "hsl(0 0% 98%)",
			ring: "hsl(240 10% 3.9%)",
			sidebar: "hsl(0 0% 98%)",
			sidebarForeground: "hsl(240 5.3% 26.1%)",
			sidebarPrimary: "hsl(240 5.9% 10%)",
			sidebarPrimaryForeground: "hsl(0 0% 98%)",
			sidebarAccent: "hsl(240 4.8% 95.9%)",
			sidebarAccentForeground: "hsl(240 5.9% 10%)",
			sidebarBorder: "hsl(220 13% 91%)",
			sidebarRing: "hsl(217.2 91.2% 59.8%)"
		},
		dark: {
			background: "hsl(240 10% 3.9%)",
			foreground: "hsl(0 0% 98%)",
			muted: "hsl(240 3.7% 15.9%)",
			mutedForeground: "hsl(240 5% 64.9%)",
			popover: "hsl(240 10% 3.9%)",
			popoverForeground: "hsl(0 0% 98%)",
			card: "hsl(240 10% 3.9%)",
			cardForeground: "hsl(0 0% 98%)",
			border: "hsl(240 3.7% 15.9%)",
			input: "hsl(240 3.7% 15.9%)",
			primary: "hsl(0 0% 98%)",
			primaryForeground: "hsl(240 5.9% 10%)",
			secondary: "hsl(240 3.7% 15.9%)",
			secondaryForeground: "hsl(0 0% 98%)",
			accent: "hsl(240 3.7% 15.9%)",
			accentForeground: "hsl(0 0% 98%)",
			destructive: "hsl(0 62.8% 30.6%)",
			destructiveForeground: "hsl(0 0% 98%)",
			ring: "hsl(240 4.9% 83.9%)",
			sidebar: "hsl(240 5.9% 10%)",
			sidebarForeground: "hsl(240 4.8% 95.9%)",
			sidebarPrimary: "hsl(224.3 76.3% 48%)",
			sidebarPrimaryForeground: "hsl(0 0% 100%)",
			sidebarAccent: "hsl(240 3.7% 15.9%)",
			sidebarAccentForeground: "hsl(240 4.8% 95.9%)",
			sidebarBorder: "hsl(240 3.7% 15.9%)",
			sidebarRing: "hsl(217.2 91.2% 59.8%)"
		}
	}
};

export const defaultThemeId = "blue";

export function getTheme(id: string): Theme {
	return themes[id] || themes[defaultThemeId];
}

export function getAllThemes(): Theme[] {
	return Object.values(themes);
}
