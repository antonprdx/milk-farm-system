import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';
import type { UserPreferences } from '$lib/api/settings';
import { getPreferences, updatePreferences } from '$lib/api/settings';

const ALL_WIDGETS_DEFAULT = [
	'kpi',
	'milk_trend',
	'alerts',
	'reproduction',
	'feed',
	'latest_milk',
	'system_status',
	'vet_followups',
	'active_withdrawals',
	'overdue_tasks',
];

const defaults: UserPreferences = {
	theme: 'light',
	page_size: 20,
	compact_view: false,
	language: 'ru',
	dashboard_widgets: ALL_WIDGETS_DEFAULT,
};

	function parseWidgets(val: string | null): string[] {
		if (!val) return ALL_WIDGETS_DEFAULT;
		try {
			const parsed = JSON.parse(val);
			if (Array.isArray(parsed)) return parsed;
		} catch {}
		return ALL_WIDGETS_DEFAULT;
	}

	function createPreferencesStore() {
		const store = writable<UserPreferences>(
			browser
				? ({
						theme: localStorage.getItem('prefs.theme') || defaults.theme,
						page_size: parseInt(localStorage.getItem('prefs.page_size') || '20'),
						compact_view: localStorage.getItem('prefs.compact_view') === 'true',
						language: localStorage.getItem('prefs.language') || defaults.language,
						dashboard_widgets: parseWidgets(localStorage.getItem('prefs.dashboard_widgets')),
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
				localStorage.setItem('prefs.dashboard_widgets', JSON.stringify(prefs.dashboard_widgets));
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
					localStorage.setItem('prefs.dashboard_widgets', JSON.stringify(prefs.dashboard_widgets));
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
				localStorage.setItem('prefs.dashboard_widgets', JSON.stringify(merged.dashboard_widgets));
			}
			try {
				const saved = await updatePreferences(partial);
				store.set(saved);
				if (browser) {
					localStorage.setItem('prefs.theme', saved.theme);
					localStorage.setItem('prefs.page_size', saved.page_size.toString());
					localStorage.setItem('prefs.compact_view', saved.compact_view.toString());
					localStorage.setItem('prefs.language', saved.language);
					localStorage.setItem('prefs.dashboard_widgets', JSON.stringify(saved.dashboard_widgets));
				}
			} catch (e) {
				console.warn('Failed to save preferences to server', e);
			}
		},
	};
}

export const preferences = createPreferencesStore();
