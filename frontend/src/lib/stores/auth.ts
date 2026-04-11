import { writable } from 'svelte/store';
import { browser } from '$app/environment';

interface AuthState {
	username: string | null;
	role: string | null;
	mustChangePassword: boolean;
	authenticated: boolean;
}

function createAuthStore() {
	const initial: AuthState = browser
		? {
				username: localStorage.getItem('username'),
				role: localStorage.getItem('role'),
				mustChangePassword: localStorage.getItem('mustChangePassword') === 'true',
				authenticated: !!localStorage.getItem('username'),
			}
		: { username: null, role: null, mustChangePassword: false, authenticated: false };

	const { subscribe, set, update } = writable<AuthState>(initial);

	return {
		subscribe,
		login(username: string, role: string, mustChangePassword = false) {
			if (browser) {
				localStorage.setItem('username', username);
				localStorage.setItem('role', role);
				localStorage.setItem('mustChangePassword', String(mustChangePassword));
			}
			set({ username, role, mustChangePassword, authenticated: true });
		},
		logout() {
			if (browser) {
				localStorage.removeItem('username');
				localStorage.removeItem('role');
				localStorage.removeItem('mustChangePassword');
			}
			set({ username: null, role: null, mustChangePassword: false, authenticated: false });
		},
		clearMustChangePassword() {
			if (browser) {
				localStorage.removeItem('mustChangePassword');
			}
			update((v) => ({ ...v, mustChangePassword: false }));
		},
		get authenticated() {
			let val = false;
			subscribe((v) => (val = v.authenticated))();
			return val;
		},
	};
}

export const auth = createAuthStore();
