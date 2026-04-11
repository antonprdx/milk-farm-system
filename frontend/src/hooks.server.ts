import type { Handle } from '@sveltejs/kit';

function decodeJwtPayload(token: string): Record<string, unknown> | null {
	try {
		const parts = token.split('.');
		if (parts.length !== 3) return null;
		const payload = parts[1];
		const decoded = atob(payload.replace(/-/g, '+').replace(/_/g, '/'));
		return JSON.parse(decoded);
	} catch {
		return null;
	}
}

export const handle: Handle = async ({ event, resolve }) => {
	const tokenCookie = event.cookies.get('token');

	if (tokenCookie) {
		const payload = decodeJwtPayload(tokenCookie);
		if (payload && payload.exp && (payload.exp as number) * 1000 > Date.now()) {
			event.locals.authenticated = true;
			event.locals.role = (payload.role as string) ?? null;
			event.locals.mustChangePassword = (payload.must_change_password as boolean) ?? false;
		} else {
			event.locals.authenticated = false;
			event.locals.role = null;
			event.locals.mustChangePassword = false;
		}
	} else {
		event.locals.authenticated = false;
		event.locals.role = null;
		event.locals.mustChangePassword = false;
	}

	const path = event.url.pathname;
	const isAuthRoute = path.startsWith('/auth');
	const isApiRoute = path.startsWith('/api');
	const isStaticAsset = path.startsWith('/_app') || path.includes('.');

	if (!event.locals.authenticated && !isAuthRoute && !isApiRoute && !isStaticAsset) {
		return new Response(null, {
			status: 302,
			headers: { location: '/auth/login' },
		});
	}

	if (event.locals.authenticated && isAuthRoute && path !== '/auth/login') {
		return new Response(null, {
			status: 302,
			headers: { location: '/' },
		});
	}

	return resolve(event);
};
