import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';

function createThemeStore() {
	const store = writable<'light' | 'dark'>(
		browser ? ((localStorage.getItem('theme') as 'light' | 'dark') ?? 'light') : 'light',
	);

	return {
		subscribe: store.subscribe,
		toggle: () => {
			const next = get(store) === 'dark' ? 'light' : 'dark';
			if (browser) {
				localStorage.setItem('theme', next);
				document.documentElement.classList.toggle('dark', next === 'dark');
			}
			store.set(next);
		},
		init: () => {
			if (browser) {
				const saved = (localStorage.getItem('theme') as 'light' | 'dark') ?? 'light';
				document.documentElement.classList.toggle('dark', saved === 'dark');
				store.set(saved);
			}
		},
	};
}

export const theme = createThemeStore();
