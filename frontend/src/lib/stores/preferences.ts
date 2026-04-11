import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';
import type { UserPreferences } from '$lib/api/settings';
import { getPreferences, updatePreferences } from '$lib/api/settings';

const defaults: UserPreferences = {
	theme: 'light',
	page_size: 20,
	compact_view: false,
	language: 'ru',
};

function createPreferencesStore() {
	const store = writable<UserPreferences>(
		browser
			? ({
					theme: localStorage.getItem('prefs.theme') || defaults.theme,
					page_size: parseInt(localStorage.getItem('prefs.page_size') || '20'),
					compact_view: localStorage.getItem('prefs.compact_view') === 'true',
					language: localStorage.getItem('prefs.language') || defaults.language,
				} as UserPreferences)
			: defaults,
	);

	return {
		subscribe: store.subscribe,
		get: () => get(store),
		set: (prefs: UserPreferences) => {
			if (browser) {
				localStorage.setItem('prefs.theme', prefs.theme);
				localStorage.setItem('prefs.page_size', prefs.page_size.toString());
				localStorage.setItem('prefs.compact_view', prefs.compact_view.toString());
				localStorage.setItem('prefs.language', prefs.language);
			}
			store.set(prefs);
		},
		loadFromServer: async () => {
			try {
				const prefs = await getPreferences();
				if (browser) {
					localStorage.setItem('prefs.theme', prefs.theme);
					localStorage.setItem('prefs.page_size', prefs.page_size.toString());
					localStorage.setItem('prefs.compact_view', prefs.compact_view.toString());
					localStorage.setItem('prefs.language', prefs.language);
				}
				store.set(prefs);
			} catch (e) {
				console.warn('Failed to load preferences from server', e);
			}
		},
		update: async (partial: Partial<UserPreferences>) => {
			const current = get(store);
			const merged = { ...current, ...partial };
			store.set(merged);
			if (browser) {
				localStorage.setItem('prefs.theme', merged.theme);
				localStorage.setItem('prefs.page_size', merged.page_size.toString());
				localStorage.setItem('prefs.compact_view', merged.compact_view.toString());
				localStorage.setItem('prefs.language', merged.language);
			}
			try {
				const saved = await updatePreferences(partial);
				store.set(saved);
				if (browser) {
					localStorage.setItem('prefs.theme', saved.theme);
					localStorage.setItem('prefs.page_size', saved.page_size.toString());
					localStorage.setItem('prefs.compact_view', saved.compact_view.toString());
					localStorage.setItem('prefs.language', saved.language);
				}
			} catch (e) {
				console.warn('Failed to save preferences to server', e);
			}
		},
	};
}

export const preferences = createPreferencesStore();
