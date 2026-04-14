<script lang="ts">
	import { onMount } from 'svelte';
	import {
		Chart,
		LineController,
		CategoryScale,
		LinearScale,
		PointElement,
		LineElement,
		Filler,
		Tooltip,
		Legend,
	} from 'chart.js';
	import { theme } from '$lib/stores/theme';
	import { defaultTooltip, defaultScales, themeColors } from '$lib/utils/chartHelpers';
	import { debounce } from '$lib/utils/debounce';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import {
		Beef,
		Milk,
		Heart,
		Wheat,
		Users,
		RefreshCw,
		Activity,
		Database,
		Server,
		CircleAlert,
		Plug,
		Settings,
		X,
	} from 'lucide-svelte';
	import {
		getKpi,
		getAlerts,
		getMilkTrend,
		getReproductionForecast,
		getFeedForecast,
		getLatestMilk,
		type KpiResponse,
		type Alert,
		type MilkTrendResponse,
		type ReproductionForecastResponse,
		type FeedForecastResponse,
		type LatestMilkEntry,
	} from '$lib/api/analytics';
	import {
		getUpcomingFollowUps,
		getActiveWithdrawals,
		VET_RECORD_TYPE_LABELS,
		type VetRecord,
	} from '$lib/api/vet';
	import {
		listTasks,
		TASK_PRIORITY_LABELS,
		type Task,
		type TaskPriority,
	} from '$lib/api/tasks';
	import { getPreferences, updatePreferences } from '$lib/api/settings';

	Chart.register(
		LineController,
		CategoryScale,
		LinearScale,
		PointElement,
		LineElement,
		Filler,
		Tooltip,
		Legend,
	);

	interface SystemStatus {
		api: 'ok' | 'error' | 'checking';
		db: 'ok' | 'error' | 'checking';
	}

	interface LelyStatus {
		enabled: boolean;
		connected?: boolean;
		last_sync?: string;
	}

	const ALL_WIDGETS = [
		{ id: 'kpi', label: 'KPI показатели' },
		{ id: 'alerts', label: 'Предупреждения' },
		{ id: 'milk_trend', label: 'Тренд надоя' },
		{ id: 'latest_milk', label: 'Последние надои' },
		{ id: 'reproduction', label: 'Воспроизводство' },
		{ id: 'feed', label: 'Прогноз корма' },
		{ id: 'system_status', label: 'Статус системы' },
		{ id: 'vet_followups', label: 'Ветеринарные повторные приёмы' },
		{ id: 'active_withdrawals', label: 'Активные периоды ожидания' },
		{ id: 'overdue_tasks', label: 'Просроченные задачи' },
	] as const;

	type WidgetId = (typeof ALL_WIDGETS)[number]['id'];

	let { data } = $props();

	let systemStatus = $state<SystemStatus>({ api: 'checking', db: 'checking' });
	let lelyStatus = $state<LelyStatus | null>(null);

	let error = $state('');
	let loading = $state(true);
	let kpi = $state<KpiResponse | null>(null);
	let alerts = $state<Alert[]>([]);
	let trend = $state<MilkTrendResponse | null>(null);
	let repro = $state<ReproductionForecastResponse | null>(null);
	let feed = $state<FeedForecastResponse | null>(null);
	let trendCanvas: HTMLCanvasElement | undefined = $state();
	let trendChart: Chart | null = null;
	let latestMilk = $state<LatestMilkEntry[]>([]);
	let vetFollowUps = $state<VetRecord[]>([]);
	let activeWithdrawals = $state<VetRecord[]>([]);
	let overdueTasks = $state<Task[]>([]);

	let widgets = $state<WidgetId[]>([
		'kpi',
		'milk_trend',
		'alerts',
		'reproduction',
		'feed',
		'latest_milk',
		'system_status',
		'vet_followups',
		'active_withdrawals',
		'overdue_tasks',
	]);
	let showSettings = $state(false);
	let settingsSaving = $state(false);

	if (data.error) error = data.error;

	if (data.initialData) {
		kpi = data.initialData.kpi;
		alerts = data.initialData.alerts;
		trend = data.initialData.trend;
		repro = data.initialData.repro;
		feed = data.initialData.feed;
		latestMilk = data.initialData.latestMilk ?? [];
		vetFollowUps = data.initialData.vetFollowUps ?? [];
		activeWithdrawals = data.initialData.activeWithdrawals ?? [];
		overdueTasks = data.initialData.overdueTasks ?? [];
		loading = false;
	}

	if (data.dashboardWidgets && Array.isArray(data.dashboardWidgets)) {
		widgets = data.dashboardWidgets;
	}

	function hasWidget(id: WidgetId): boolean {
		return widgets.includes(id);
	}

	function toggleWidget(id: WidgetId) {
		if (widgets.includes(id)) {
			widgets = widgets.filter((w) => w !== id);
		} else {
			widgets = [...widgets, id];
		}
		saveWidgets();
	}

	async function saveWidgets() {
		settingsSaving = true;
		try {
			await updatePreferences({ dashboard_widgets: widgets } as any);
		} catch {
		} finally {
			settingsSaving = false;
		}
	}

	async function checkHealth() {
		systemStatus = { api: 'checking', db: 'checking' };
		try {
			const base = import.meta.env.VITE_API_BASE || '/api/v1';
			const res = await fetch(`${base}/health`, {
				credentials: 'include',
			});
			if (res.ok) {
				const data = await res.json();
				systemStatus = {
					api: 'ok',
					db: data.db === 'ok' ? 'ok' : 'error',
				};
				lelyStatus = data.lely || null;
			} else {
				systemStatus = { api: 'error', db: 'error' };
			}
		} catch {
			systemStatus = { api: 'error', db: 'error' };
		}
	}

	onMount(() => {
		checkHealth();
		if (!data.initialData) loadAll();
		return () => {
			if (trendChart) {
				trendChart.destroy();
				trendChart = null;
			}
		};
	});

	async function loadAll() {
		error = '';
		loading = true;
		checkHealth();
		try {
			const results = await Promise.allSettled([
				getKpi(),
				getAlerts(),
				getMilkTrend(30, 14),
				getReproductionForecast(),
				getFeedForecast(),
				getLatestMilk(),
				getUpcomingFollowUps(7),
				getActiveWithdrawals(),
				listTasks({ overdue: true }),
			]);
			if (results[0].status === 'fulfilled') kpi = results[0].value;
			if (results[1].status === 'fulfilled') alerts = results[1].value.alerts;
			if (results[2].status === 'fulfilled') trend = results[2].value;
			if (results[3].status === 'fulfilled') repro = results[3].value;
			if (results[4].status === 'fulfilled') feed = results[4].value;
			if (results[5].status === 'fulfilled') latestMilk = results[5].value;
			if (results[6].status === 'fulfilled') vetFollowUps = results[6].value.data ?? [];
			if (results[7].status === 'fulfilled') activeWithdrawals = results[7].value.data ?? [];
			if (results[8].status === 'fulfilled') overdueTasks = results[8].value.data ?? [];
			const failed = results.find((r) => r.status === 'rejected');
			if (failed && failed.status === 'rejected') {
				error = failed.reason?.message || 'Ошибка загрузки данных';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			loading = false;
		}
	}

	function fmt(v: number | null | undefined, suffix = ''): string {
		if (v == null) return '—';
		return v.toFixed(1) + suffix;
	}

	function severityClass(s: string): string {
		if (s === 'critical') return 'bg-red-100 text-red-800 dark:bg-red-900/40 dark:text-red-300';
		if (s === 'warning')
			return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/40 dark:text-yellow-300';
		return 'bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300';
	}

	function severityLabel(s: string): string {
		if (s === 'critical') return 'Критично';
		if (s === 'warning') return 'Внимание';
		return 'Инфо';
	}

	function trendLabel(dir: string): string {
		if (dir === 'up') return 'Рост';
		if (dir === 'down') return 'Снижение';
		if (dir === 'stable') return 'Стабильно';
		return 'Мало данных';
	}

	function trendColor(dir: string): string {
		if (dir === 'up') return 'text-green-600 dark:text-green-400';
		if (dir === 'down') return 'text-red-600 dark:text-red-400';
		return 'text-slate-600 dark:text-slate-400';
	}

	function riskBadge(score: number): string {
		if (score >= 0.7) return 'bg-red-100 text-red-800 dark:bg-red-900/40 dark:text-red-300';
		if (score >= 0.5)
			return 'bg-orange-100 text-orange-800 dark:bg-orange-900/40 dark:text-orange-300';
		return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/40 dark:text-yellow-300';
	}

	function priorityBadge(p: TaskPriority): string {
		if (p === 'urgent') return 'bg-red-100 text-red-800 dark:bg-red-900/40 dark:text-red-300';
		if (p === 'high') return 'bg-orange-100 text-orange-800 dark:bg-orange-900/40 dark:text-orange-300';
		if (p === 'medium') return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/40 dark:text-yellow-300';
		return 'bg-slate-100 text-slate-800 dark:bg-slate-700/40 dark:text-slate-300';
	}

	function buildTrendChart() {
		if (!trendCanvas || !trend || trend.daily.length === 0) return;
		if (trendChart) {
			trendChart.destroy();
			trendChart = null;
		}

		const isDark = $theme === 'dark';
		const { textColor } = themeColors(isDark);

		const allDates = [...trend.daily.map((d) => d.date), ...trend.forecast.map((f) => f.date)];
		const actualData = trend.daily.map((d) => d.total_milk ?? null);
		const forecastData: (number | null)[] = new Array(trend.daily.length).fill(null);
		if (trend.daily.length > 0) {
			forecastData.push(trend.daily[trend.daily.length - 1]?.total_milk ?? null);
		}
		for (const f of trend.forecast) {
			forecastData.push(f.predicted);
		}
		const upperData: (number | null)[] = new Array(trend.daily.length).fill(null);
		if (trend.daily.length > 0) {
			upperData.push(trend.daily[trend.daily.length - 1]?.total_milk ?? null);
		}
		for (const f of trend.forecast) {
			upperData.push(f.upper);
		}
		const lowerData: (number | null)[] = new Array(trend.daily.length).fill(null);
		if (trend.daily.length > 0) {
			lowerData.push(trend.daily[trend.daily.length - 1]?.total_milk ?? null);
		}
		for (const f of trend.forecast) {
			lowerData.push(f.lower);
		}

		const labels = allDates.map((d) => {
			const parts = d.split('-');
			return `${parts[2]}.${parts[1]}`;
		});

		trendChart = new Chart(trendCanvas, {
			type: 'line',
			data: {
				labels,
				datasets: [
					{
						label: 'Надой (факт), л',
						data: actualData,
						borderColor: isDark ? 'rgba(96,165,250,1)' : 'rgba(37,99,235,1)',
						backgroundColor: isDark ? 'rgba(96,165,250,0.1)' : 'rgba(37,99,235,0.1)',
						fill: true,
						tension: 0.3,
						pointRadius: 2,
						pointHoverRadius: 5,
						spanGaps: false,
					},
					{
						label: 'Прогноз, л',
						data: forecastData,
						borderColor: isDark ? 'rgba(52,211,153,1)' : 'rgba(5,150,105,1)',
						borderDash: [6, 3],
						fill: false,
						tension: 0.3,
						pointRadius: 2,
						pointHoverRadius: 5,
						spanGaps: false,
					},
					{
						label: 'Верхняя граница',
						data: upperData,
						borderColor: 'transparent',
						backgroundColor: isDark ? 'rgba(52,211,153,0.08)' : 'rgba(5,150,105,0.08)',
						fill: '+1',
						tension: 0.3,
						pointRadius: 0,
						spanGaps: false,
					},
					{
						label: 'Нижняя граница',
						data: lowerData,
						borderColor: 'transparent',
						fill: false,
						tension: 0.3,
						pointRadius: 0,
						spanGaps: false,
					},
				],
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				interaction: { intersect: false, mode: 'index' },
				plugins: {
					legend: {
						labels: { color: textColor, usePointStyle: true, font: { size: 11 } },
					},
					tooltip: defaultTooltip(isDark, {
						// eslint-disable-next-line @typescript-eslint/no-explicit-any
						label: (ctx: any) =>
							ctx.dataset.label?.includes('граница')
								? ''
								: `${ctx.dataset.label}: ${ctx.parsed.y?.toFixed(1) ?? '—'} л`,
					}),
				},
				scales: defaultScales(isDark, (v) => `${v} л`),
			},
		});
	}

	let debouncedTrend = debounce(() => buildTrendChart(), 50);

	$effect(() => {
		trend;
		$theme;
		if (trendCanvas && trend) {
			debouncedTrend();
		}
	});
</script>

<svelte:head>
	<title>Дашборд — Молочная ферма</title>
</svelte:head>

{#if showSettings}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
		onclick={() => (showSettings = false)}
		role="presentation"
	>
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-xl border border-slate-200 dark:border-slate-700 w-full max-w-md mx-4 p-6"
			onclick={(e) => e.stopPropagation()}
		>
			<div class="flex items-center justify-between mb-4">
				<h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100">Настройка виджетов</h2>
				<button
					onclick={() => (showSettings = false)}
					class="p-1 rounded hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors cursor-pointer"
				><X size={18} class="text-slate-400" /></button>
			</div>
			<div class="space-y-2">
				{#each ALL_WIDGETS as w (w.id)}
					<label class="flex items-center gap-3 p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700/50 cursor-pointer">
						<input
							type="checkbox"
							checked={hasWidget(w.id)}
							onchange={() => toggleWidget(w.id)}
							class="h-4 w-4 rounded border-slate-300 dark:border-slate-600 text-blue-600 focus:ring-blue-500"
						/>
						<span class="text-sm text-slate-700 dark:text-slate-300">{w.label}</span>
					</label>
				{/each}
			</div>
			<div class="mt-4 flex justify-between items-center">
				<button
					onclick={() => { widgets = ALL_WIDGETS.map(w => w.id); saveWidgets(); }}
					class="text-xs text-blue-600 dark:text-blue-400 hover:underline cursor-pointer"
				>Показать все</button>
				<button
					onclick={() => (showSettings = false)}
					class="px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors cursor-pointer"
				>Готово</button>
			</div>
			{#if settingsSaving}
				<div class="mt-2 text-xs text-slate-400 text-center">Сохранение...</div>
			{/if}
		</div>
	</div>
{/if}

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Дашборд</h1>
	<div class="flex items-center gap-2">
		<button
			onclick={() => (showSettings = true)}
			class="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors cursor-pointer"
			title="Настройка виджетов"
		><Settings size={14} /></button>
		<button
			onclick={loadAll}
			disabled={loading}
			class="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors cursor-pointer disabled:opacity-50"
		><RefreshCw size={14} />Обновить</button
		>
	</div>
</div>

<ErrorAlert message={error} />

{#if hasWidget('system_status')}
	<div
		class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-6"
	>
		<div class="flex items-center gap-3 flex-wrap">
			<span class="text-xs font-medium text-slate-500 dark:text-slate-400 uppercase tracking-wide"
				>Статус системы</span
			>
			<div class="flex items-center gap-1.5">
				{#if systemStatus.api === 'checking'}
					<div
						class="h-2.5 w-2.5 rounded-full bg-slate-300 dark:bg-slate-600 animate-pulse"
						title="Проверка..."
					></div>
				{:else if systemStatus.api === 'ok'}
					<div class="h-2.5 w-2.5 rounded-full bg-green-500" title="API работает"></div>
				{:else}
					<div class="h-2.5 w-2.5 rounded-full bg-red-500" title="API недоступен"></div>
				{/if}
				<Server size={14} class="text-slate-400" />
				<span
					class="text-xs {systemStatus.api === 'ok'
						? 'text-green-600 dark:text-green-400'
						: systemStatus.api === 'error'
							? 'text-red-600 dark:text-red-400'
							: 'text-slate-400'}"
				>
					{systemStatus.api === 'checking'
						? 'Проверка...'
						: systemStatus.api === 'ok'
							? 'API'
							: 'API недоступен'}
				</span>
			</div>
			<div class="flex items-center gap-1.5">
				{#if systemStatus.db === 'checking'}
					<div
						class="h-2.5 w-2.5 rounded-full bg-slate-300 dark:bg-slate-600 animate-pulse"
						title="Проверка..."
					></div>
				{:else if systemStatus.db === 'ok'}
					<div class="h-2.5 w-2.5 rounded-full bg-green-500" title="БД работает"></div>
				{:else}
					<div class="h-2.5 w-2.5 rounded-full bg-red-500" title="БД недоступна"></div>
				{/if}
				<Database size={14} class="text-slate-400" />
				<span
					class="text-xs {systemStatus.db === 'ok'
						? 'text-green-600 dark:text-green-400'
						: systemStatus.db === 'error'
							? 'text-red-600 dark:text-red-400'
							: 'text-slate-400'}"
				>
					{systemStatus.db === 'checking'
						? 'Проверка...'
						: systemStatus.db === 'ok'
							? 'БД'
							: 'БД недоступна'}
				</span>
			</div>
			{#if lelyStatus}
				<div class="flex items-center gap-1.5">
					{#if lelyStatus.connected}
						<div class="h-2.5 w-2.5 rounded-full bg-green-500" title="Lely подключён"></div>
					{:else if lelyStatus.enabled}
						<div class="h-2.5 w-2.5 rounded-full bg-yellow-500" title="Lely: ошибка"></div>
					{:else}
						<div
							class="h-2.5 w-2.5 rounded-full bg-slate-300 dark:bg-slate-600"
							title="Lely отключён"
						></div>
					{/if}
					<Plug size={14} class="text-slate-400" />
					<span
						class="text-xs {lelyStatus.connected
							? 'text-green-600 dark:text-green-400'
							: lelyStatus.enabled
								? 'text-yellow-600 dark:text-yellow-400'
								: 'text-slate-400'}"
					>
						{lelyStatus.connected ? 'Lely' : lelyStatus.enabled ? 'Lely: ошибка' : 'Lely: выкл'}
					</span>
				</div>
			{/if}
			{#if systemStatus.api === 'error' || systemStatus.db === 'error'}
				<div class="flex items-center gap-1 ml-auto">
					<CircleAlert size={14} class="text-amber-500" />
					<span class="text-xs text-amber-600 dark:text-amber-400">Часть сервисов недоступна</span>
				</div>
			{:else if systemStatus.api === 'ok' && systemStatus.db === 'ok'}
				<div class="flex items-center gap-1 ml-auto">
					<Activity size={14} class="text-green-500" />
					<span class="text-xs text-green-600 dark:text-green-400">Все системы работают</span>
				</div>
			{/if}
		</div>
	</div>
{/if}

{#if loading}
	<div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
		{#each Array(8) as _, i (i)}
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
			>
				<div class="h-3 bg-slate-200 dark:bg-slate-700 rounded w-2/3 mb-3 animate-pulse"></div>
				<div class="h-7 bg-slate-200 dark:bg-slate-700 rounded w-1/2 animate-pulse"></div>
			</div>
		{/each}
	</div>
	<div
		class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-6"
	>
		<div class="h-4 bg-slate-200 dark:bg-slate-700 rounded w-1/4 mb-4 animate-pulse"></div>
		<div class="h-72 bg-slate-100 dark:bg-slate-900 rounded animate-pulse"></div>
	</div>
{:else}
	{#if hasWidget('kpi')}
		<div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">Интервал отёлов</div>
				<div class="text-2xl font-bold text-blue-600 dark:text-blue-400 mt-1">
					{fmt(kpi?.avg_calving_interval_days, ' д')}
				</div>
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">% оплодотворяемости</div>
				<div class="text-2xl font-bold text-green-600 dark:text-green-400 mt-1">
					{fmt(kpi?.conception_rate_pct, '%')}
				</div>
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">Эффективность корма</div>
				<div class="text-2xl font-bold text-purple-600 dark:text-purple-400 mt-1">
					{fmt(kpi?.feed_efficiency)}
				</div>
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">Дней до 1-й инсеминации</div>
				<div class="text-2xl font-bold text-orange-600 dark:text-orange-400 mt-1">
					{fmt(kpi?.avg_days_to_first_ai, ' д')}
				</div>
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">Средний SCC</div>
				<div class="text-2xl font-bold text-red-600 dark:text-red-400 mt-1">
					{kpi?.avg_scc != null ? Math.round(kpi.avg_scc).toLocaleString('ru-RU') : '—'}
				</div>
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">% отказов</div>
				<div class="text-2xl font-bold text-yellow-600 dark:text-yellow-400 mt-1">
					{fmt(kpi?.refusal_rate_pct, '%')}
				</div>
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">Надой по лактациям</div>
				<div class="text-sm font-semibold text-slate-700 dark:text-slate-300 mt-1">
					{#if kpi?.avg_milk_by_lactation?.length}
						{#each kpi.avg_milk_by_lactation as l, i (i)}
							{i > 0 ? ', ' : ''}Л{l.lac}: {fmt(l.avg_milk)} л
						{/each}
					{:else}—{/if}
				</div>
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">Прогноз корма (7 д)</div>
				<div class="text-2xl font-bold text-teal-600 dark:text-teal-400 mt-1">
					{fmt(feed?.predicted_next_week_kg, ' кг')}
				</div>
				{#if feed?.milk_per_feed != null}
					<div class="text-xs text-slate-400 mt-0.5">Молоко/корм: {fmt(feed.milk_per_feed)}</div>
				{/if}
			</div>
		</div>
	{/if}

	{#if hasWidget('alerts') && alerts.length > 0}
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700 mb-6"
		>
			<h2 class="text-base font-semibold text-slate-700 dark:text-slate-300 mb-3">
				Предупреждения ({alerts.length})
			</h2>
			<div class="space-y-2 max-h-64 overflow-y-auto">
				{#each alerts as a, i (i)}
					<div
						class="flex items-start gap-3 p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700/50"
					>
						<span
							class="px-2 py-0.5 text-xs font-medium rounded-full whitespace-nowrap {severityClass(
								a.severity,
							)}">{severityLabel(a.severity)}</span
						>
						<div class="flex-1 min-w-0">
							<div class="text-sm text-slate-700 dark:text-slate-300">
								{#if a.animal_name}
									<a
										href="/animals/{a.animal_id}"
										class="font-medium text-blue-600 dark:text-blue-400 hover:underline"
										>{a.animal_name}</a
									>
								{/if}
								{a.message}
							</div>
							<div class="text-xs text-slate-500 dark:text-slate-400">{a.value}</div>
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	{#if hasWidget('milk_trend')}
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700 mb-6"
		>
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-base font-semibold text-slate-700 dark:text-slate-300">Тренд надоя</h2>
				{#if trend}
					<span class="text-sm font-medium {trendColor(trend.trend_direction)}"
						>{trendLabel(trend.trend_direction)}</span
					>
				{/if}
			</div>
			<div class="relative h-72">
				{#if trend && trend.daily.length > 0}
					<canvas bind:this={trendCanvas}></canvas>
				{:else}
					<div
						class="flex items-center justify-center h-full text-slate-400 dark:text-slate-500 text-sm"
					>
						Недостаточно данных для графика
					</div>
				{/if}
			</div>
		</div>
	{/if}

	{#if hasWidget('latest_milk') && latestMilk.length > 0}
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700 mb-6"
		>
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-base font-semibold text-slate-700 dark:text-slate-300">Последние надои</h2>
				<span class="text-xs text-slate-400">{latestMilk[0]?.date}</span>
			</div>
			<div class="overflow-x-auto">
				<table class="w-full text-sm">
					<thead>
						<tr
							class="text-left text-xs text-slate-500 dark:text-slate-400 border-b border-slate-200 dark:border-slate-700"
						>
							<th class="pb-2 font-medium">Животное</th>
							<th class="pb-2 font-medium text-right">Надой, л</th>
							<th class="pb-2 font-medium text-right">Среднее, л</th>
							<th class="pb-2 font-medium text-right">ИМК</th>
						</tr>
					</thead>
					<tbody>
						{#each latestMilk as m, i (i)}
							<tr
								class="border-b border-slate-100 dark:border-slate-700/50 hover:bg-slate-50 dark:hover:bg-slate-700/50"
							>
								<td class="py-1.5">
									<a
										href="/animals/{m.animal_id}"
										class="text-blue-600 dark:text-blue-400 hover:underline"
										>{m.name || 'ID ' + m.animal_id}</a
									>
								</td>
								<td class="py-1.5 text-right font-medium text-slate-700 dark:text-slate-300"
									>{m.milk_amount != null ? m.milk_amount.toFixed(1) : '—'}</td
								>
								<td class="py-1.5 text-right text-slate-600 dark:text-slate-400"
									>{m.avg_amount != null ? m.avg_amount.toFixed(1) : '—'}</td
								>
								<td class="py-1.5 text-right text-slate-600 dark:text-slate-400"
									>{m.isk != null ? m.isk.toFixed(1) : '—'}</td
								>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}

	{#if hasWidget('reproduction')}
		<div class="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
			>
				<h2 class="text-base font-semibold text-slate-700 dark:text-slate-300 mb-3">
					Ожидаемые отёлы
				</h2>
				{#if repro?.expected_calvings?.length}
					<div class="space-y-2 max-h-64 overflow-y-auto">
						{#each repro.expected_calvings as c, i (`calving-${c.animal_id}-${i}`)}
							<a
								href="/animals/{c.animal_id}"
								class="block p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700/50"
							>
								<div class="text-sm font-medium text-slate-700 dark:text-slate-300">
									{c.name || 'ID ' + c.animal_id}
								</div>
								<div class="flex justify-between text-xs text-slate-500 dark:text-slate-400 mt-0.5">
									<span>{c.expected_date}</span>
									<span
										class:font-semibold={c.days_left <= 14}
										class:text-red-600={c.days_left <= 14}
									>
										{c.days_left > 0 ? c.days_left + ' дн.' : 'Скоро!'}
									</span>
								</div>
							</a>
						{/each}
					</div>
				{:else}
					<div class="text-sm text-slate-400 dark:text-slate-500 py-4 text-center">Нет данных</div>
				{/if}
			</div>

			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
			>
				<h2 class="text-base font-semibold text-slate-700 dark:text-slate-300 mb-3">
					Ожидаемые охоты
				</h2>
				{#if repro?.expected_heats?.length}
					<div class="space-y-2 max-h-64 overflow-y-auto">
						{#each repro.expected_heats as h, i (`heat-${h.animal_id}-${i}`)}
							<a
								href="/animals/{h.animal_id}"
								class="block p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700/50"
							>
								<div class="text-sm font-medium text-slate-700 dark:text-slate-300">
									{h.name || 'ID ' + h.animal_id}
									{#if h.overdue}
										<span
											class="ml-1 px-1.5 py-0.5 text-xs bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300 rounded"
											>Просрочено</span
										>
									{/if}
								</div>
								<div class="flex justify-between text-xs text-slate-500 dark:text-slate-400 mt-0.5">
									<span>{h.expected_next}</span>
									<span
										>{h.days_until >= 0
											? h.days_until + ' дн.'
											: Math.abs(h.days_until) + ' дн. назад'}</span
									>
								</div>
							</a>
						{/each}
					</div>
				{:else}
					<div class="text-sm text-slate-400 dark:text-slate-500 py-4 text-center">Нет данных</div>
				{/if}
			</div>

			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
			>
				<h2 class="text-base font-semibold text-slate-700 dark:text-slate-300 mb-3">
					Рекомендуемые запуски
				</h2>
				{#if repro?.dry_off_recommendations?.length}
					<div class="space-y-2 max-h-64 overflow-y-auto">
						{#each repro.dry_off_recommendations as d, i (`dryoff-${d.animal_id}-${i}`)}
							<a
								href="/animals/{d.animal_id}"
								class="block p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700/50"
							>
								<div class="text-sm font-medium text-slate-700 dark:text-slate-300">
									{d.name || 'ID ' + d.animal_id}
								</div>
								<div class="flex justify-between text-xs text-slate-500 dark:text-slate-400 mt-0.5">
									<span>{d.recommended_dry_off}</span>
									<span
										class:font-semibold={d.days_until_dry_off <= 7}
										class:text-orange-600={d.days_until_dry_off <= 7}
									>
										{d.days_until_dry_off > 0 ? d.days_until_dry_off + ' дн.' : 'Сейчас!'}
									</span>
								</div>
							</a>
						{/each}
					</div>
				{:else}
					<div class="text-sm text-slate-400 dark:text-slate-500 py-4 text-center">Нет данных</div>
				{/if}
			</div>
		</div>
	{/if}

	{#if hasWidget('kpi') && kpi?.culling_risk?.length}
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700 mb-6"
		>
			<h2 class="text-base font-semibold text-slate-700 dark:text-slate-300 mb-3">Риск выбытия</h2>
			<div class="overflow-x-auto">
				<table class="w-full text-sm">
					<thead>
						<tr
							class="text-left text-xs text-slate-500 dark:text-slate-400 border-b border-slate-200 dark:border-slate-700"
						>
							<th class="pb-2 font-medium">Животное</th>
							<th class="pb-2 font-medium">Риск</th>
							<th class="pb-2 font-medium">Причины</th>
						</tr>
					</thead>
					<tbody>
						{#each kpi.culling_risk as r, i (i)}
							<tr class="border-b border-slate-100 dark:border-slate-700/50">
								<td class="py-2">
									<a
										href="/animals/{r.animal_id}"
										class="text-blue-600 dark:text-blue-400 hover:underline"
										>{r.name || 'ID ' + r.animal_id}</a
									>
								</td>
								<td class="py-2">
									<span class="px-2 py-0.5 text-xs font-medium rounded-full {riskBadge(r.score)}">
										{(r.score * 100).toFixed(0)}%
									</span>
								</td>
								<td class="py-2 text-slate-600 dark:text-slate-400">{r.reasons.join(', ')}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}

	{#if hasWidget('vet_followups') && vetFollowUps.length > 0}
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700 mb-6"
		>
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-base font-semibold text-slate-700 dark:text-slate-300">
					Повторные приёмы ({vetFollowUps.length})
				</h2>
				<a
					href="/vet"
					class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
				>Все записи</a>
			</div>
			<div class="space-y-2 max-h-64 overflow-y-auto">
				{#each vetFollowUps as v, i (`followup-${v.id}-${i}`)}
					<a
						href="/animals/{v.animal_id}"
						class="flex items-center gap-3 p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700/50"
					>
						<span
							class="px-2 py-0.5 text-xs font-medium rounded-full whitespace-nowrap bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300"
						>
							{VET_RECORD_TYPE_LABELS[v.record_type] ?? v.record_type}
						</span>
						<div class="flex-1 min-w-0">
							<div class="text-sm font-medium text-slate-700 dark:text-slate-300">
								Животное ID {v.animal_id}
							</div>
							{#if v.follow_up_date}
								<div class="text-xs text-slate-500 dark:text-slate-400">
									Дата: {v.follow_up_date}
								</div>
							{/if}
							{#if v.diagnosis}
								<div class="text-xs text-slate-500 dark:text-slate-400">
									Диагноз: {v.diagnosis}
								</div>
							{/if}
						</div>
					</a>
				{/each}
			</div>
		</div>
	{/if}

	{#if hasWidget('active_withdrawals') && activeWithdrawals.length > 0}
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700 mb-6"
		>
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-base font-semibold text-slate-700 dark:text-slate-300">
					Периоды ожидания ({activeWithdrawals.length})
				</h2>
			</div>
			<div class="space-y-2 max-h-64 overflow-y-auto">
				{#each activeWithdrawals as w, i (`withdrawal-${w.id}-${i}`)}
					<a
						href="/animals/{w.animal_id}"
						class="flex items-center gap-3 p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700/50"
					>
						<span
							class="px-2 py-0.5 text-xs font-medium rounded-full whitespace-nowrap bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
						>
							{w.medication ?? 'Препарат'}
						</span>
						<div class="flex-1 min-w-0">
							<div class="text-sm font-medium text-slate-700 dark:text-slate-300">
								Животное ID {w.animal_id}
							</div>
							{#if w.withdrawal_end_date}
								<div class="text-xs text-slate-500 dark:text-slate-400">
									Окончание: {w.withdrawal_end_date}
								</div>
							{/if}
							{#if w.dosage}
								<div class="text-xs text-slate-500 dark:text-slate-400">
									Дозировка: {w.dosage}
								</div>
							{/if}
						</div>
					</a>
				{/each}
			</div>
		</div>
	{/if}

	{#if hasWidget('overdue_tasks') && overdueTasks.length > 0}
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700 mb-6"
		>
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-base font-semibold text-slate-700 dark:text-slate-300">
					Просроченные задачи ({overdueTasks.length})
				</h2>
			</div>
			<div class="space-y-2 max-h-64 overflow-y-auto">
				{#each overdueTasks as t, i (`task-${t.id}-${i}`)}
					<div
						class="flex items-center gap-3 p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700/50"
					>
						<span
							class="px-2 py-0.5 text-xs font-medium rounded-full whitespace-nowrap {priorityBadge(t.priority)}"
						>
							{TASK_PRIORITY_LABELS[t.priority] ?? t.priority}
						</span>
						<div class="flex-1 min-w-0">
							<div class="text-sm font-medium text-slate-700 dark:text-slate-300">
								{t.title}
							</div>
							<div class="text-xs text-slate-500 dark:text-slate-400">
								Срок: {t.due_date ?? '—'}
								{#if t.assigned_to}
									· {t.assigned_to}
								{/if}
							</div>
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Quick Links -->
	<div
		class="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4 border border-slate-100 dark:border-slate-700"
	>
		<h2 class="text-base font-semibold text-slate-700 dark:text-slate-300 mb-2">Быстрый переход</h2>
		<div class="grid grid-cols-2 md:grid-cols-5 gap-3 mt-3">
			<a
				href="/animals"
				class="flex flex-col items-center gap-1 p-3 rounded-lg border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 hover:border-blue-300 transition-colors"
			>
				<Beef size={24} class="text-slate-600 dark:text-slate-400" />
				<span class="text-xs text-slate-600 dark:text-slate-400">Животные</span>
			</a>
			<a
				href="/milk"
				class="flex flex-col items-center gap-1 p-3 rounded-lg border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 hover:border-blue-300 transition-colors"
			>
				<Milk size={24} class="text-slate-600 dark:text-slate-400" />
				<span class="text-xs text-slate-600 dark:text-slate-400">Удои</span>
			</a>
			<a
				href="/reproduction"
				class="flex flex-col items-center gap-1 p-3 rounded-lg border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 hover:border-blue-300 transition-colors"
			>
				<Heart size={24} class="text-slate-600 dark:text-slate-400" />
				<span class="text-xs text-slate-600 dark:text-slate-400">Воспроизводство</span>
			</a>
			<a
				href="/feed"
				class="flex flex-col items-center gap-1 p-3 rounded-lg border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 hover:border-blue-300 transition-colors"
			>
				<Wheat size={24} class="text-slate-600 dark:text-slate-400" />
				<span class="text-xs text-slate-600 dark:text-slate-400">Кормление</span>
			</a>
			<a
				href="/contacts"
				class="flex flex-col items-center gap-1 p-3 rounded-lg border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 hover:border-blue-300 transition-colors"
			>
				<Users size={24} class="text-slate-600 dark:text-slate-400" />
				<span class="text-xs text-slate-600 dark:text-slate-400">Контакты</span>
			</a>
		</div>
	</div>
{/if}
