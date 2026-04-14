import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const base = '/api/v1';
	try {
		const [kpiRes, alertsRes, trendRes, reproRes, feedRes, milkRes, vetFollowUpsRes, withdrawalsRes, overdueTasksRes, prefsRes] = await Promise.allSettled([
			fetch(`${base}/analytics/kpi`, { credentials: 'include' }),
			fetch(`${base}/analytics/alerts`, { credentials: 'include' }),
			fetch(`${base}/analytics/milk-trend?days=30&forecast_days=14`, { credentials: 'include' }),
			fetch(`${base}/analytics/reproduction-forecast`, { credentials: 'include' }),
			fetch(`${base}/analytics/feed-forecast`, { credentials: 'include' }),
			fetch(`${base}/analytics/latest-milk`, { credentials: 'include' }),
			fetch(`${base}/vet/follow-ups?days=7`, { credentials: 'include' }),
			fetch(`${base}/vet/withdrawals`, { credentials: 'include' }),
			fetch(`${base}/tasks?overdue=true`, { credentials: 'include' }),
			fetch(`${base}/settings/preferences`, { credentials: 'include' }),
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
		const vetFollowUps =
			vetFollowUpsRes.status === 'fulfilled' && vetFollowUpsRes.value.ok ? await vetFollowUpsRes.value.json() : { data: [] };
		const activeWithdrawals =
			withdrawalsRes.status === 'fulfilled' && withdrawalsRes.value.ok ? await withdrawalsRes.value.json() : { data: [] };
		const overdueTasks =
			overdueTasksRes.status === 'fulfilled' && overdueTasksRes.value.ok ? await overdueTasksRes.value.json() : { data: [] };
		const prefs =
			prefsRes.status === 'fulfilled' && prefsRes.value.ok ? await prefsRes.value.json() : null;

		return {
			initialData: {
				kpi,
				alerts: alertsRaw?.alerts ?? [],
				trend,
				repro,
				feed,
				latestMilk,
				vetFollowUps: vetFollowUps?.data ?? [],
				activeWithdrawals: activeWithdrawals?.data ?? [],
				overdueTasks: overdueTasks?.data ?? [],
			},
			dashboardWidgets: prefs?.dashboard_widgets ?? null,
		};
	} catch {
		return { initialData: null, error: 'Не удалось подключиться к серверу', dashboardWidgets: null };
	}
};
