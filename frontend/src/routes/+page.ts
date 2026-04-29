import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	try {
		const res = await fetch('/api/v1/analytics/dashboard?forecast_days=14', {
			credentials: 'include',
		});
		if (!res.ok) {
			return { initialData: null, error: 'Не удалось подключиться к серверу', dashboardWidgets: null };
		}
		const data = await res.json();
		return {
			initialData: {
				kpi: data.kpi,
				trend: data.trend,
				repro: data.reproduction,
				feed: data.feed,
				latestMilk: data.latest_milk,
				vetFollowUps: data.vet_follow_ups ?? [],
				activeWithdrawals: data.active_withdrawals ?? [],
			},
			dashboardWidgets: data.dashboard_widgets ?? null,
		};
	} catch {
		return { initialData: null, error: 'Не удалось подключиться к серверу', dashboardWidgets: null };
	}
};
