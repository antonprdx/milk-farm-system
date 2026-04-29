import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals }) => {
	return {
		serverAuth: {
			role: locals.role,
			mustChangePassword: locals.mustChangePassword,
		},
	};
};
