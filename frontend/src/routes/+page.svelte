<script lang="ts">
	import { onMount } from 'svelte';
	import { Chart, LineController, CategoryScale, LinearScale, PointElement, LineElement, Filler, Tooltip, Legend } from 'chart.js';
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
	} from 'lucide-svelte';
	import {
		getKpi,
		getAlerts,
		getMilkTrend,
		getReproductionForecast,
		getFeedForecast,
		type KpiResponse,
		type Alert,
		type MilkTrendResponse,
		type ReproductionForecastResponse,
		type FeedForecastResponse,
	} from '$lib/api/analytics';

	Chart.register(LineController, CategoryScale, LinearScale, PointElement, LineElement, Filler, Tooltip, Legend);

	interface SystemStatus {
		api: 'ok' | 'error' | 'checking';
		db: 'ok' | 'error' | 'checking';
	}

	let systemStatus = $state<SystemStatus>({ api: 'checking', db: 'checking' });

	let error = $state('');
	let loading = $state(true);
	let kpi = $state<KpiResponse | null>(null);
	let alerts = $state<Alert[]>([]);
	let trend = $state<MilkTrendResponse | null>(null);
	let repro = $state<ReproductionForecastResponse | null>(null);
	let feed = $state<FeedForecastResponse | null>(null);
	let trendCanvas: HTMLCanvasElement | undefined = $state();
	let trendChart: Chart | null = null;

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
			} else {
				systemStatus = { api: 'error', db: 'error' };
			}
		} catch {
			systemStatus = { api: 'error', db: 'error' };
		}
	}

	onMount(() => {
		loadAll();
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
			]);
			if (results[0].status === 'fulfilled') kpi = results[0].value;
			if (results[1].status === 'fulfilled') alerts = results[1].value.alerts;
			if (results[2].status === 'fulfilled') trend = results[2].value;
			if (results[3].status === 'fulfilled') repro = results[3].value;
			if (results[4].status === 'fulfilled') feed = results[4].value;
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

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Дашборд</h1>
	<button
		onclick={loadAll}
		disabled={loading}
		class="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors cursor-pointer disabled:opacity-50"
		><RefreshCw size={14} />Обновить</button
	>
</div>

<ErrorAlert message={error} />

<!-- System Status -->
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
	<!-- KPI Cards -->
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

	<!-- Alerts -->
	{#if alerts.length > 0}
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

	<!-- Milk Trend Chart -->
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

	<!-- Reproduction Forecasts -->
	<div class="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
		<!-- Expected Calvings -->
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

		<!-- Expected Heats -->
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

		<!-- Dry Off Recommendations -->
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

	<!-- Culling Risk -->
	{#if kpi?.culling_risk?.length}
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
