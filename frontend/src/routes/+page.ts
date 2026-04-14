import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const base = '/api/v1';
	try {
		const [kpiRes, alertsRes, trendRes, reproRes, feedRes, milkRes] = await Promise.allSettled([
			fetch(`${base}/analytics/kpi`, { credentials: 'include' }),
			fetch(`${base}/analytics/alerts`, { credentials: 'include' }),
			fetch(`${base}/analytics/milk-trend?days=30&forecast_days=14`, { credentials: 'include' }),
			fetch(`${base}/analytics/reproduction-forecast`, { credentials: 'include' }),
			fetch(`${base}/analytics/feed-forecast`, { credentials: 'include' }),
			fetch(`${base}/analytics/latest-milk`, { credentials: 'include' }),
		]);

		const kpi = kpiRes.status === 'fulfilled' && kpiRes.value.ok ? await kpiRes.value.json() : null;
		const alertsRaw =
			alertsRes.status === 'fulfilled' && alertsRes.value.ok ? await alertsRes.value.json() : null;
		const trend =
			trendRes.status === 'fulfilled' && trendRes.value.ok ? await trendRes.value.json() : null;
		const repro =
			reproRes.status === 'fulfilled' && reproRes.value.ok ? await reproRes.value.json() : null;
		const feed =
			feedRes.status === 'fulfilled' && feedRes.value.ok ? await feedRes.value.json() : null;
		const latestMilk =
			milkRes.status === 'fulfilled' && milkRes.value.ok ? await milkRes.value.json() : [];

		return {
			initialData: {
				kpi,
				alerts: alertsRaw?.alerts ?? [],
				trend,
				repro,
				feed,
				latestMilk,
			},
		};
	} catch {
		return { initialData: null, error: 'Не удалось подключиться к серверу' };
	}
};
