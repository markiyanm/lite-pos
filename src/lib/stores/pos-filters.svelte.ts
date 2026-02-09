let searchQuery = $state("");
let selectedCategoryId = $state<number | null>(null);

export function getPosFilters() {
	return {
		get searchQuery() { return searchQuery; },
		get selectedCategoryId() { return selectedCategoryId; },

		setSearch(query: string) {
			searchQuery = query;
		},

		setCategory(categoryId: number | null) {
			selectedCategoryId = categoryId;
		},

		clear() {
			searchQuery = "";
			selectedCategoryId = null;
		}
	};
}

export const posFilters = getPosFilters();
