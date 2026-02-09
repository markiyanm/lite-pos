import type { User } from "$lib/types/index.js";

let currentUser = $state<User | null>(null);

export function getSession() {
	const isAuthenticated = $derived(currentUser !== null);
	const isAdmin = $derived(currentUser?.role === "admin");

	return {
		get user() { return currentUser; },
		get isAuthenticated() { return isAuthenticated; },
		get isAdmin() { return isAdmin; },

		login(user: User) {
			currentUser = user;
		},

		logout() {
			currentUser = null;
		}
	};
}

export const session = getSession();
