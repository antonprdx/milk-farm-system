import { writable } from 'svelte/store';

export interface Toast {
	id: number;
	message: string;
	type: 'success' | 'error' | 'warning' | 'info';
	details?: string[];
}

let nextId = 0;

function createToastStore() {
	const { subscribe, update } = writable<Toast[]>([]);

	function add(message: string, type: Toast['type'], duration: number, details?: string[]) {
		const id = nextId++;
		update((t) => [...t, { id, message, type, details }]);
		setTimeout(() => update((t) => t.filter((x) => x.id !== id)), duration);
	}

	return {
		subscribe,
		success(message: string) {
			add(message, 'success', 3000);
		},
		error(message: string, details?: string[]) {
			add(message, 'error', 6000, details);
		},
		warning(message: string, details?: string[]) {
			add(message, 'warning', 5000, details);
		},
		info(message: string) {
			add(message, 'info', 4000);
		},
		dismiss(id: number) {
			update((t) => t.filter((x) => x.id !== id));
		},
	};
}

export const toasts = createToastStore();
