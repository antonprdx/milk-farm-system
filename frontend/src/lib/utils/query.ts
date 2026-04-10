export function buildQuery(obj: object): string {
	const params = new URLSearchParams();
	for (const [key, value] of Object.entries(obj)) {
		if (value !== undefined && value !== '') {
			params.set(key, String(value));
		}
	}
	const qs = params.toString();
	return qs ? '?' + qs : '';
}
