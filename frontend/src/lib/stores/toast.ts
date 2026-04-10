import { writable } from 'svelte/store';

export interface Toast {
	id: number;
	message: string;
	type: 'success' | 'error';
}

let nextId = 0;

function createToastStore() {
	const { subscribe, update } = writable<Toast[]>([]);

	return {
		subscribe,
		success(message: string) {
			const id = nextId++;
			update((t) => [...t, { id, message, type: 'success' }]);
			setTimeout(() => update((t) => t.filter((x) => x.id !== id)), 3000);
		},
		error(message: string) {
			const id = nextId++;
			update((t) => [...t, { id, message, type: 'error' }]);
			setTimeout(() => update((t) => t.filter((x) => x.id !== id)), 5000);
		},
		dismiss(id: number) {
			update((t) => t.filter((x) => x.id !== id));
		},
	};
}

export const toasts = createToastStore();
