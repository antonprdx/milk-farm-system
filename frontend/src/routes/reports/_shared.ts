export let thCls =
	'px-3 py-2 text-left text-xs font-medium text-slate-500 dark:text-slate-400 uppercase tracking-wider';
export let tdCls = 'px-3 py-2 text-sm text-slate-700 dark:text-slate-300 whitespace-nowrap';
export let tblCls = 'min-w-full divide-y divide-slate-200 dark:divide-slate-700';
export let badgeRed =
	'px-1.5 py-0.5 text-xs rounded bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400';
export let badgeYellow =
	'px-1.5 py-0.5 text-xs rounded bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400';
export let badgeGreen =
	'px-1.5 py-0.5 text-xs rounded bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400';

export function statusBadge(status: string) {
	if (status === 'critical') return badgeRed;
	if (status === 'warning') return badgeYellow;
	return badgeGreen;
}

export type TabId =
	| 'summary'
	| 'herd'
	| 'rest-feed'
	| 'robot'
	| 'failed'
	| 'udder-work'
	| 'udder-analyze'
	| 'milk-time'
	| 'visit'
	| 'calendar'
	| 'health-act'
	| 'efficiency'
	| 'lactation'
	| 'feed-type'
	| 'feed-cow'
	| 'health-task'
	| 'pregnancy'
	| 'transition';

export interface TabDef {
	id: TabId;
	label: string;
	group: string;
}

export let tabs: TabDef[] = [
	{ id: 'summary', label: 'Сводка', group: 'Общие' },
	{ id: 'herd', label: 'R16 Обзор стада', group: 'Стадо' },
	{ id: 'rest-feed', label: 'R18 Остаток корма', group: 'Стадо' },
	{ id: 'robot', label: 'R56 Робот', group: 'Доение' },
	{ id: 'failed', label: 'R13 Неудачные доения', group: 'Доение' },
	{ id: 'udder-work', label: 'R12 Здоровье вымени', group: 'Здоровье' },
	{ id: 'udder-analyze', label: 'R23 Анализ вымени', group: 'Здоровье' },
	{ id: 'health-task', label: 'Здоровье (sick chance)', group: 'Здоровье' },
	{ id: 'health-act', label: 'R24 Активность/жвачка', group: 'Здоровье' },
	{ id: 'transition', label: 'Транзитный период', group: 'Здоровье' },
	{ id: 'milk-time', label: 'R20 Надой по времени', group: 'Аналитика' },
	{ id: 'visit', label: 'R35 Визиты', group: 'Аналитика' },
	{ id: 'calendar', label: 'R31-34 Календарь', group: 'Воспр.' },
	{ id: 'efficiency', label: 'R41 Эффективность', group: 'Аналитика' },
	{ id: 'lactation', label: 'R52 Лактация', group: 'Аналитика' },
	{ id: 'feed-type', label: 'R70 Корм по типам', group: 'Кормление' },
	{ id: 'feed-cow', label: 'R72 Корм на корову', group: 'Кормление' },
	{ id: 'pregnancy', label: 'Коэфф. стельности', group: 'Воспр.' },
];

export const noFilterTabs: TabId[] = [
	'udder-work',
	'udder-analyze',
	'calendar',
	'health-act',
	'efficiency',
	'lactation',
	'health-task',
	'pregnancy',
	'transition',
];

export const tabExportType: Record<string, string> = {
	herd: 'herd-overview',
	'rest-feed': 'rest-feed',
	robot: 'robot-performance',
	failed: 'failed-milkings',
	'udder-work': 'udder-health-worklist',
	'udder-analyze': 'udder-health-analyze',
	'milk-time': 'milk-day-production-time',
	visit: 'visit-behavior',
	calendar: 'calendar',
	'health-act': 'health-activity-rumination',
	efficiency: 'cow-robot-efficiency',
	lactation: 'lactation-analysis',
	'feed-type': 'feed-per-type-day',
	'feed-cow': 'feed-per-cow-day',
	'health-task': 'health-task',
	pregnancy: 'pregnancy-rate',
	transition: 'transition',
};

export function groupedTabs(): { group: string; items: TabDef[] }[] {
	const groups: { group: string; items: TabDef[] }[] = [];
	for (const t of tabs) {
		const existing = groups.find((g) => g.group === t.group);
		if (existing) {
			existing.items.push(t);
		} else {
			groups.push({ group: t.group, items: [t] });
		}
	}
	return groups;
}
