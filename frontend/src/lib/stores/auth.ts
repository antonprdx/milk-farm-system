import { writable } from 'svelte/store';
import { browser } from '$app/environment';

interface AuthState {
	username: string | null;
	role: string | null;
	mustChangePassword: boolean;
	authenticated: boolean;
	initialized: boolean;
}

function createAuthStore() {
	const initial: AuthState = {
		username: null,
		role: null,
		mustChangePassword: false,
		authenticated: false,
		initialized: false,
	};

	const { subscribe, set, update } = writable<AuthState>(initial);

	return {
		subscribe,
		async init() {
			if (!browser) return;
			try {
				const res = await fetch('/api/v1/auth/refresh', {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					credentials: 'include',
				});
				if (res.ok) {
					const data = await res.json();
					set({
						username: data.username,
						role: data.role,
						mustChangePassword: data.must_change_password,
						authenticated: true,
						initialized: true,
					});
				} else {
					set({ username: null, role: null, mustChangePassword: false, authenticated: false, initialized: true });
				}
			} catch {
				set({ username: null, role: null, mustChangePassword: false, authenticated: false, initialized: true });
			}
		},
		login(username: string, role: string, mustChangePassword = false) {
			set({ username, role, mustChangePassword, authenticated: true, initialized: true });
		},
		logout() {
			set({ username: null, role: null, mustChangePassword: false, authenticated: false, initialized: true });
		},
		clearMustChangePassword() {
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
