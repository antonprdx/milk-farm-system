<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import {
		getAnimal,
		deleteAnimal,
		getAnimalTimeline,
		getAnimalStats,
		type Animal,
		type TimelineEvent,
		type AnimalStats,
	} from '$lib/api/animals';
	import {
		getHealthIndex,
		getMastitisRisk,
		getEstrusDetection,
		getMilkForecast,
		getEnergyBalance,
		getFeedRecommendation,
		getKetosisWarning,
		getLifetimeValue,
		getCullingSurvival,
		getCowClusters,
		type CowHealthIndex,
		type MastitisRiskEntry,
		type EstrusPrediction,
		type MilkForecastResponse,
		type CowEnergyBalance,
		type FeedRecommendationEntry,
		type KetosisWarningEntry,
		type LifetimeValueEntry,
		type CullingSurvivalEntry,
		type ClusterEntry,
	} from '$lib/api/analytics';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { toasts } from '$lib/stores/toast';
	import { theme } from '$lib/stores/theme';
	import {
		Chart,
		LineController,
		CategoryScale,
		LinearScale,
		PointElement,
		LineElement,
		Filler,
		Tooltip,
	} from 'chart.js';
	import { defaultTooltip, defaultScales, dsColors } from '$lib/utils/chartHelpers';
	import {
		Baby,
		Heart,
		Droplets,
		ThermometerSun,
		PauseCircle,
		Milk,
		TrendingUp,
		Activity,
		Calendar,
		ShieldCheck,
		CircleDot,
	} from 'lucide-svelte';

	Chart.register(
		LineController,
		CategoryScale,
		LinearScale,
		PointElement,
		LineElement,
		Filler,
		Tooltip,
	);

	let animal = $state<Animal | null>(null);
	let loading = $state(true);
	let error = $state('');
	let showDelete = $state(false);
	let deleteLoading = $state(false);

	let timeline = $state<TimelineEvent[]>([]);
	let timelineTotal = $state(0);
	let timelinePage = $state(1);
	let timelineLoading = $state(false);

	let stats = $state<AnimalStats | null>(null);
	let statsLoading = $state(false);

	let milkCanvas: HTMLCanvasElement | undefined = $state();
	let sccCanvas: HTMLCanvasElement | undefined = $state();
	let forecastCanvas: HTMLCanvasElement | undefined = $state();
	let milkChart: Chart | null = null;
	let sccChart: Chart | null = null;
	let forecastChart: Chart | null = null;

	let healthIndex = $state<CowHealthIndex | null>(null);
	let mastitisRisk = $state<MastitisRiskEntry | null>(null);
	let estrusPred = $state<EstrusPrediction | null>(null);
	let milkForecast = $state<MilkForecastResponse | null>(null);
	let energyBalance = $state<CowEnergyBalance | null>(null);
	let feedRec = $state<FeedRecommendationEntry | null>(null);
	let ketosisWarn = $state<KetosisWarningEntry | null>(null);
	let lifetimeVal = $state<LifetimeValueEntry | null>(null);
	let cullingRisk = $state<CullingSurvivalEntry | null>(null);
	let cowCluster = $state<ClusterEntry | null>(null);

	let id = $derived(Number($page.params.id));

	async function load() {
		try {
			loading = true;
			animal = (await getAnimal(id)).data;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			loading = false;
		}
	}

	async function loadTimeline() {
		try {
			timelineLoading = true;
			const res = await getAnimalTimeline(id, timelinePage, 30);
			timeline = res.data;
			timelineTotal = res.total;
		} catch (e) {
			console.warn('Failed to load timeline', e);
		} finally {
			timelineLoading = false;
		}
	}

	async function loadStats() {
		try {
			statsLoading = true;
			stats = await getAnimalStats(id);
		} catch (e) {
			console.warn('Failed to load animal stats', e);
		} finally {
			statsLoading = false;
		}
	}

	async function handleDelete() {
		try {
			deleteLoading = true;
			await deleteAnimal(id);
			toasts.success('Животное удалено');
			goto('/animals');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка удаления';
			showDelete = false;
		} finally {
			deleteLoading = false;
		}
	}

	function eventIcon(type: string) {
		switch (type) {
			case 'Отёл':
				return Baby;
			case 'Осеменение':
				return Heart;
			case 'Охота':
				return ThermometerSun;
			case 'Запуск':
				return PauseCircle;
			case 'Надой':
				return Milk;
			default:
				return Droplets;
		}
	}

	function eventColor(type: string) {
		switch (type) {
			case 'Отёл':
				return 'bg-green-100 dark:bg-green-900/50 text-green-600 dark:text-green-400';
			case 'Осеменение':
				return 'bg-pink-100 dark:bg-pink-900/50 text-pink-600 dark:text-pink-400';
			case 'Охота':
				return 'bg-orange-100 dark:bg-orange-900/50 text-orange-600 dark:text-orange-400';
			case 'Запуск':
				return 'bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-400';
			case 'Надой':
				return 'bg-blue-100 dark:bg-blue-900/50 text-blue-600 dark:text-blue-400';
			default:
				return 'bg-purple-100 dark:bg-purple-900/50 text-purple-600 dark:text-purple-400';
		}
	}

	function buildMilkChart() {
		if (!milkCanvas || !stats || stats.milk_production_30d.length === 0) return;

		if (milkChart) {
			milkChart.destroy();
			milkChart = null;
		}

		const isDark = $theme === 'dark';

		const data = stats.milk_production_30d;

		milkChart = new Chart(milkCanvas, {
			type: 'line',
			data: {
				labels: data.map((p) => {
					const d = new Date(p.date + 'T00:00:00');
					return d.toLocaleDateString('ru-RU', { day: 'numeric', month: 'short' });
				}),
				datasets: [
					{
						label: 'Надой, л',
						data: data.map((p) => p.amount),
						borderColor: dsColors(isDark, 'blue').border,
						backgroundColor: dsColors(isDark, 'blue').bg,
						fill: true,
						tension: 0.3,
						pointRadius: 3,
						pointHoverRadius: 6,
						pointBackgroundColor: dsColors(isDark, 'blue').point,
						borderWidth: 2,
					},
				],
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: { display: false },
					tooltip: defaultTooltip(isDark, {
						// eslint-disable-next-line @typescript-eslint/no-explicit-any
						label: (ctx: any) => `${ctx.parsed.y?.toFixed(1) ?? '0'} л`,
					}),
				},
				scales: defaultScales(isDark, (v) => `${v} л`),
			},
		});
	}

	function buildSccChart() {
		if (!sccCanvas || !stats || stats.scc_trend_90d.length === 0) return;

		if (sccChart) {
			sccChart.destroy();
			sccChart = null;
		}

		const isDark = $theme === 'dark';
		const data = stats.scc_trend_90d;

		sccChart = new Chart(sccCanvas, {
			type: 'line',
			data: {
				labels: data.map((p) => {
					const d = new Date(p.date + 'T00:00:00');
					return d.toLocaleDateString('ru-RU', { day: 'numeric', month: 'short' });
				}),
				datasets: [
					{
						label: 'СОМО (тыс.)',
						data: data.map((p) => p.scc / 1000),
						borderColor: dsColors(isDark, 'red').border,
						backgroundColor: dsColors(isDark, 'red').bg,
						fill: true,
						tension: 0.3,
						pointRadius: 3,
						pointHoverRadius: 6,
						pointBackgroundColor: dsColors(isDark, 'red').point,
						borderWidth: 2,
					},
				],
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: { display: false },
					tooltip: defaultTooltip(isDark, {
						// eslint-disable-next-line @typescript-eslint/no-explicit-any
						label: (ctx: any) => `${ctx.parsed.y?.toFixed(0) ?? '0'} тыс.`,
					}),
				},
				scales: defaultScales(isDark, (v) => `${v} тыс.`),
			},
		});
	}

	$effect(() => {
		stats;
		$theme;
		if (stats) {
			buildMilkChart();
			buildSccChart();
		}
	});

	$effect(() => {
		milkForecast;
		$theme;
		buildForecastChart();
	});

	$effect(() => {
		return () => {
			if (milkChart) {
				milkChart.destroy();
				milkChart = null;
			}
			if (sccChart) {
				sccChart.destroy();
				sccChart = null;
			}
			if (forecastChart) {
				forecastChart.destroy();
				forecastChart = null;
			}
		};
	});

	async function loadAnalytics() {
		const results = await Promise.allSettled([
			getHealthIndex(),
			getMastitisRisk(),
			getEstrusDetection(),
			getMilkForecast(id, 30),
			getEnergyBalance(),
			getFeedRecommendation(),
			getKetosisWarning(),
			getLifetimeValue(),
			getCullingSurvival(),
			getCowClusters(),
		]);

		const [health, mastitis, estrus, forecast, energy, feed, ketosis, lifetime, culling, clusters] = results;

		if (health.status === 'fulfilled')
			healthIndex = health.value.cows.find((c) => c.animal_id === id) ?? null;
		if (mastitis.status === 'fulfilled')
			mastitisRisk = mastitis.value.cows.find((c) => c.animal_id === id) ?? null;
		if (estrus.status === 'fulfilled')
			estrusPred = estrus.value.predictions.find((p) => p.animal_id === id) ?? null;
		if (forecast.status === 'fulfilled' && forecast.value.forecast.length > 0)
			milkForecast = forecast.value;
		if (energy.status === 'fulfilled')
			energyBalance = energy.value.cows.find((c) => c.animal_id === id) ?? null;
		if (feed.status === 'fulfilled')
			feedRec = feed.value.recommendations.find((r) => r.animal_id === id) ?? null;
		if (ketosis.status === 'fulfilled')
			ketosisWarn = ketosis.value.predictions.find((p) => p.animal_id === id) ?? null;
		if (lifetime.status === 'fulfilled')
			lifetimeVal = lifetime.value.cows.find((c) => c.animal_id === id) ?? null;
		if (culling.status === 'fulfilled')
			cullingRisk = culling.value.cows.find((c) => c.animal_id === id) ?? null;
		if (clusters.status === 'fulfilled')
			cowCluster = clusters.value.clusters.find((c) => c.animal_id === id) ?? null;
	}

	function buildForecastChart() {
		if (!forecastCanvas || !milkForecast || milkForecast.forecast.length === 0) return;

		if (forecastChart) {
			forecastChart.destroy();
			forecastChart = null;
		}

		const isDark = $theme === 'dark';
		const data = milkForecast.forecast;

		forecastChart = new Chart(forecastCanvas, {
			type: 'line',
			data: {
				labels: data.map((d) => `+${d.day_offset}д`),
				datasets: [
					{
						label: 'Прогноз, л',
						data: data.map((d) => d.predicted_milk),
						borderColor: dsColors(isDark, 'green').border,
						backgroundColor: dsColors(isDark, 'green').bg,
						fill: false,
						tension: 0.3,
						pointRadius: 2,
						borderWidth: 2,
					},
					{
						label: 'Верхняя граница',
						data: data.map((d) => d.upper_bound),
						borderColor: 'transparent',
						backgroundColor: dsColors(isDark, 'green').bg,
						fill: '+1',
						pointRadius: 0,
						borderWidth: 0,
					},
					{
						label: 'Нижняя граница',
						data: data.map((d) => d.lower_bound),
						borderColor: 'transparent',
						backgroundColor: 'transparent',
						fill: false,
						pointRadius: 0,
						borderWidth: 0,
					},
				],
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: { display: false },
					tooltip: defaultTooltip(isDark, {
						// eslint-disable-next-line @typescript-eslint/no-explicit-any
						label: (ctx: any) => `${ctx.parsed.y?.toFixed(1) ?? '0'} л`,
					}),
				},
				scales: defaultScales(isDark, (v) => `${v} л`),
			},
		});
	}

	function fmt(v: number | null | undefined, suffix = ''): string {
		if (v == null) return '—';
		return `${v.toFixed(1)}${suffix}`;
	}

	function fmtInt(v: number | null | undefined): string {
		if (v == null) return '—';
		return String(v);
	}

	onMount(() => {
		load();
		loadTimeline();
		loadStats();
		loadAnalytics();
	});
</script>

<svelte:head>
	<title>{animal?.name || 'Животное'} — Молочная ферма</title>
</svelte:head>

<div class="mb-6">
	<a
		href="/animals"
		class="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
		>&larr; Назад к списку</a
	>
</div>

<ErrorAlert message={error} />

{#if loading}
	<div
		class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
	>
		<div class="animate-pulse space-y-4">
			{#each Array(6) as _, i (i)}
				<div class="h-4 bg-slate-200 dark:bg-slate-700 rounded w-1/3"></div>
			{/each}
		</div>
	</div>
{:else if animal}
	<div class="flex items-center justify-between mb-6">
		<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">
			{animal.name || 'Без имени'}
			<span class="text-base font-normal text-slate-400 dark:text-slate-500 ml-2"
				>#{animal.life_number || animal.user_number || animal.id}</span
			>
		</h1>
		<div class="flex gap-2">
			<a
				href="/animals/{animal.id}/edit"
				class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-lg transition-colors"
			>
				Редактировать
			</a>
			<button
				onclick={() => (showDelete = true)}
				class="px-4 py-2 border border-red-300 text-red-600 hover:bg-red-50 dark:bg-red-900/50 text-sm rounded-lg transition-colors cursor-pointer"
			>
				Удалить
			</button>
		</div>
	</div>

	<!-- Status Cards -->
	{#if stats && !statsLoading}
		<div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4 mb-6">
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
			>
				<div class="flex items-center gap-2 mb-1">
					<Milk size={14} class="text-blue-500" />
					<span class="text-xs text-slate-500 dark:text-slate-400">Лактация</span>
				</div>
				<p class="text-lg font-semibold text-slate-800 dark:text-slate-100">
					{stats.reproduction.is_dry
						? 'Сухостой'
						: stats.reproduction.lactation_number != null
							? `#${stats.reproduction.lactation_number}`
							: '—'}
				</p>
			</div>

			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
			>
				<div class="flex items-center gap-2 mb-1">
					<ShieldCheck
						size={14}
						class={stats.reproduction.is_pregnant ? 'text-green-500' : 'text-slate-400'}
					/>
					<span class="text-xs text-slate-500 dark:text-slate-400">Стельность</span>
				</div>
				<p class="text-lg font-semibold text-slate-800 dark:text-slate-100">
					{stats.reproduction.is_pregnant ? 'Стельная' : 'Нет'}
				</p>
			</div>

			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
			>
				<div class="flex items-center gap-2 mb-1">
					<Calendar size={14} class="text-purple-500" />
					<span class="text-xs text-slate-500 dark:text-slate-400">Дней в лактации</span>
				</div>
				<p class="text-lg font-semibold text-slate-800 dark:text-slate-100">
					{stats.reproduction.days_in_milk ?? '—'}
				</p>
			</div>

			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
			>
				<div class="flex items-center gap-2 mb-1">
					<TrendingUp size={14} class="text-emerald-500" />
					<span class="text-xs text-slate-500 dark:text-slate-400">Сред. надой (30д)</span>
				</div>
				<p class="text-lg font-semibold text-slate-800 dark:text-slate-100">
					{fmt(stats.latest_metrics.avg_milk_30d, ' л')}
				</p>
			</div>

			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
			>
				<div class="flex items-center gap-2 mb-1">
					<Activity size={14} class="text-orange-500" />
					<span class="text-xs text-slate-500 dark:text-slate-400">Посл. СОМО</span>
				</div>
				<p class="text-lg font-semibold text-slate-800 dark:text-slate-100">
					{stats.latest_metrics.last_scc != null
						? `${(stats.latest_metrics.last_scc / 1000).toFixed(0)} тыс.`
						: '—'}
				</p>
			</div>

			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
			>
				<div class="flex items-center gap-2 mb-1">
					<CircleDot size={14} class="text-cyan-500" />
					<span class="text-xs text-slate-500 dark:text-slate-400">Сред. вес (30д)</span>
				</div>
				<p class="text-lg font-semibold text-slate-800 dark:text-slate-100">
					{fmt(stats.latest_metrics.avg_weight_30d, ' кг')}
				</p>
			</div>
		</div>

		<!-- Reproduction Summary -->
		{#if stats.reproduction.last_calving_date || stats.reproduction.expected_calving_date}
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-5 mb-6"
			>
				<h2
					class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3 flex items-center gap-2"
				>
					<Baby size={16} class="text-green-500" />
					Репродукция
				</h2>
				<div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
					<div>
						<dt class="text-xs text-slate-500 dark:text-slate-400">Последний отёл</dt>
						<dd class="text-sm font-medium text-slate-800 dark:text-slate-100 mt-0.5">
							{stats.reproduction.last_calving_date || '—'}
						</dd>
					</div>
					<div>
						<dt class="text-xs text-slate-500 dark:text-slate-400">Осеменений</dt>
						<dd class="text-sm font-medium text-slate-800 dark:text-slate-100 mt-0.5">
							{stats.reproduction.total_inseminations}
						</dd>
					</div>
					<div>
						<dt class="text-xs text-slate-500 dark:text-slate-400">Ожидаемый отёл</dt>
						<dd class="text-sm font-medium text-slate-800 dark:text-slate-100 mt-0.5">
							{stats.reproduction.expected_calving_date || '—'}
						</dd>
					</div>
					<div>
						<dt class="text-xs text-slate-500 dark:text-slate-400">Номер лактации</dt>
						<dd class="text-sm font-medium text-slate-800 dark:text-slate-100 mt-0.5">
							{fmtInt(stats.reproduction.lactation_number)}
						</dd>
					</div>
				</div>
			</div>
		{/if}

		<!-- Charts -->
		<div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
			>
				<h2
					class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3 flex items-center gap-2"
				>
					<TrendingUp size={16} class="text-blue-500" />
					Удои за 30 дней
				</h2>
				<div class="relative h-64">
					{#if stats.milk_production_30d.length === 0}
						<div
							class="flex items-center justify-center h-full text-slate-400 dark:text-slate-500 text-sm"
						>
							Нет данных
						</div>
					{:else}
						<canvas bind:this={milkCanvas}></canvas>
					{/if}
				</div>
			</div>

			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
			>
				<h2
					class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3 flex items-center gap-2"
				>
					<Activity size={16} class="text-red-500" />
					СОМО за 90 дней
				</h2>
				<div class="relative h-64">
					{#if stats.scc_trend_90d.length === 0}
						<div
							class="flex items-center justify-center h-full text-slate-400 dark:text-slate-500 text-sm"
						>
							Нет данных
						</div>
					{:else}
						<canvas bind:this={sccCanvas}></canvas>
					{/if}
				</div>
			</div>
		</div>
	{/if}

	<!-- Predictive Analytics -->
	{#if healthIndex || mastitisRisk || estrusPred || ketosisWarn || cullingRisk || energyBalance || feedRec || lifetimeVal || cowCluster || milkForecast}
		<div class="mb-6">
			<h2
				class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4 flex items-center gap-2"
			>
				<TrendingUp size={20} class="text-indigo-500" />
				Предиктивная аналитика
			</h2>

			<div
				class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4 mb-6"
			>
				{#if healthIndex}
					<div
						class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
					>
						<div class="flex items-center gap-2 mb-1">
							<ShieldCheck size={14} class="text-indigo-500" />
							<span class="text-xs text-slate-500 dark:text-slate-400"
								>Индекс здоровья</span
							>
						</div>
						<div class="flex items-baseline gap-2">
							<p
								class="text-lg font-semibold {healthIndex.health_score >= 80 ? 'text-green-600' : healthIndex.health_score >= 60 ? 'text-yellow-600' : healthIndex.health_score >= 40 ? 'text-orange-600' : 'text-red-600'}"
							>
								{healthIndex.health_score.toFixed(0)}
							</p>
							<span
								class="text-xs px-1.5 py-0.5 rounded {healthIndex.risk_level === 'low' ? 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-400' : healthIndex.risk_level === 'moderate' ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/50 dark:text-yellow-400' : healthIndex.risk_level === 'high' ? 'bg-orange-100 text-orange-700 dark:bg-orange-900/50 dark:text-orange-400' : 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-400'}"
							>
								{healthIndex.risk_level}
							</span>
						</div>
						{#if healthIndex.top_concern}
							<p class="text-xs text-slate-400 mt-1">{healthIndex.top_concern}</p>
						{/if}
					</div>
				{/if}

				{#if mastitisRisk}
					<div
						class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
					>
						<div class="flex items-center gap-2 mb-1">
							<Activity size={14} class="text-rose-500" />
							<span class="text-xs text-slate-500 dark:text-slate-400"
								>Риск мастита</span
							>
						</div>
						<div class="flex items-baseline gap-2">
							<p
								class="text-lg font-semibold {mastitisRisk.risk_level === 'high' ? 'text-red-600' : mastitisRisk.risk_level === 'medium' ? 'text-orange-600' : 'text-green-600'}"
							>
								{(mastitisRisk.risk_score * 100).toFixed(0)}%
							</p>
							<span
								class="text-xs px-1.5 py-0.5 rounded {mastitisRisk.risk_level === 'high' ? 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-400' : mastitisRisk.risk_level === 'medium' ? 'bg-orange-100 text-orange-700 dark:bg-orange-900/50 dark:text-orange-400' : 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-400'}"
							>
								{mastitisRisk.risk_level}
							</span>
						</div>
						{#if mastitisRisk.contributing_factors.length > 0}
							<p class="text-xs text-slate-400 mt-1">
								{mastitisRisk.contributing_factors.join(', ')}
							</p>
						{/if}
					</div>
				{/if}

				{#if estrusPred}
					<div
						class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
					>
						<div class="flex items-center gap-2 mb-1">
							<ThermometerSun size={14} class="text-pink-500" />
							<span class="text-xs text-slate-500 dark:text-slate-400"
								>Детекция охоты</span
							>
						</div>
						<div class="flex items-baseline gap-2">
							<p
								class="text-lg font-semibold {estrusPred.status === 'in_estrus' ? 'text-pink-600' : estrusPred.status === 'approaching' ? 'text-orange-600' : 'text-slate-600 dark:text-slate-400'}"
							>
								{(estrusPred.estrus_probability * 100).toFixed(0)}%
							</p>
							<span
								class="text-xs px-1.5 py-0.5 rounded {estrusPred.status === 'in_estrus' ? 'bg-pink-100 text-pink-700 dark:bg-pink-900/50 dark:text-pink-400' : estrusPred.status === 'approaching' ? 'bg-orange-100 text-orange-700 dark:bg-orange-900/50 dark:text-orange-400' : 'bg-slate-100 text-slate-600 dark:bg-slate-700 dark:text-slate-400'}"
							>
								{estrusPred.status}
							</span>
						</div>
						{#if estrusPred.optimal_window}
							<p class="text-xs text-slate-400 mt-1">{estrusPred.optimal_window}</p>
						{/if}
					</div>
				{/if}

				{#if ketosisWarn}
					<div
						class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
					>
						<div class="flex items-center gap-2 mb-1">
							<Droplets size={14} class="text-amber-500" />
							<span class="text-xs text-slate-500 dark:text-slate-400"
								>Риск кетоза</span
							>
						</div>
						<div class="flex items-baseline gap-2">
							<p
								class="text-lg font-semibold {ketosisWarn.severity === 'high' ? 'text-red-600' : ketosisWarn.severity === 'moderate' ? 'text-orange-600' : 'text-green-600'}"
							>
								{(ketosisWarn.risk_probability * 100).toFixed(0)}%
							</p>
							<span
								class="text-xs px-1.5 py-0.5 rounded {ketosisWarn.severity === 'high' ? 'bg-red-100 text-red-700' : ketosisWarn.severity === 'moderate' ? 'bg-orange-100 text-orange-700' : 'bg-green-100 text-green-700'}"
							>
								{ketosisWarn.severity}
							</span>
						</div>
						<p class="text-xs text-slate-400 mt-1">
							FPR: {ketosisWarn.fpr_current.toFixed(2)}
						</p>
					</div>
				{/if}

				{#if cullingRisk}
					<div
						class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
					>
						<div class="flex items-center gap-2 mb-1">
							<Activity size={14} class="text-slate-500" />
							<span class="text-xs text-slate-500 dark:text-slate-400"
								>Риск выбраковки</span
							>
						</div>
						<div class="flex items-baseline gap-2">
							<p
								class="text-lg font-semibold {cullingRisk.risk_score > 0.5 ? 'text-red-600' : cullingRisk.risk_score > 0.3 ? 'text-orange-600' : 'text-green-600'}"
							>
								{(cullingRisk.risk_score * 100).toFixed(0)}%
							</p>
							{#if cullingRisk.expected_days_remaining}
								<span class="text-xs text-slate-400"
									>~{cullingRisk.expected_days_remaining}д</span
								>
							{/if}
						</div>
						{#if cullingRisk.risk_factors.length > 0}
							<p class="text-xs text-slate-400 mt-1">
								{cullingRisk.risk_factors.join(', ')}
							</p>
						{/if}
					</div>
				{/if}

				{#if energyBalance}
					<div
						class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
					>
						<div class="flex items-center gap-2 mb-1">
							<ThermometerSun size={14} class="text-teal-500" />
							<span class="text-xs text-slate-500 dark:text-slate-400"
								>Энергобаланс (FPR)</span
							>
						</div>
						<div class="flex items-baseline gap-2">
							<p
								class="text-lg font-semibold {energyBalance.status === 'optimal' ? 'text-green-600' : energyBalance.status === 'ketosis_risk' || energyBalance.status === 'acidosis_risk' ? 'text-red-600' : 'text-slate-600 dark:text-slate-400'}"
							>
								{energyBalance.fat_protein_ratio?.toFixed(2) ?? '—'}
							</p>
							<span
								class="text-xs px-1.5 py-0.5 rounded {energyBalance.status === 'optimal' ? 'bg-green-100 text-green-700' : energyBalance.status === 'ketosis_risk' ? 'bg-amber-100 text-amber-700' : 'bg-slate-100 text-slate-600'}"
							>
								{energyBalance.status}
							</span>
						</div>
						{#if energyBalance.trend_30d != null}
							<p class="text-xs text-slate-400 mt-1">
								Тренд 30д: {energyBalance.trend_30d > 0 ? '+' : ''}{(energyBalance.trend_30d * 100).toFixed(0)}%
							</p>
						{/if}
					</div>
				{/if}

				{#if feedRec}
					<div
						class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
					>
						<div class="flex items-center gap-2 mb-1">
							<TrendingUp size={14} class="text-lime-500" />
							<span class="text-xs text-slate-500 dark:text-slate-400"
								>Рекомендация корма</span
							>
						</div>
						<div class="flex items-baseline gap-2">
							<p class="text-lg font-semibold text-slate-800 dark:text-slate-100">
								{feedRec.recommended_feed.toFixed(1)} кг
							</p>
							<span
								class="text-xs px-1.5 py-0.5 rounded {feedRec.difference_kg > 0 ? 'bg-amber-100 text-amber-700' : feedRec.difference_kg < -0.5 ? 'bg-blue-100 text-blue-700' : 'bg-green-100 text-green-700'}"
							>
								{feedRec.difference_kg > 0 ? '+' : ''}{feedRec.difference_kg.toFixed(1)}
							</span>
						</div>
						<p class="text-xs text-slate-400 mt-1">
							Текущий: {feedRec.current_feed_avg.toFixed(1)} кг
						</p>
					</div>
				{/if}

				{#if lifetimeVal}
					<div
						class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
					>
						<div class="flex items-center gap-2 mb-1">
							<Milk size={14} class="text-violet-500" />
							<span class="text-xs text-slate-500 dark:text-slate-400"
								>Пожизненная ценность</span
							>
						</div>
						<div class="flex items-baseline gap-2">
							<p
								class="text-lg font-semibold {(lifetimeVal.projected_net_value ?? 0) >= 0 ? 'text-green-600' : 'text-red-600'}"
							>
								{((lifetimeVal.projected_net_value ?? 0) / 1000).toFixed(0)}к
							</p>
							<span
								class="text-xs px-1.5 py-0.5 rounded {lifetimeVal.recommendation === 'keep' ? 'bg-green-100 text-green-700' : lifetimeVal.recommendation === 'culling_candidate' ? 'bg-red-100 text-red-700' : 'bg-yellow-100 text-yellow-700'}"
							>
								{lifetimeVal.recommendation}
							</span>
						</div>
						<p class="text-xs text-slate-400 mt-1">
							Лактаций осталось: {lifetimeVal.estimated_remaining_lactations}
						</p>
					</div>
				{/if}

				{#if cowCluster}
					<div
						class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4"
					>
						<div class="flex items-center gap-2 mb-1">
							<CircleDot size={14} class="text-sky-500" />
							<span class="text-xs text-slate-500 dark:text-slate-400"
								>Кластер</span
							>
						</div>
						<p class="text-lg font-semibold text-slate-800 dark:text-slate-100">
							{cowCluster.cluster_name}
						</p>
						<p class="text-xs text-slate-400 mt-1">
							Надой: {cowCluster.avg_milk.toFixed(1)}л / Жвачка: {cowCluster.avg_rumination.toFixed(0)}мин
						</p>
					</div>
				{/if}
			</div>

			{#if milkForecast && milkForecast.forecast.length > 0}
				<div
					class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-6"
				>
					<h2
						class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3 flex items-center gap-2"
					>
						<TrendingUp size={16} class="text-green-500" />
						Прогноз удоя 30 дней
						{#if milkForecast.current_daily_avg}
							<span class="text-xs text-slate-400 font-normal">
								(текущий: {milkForecast.current_daily_avg.toFixed(1)} л/день)
							</span>
						{/if}
					</h2>
					<div class="relative h-64">
						<canvas bind:this={forecastCanvas}></canvas>
					</div>
				</div>
			{/if}
		</div>
	{/if}

	<!-- Basic Info Cards -->
	<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4">
				Основная информация
			</h2>
			<dl class="space-y-3">
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Пол</dt>
					<dd>
						<span
							class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {animal.gender ===
							'female'
								? 'bg-pink-100 dark:bg-pink-900/50 text-pink-700'
								: 'bg-blue-100 dark:bg-blue-900/50 text-blue-700'}"
						>
							{animal.gender === 'female' ? 'Корова' : 'Бык'}
						</span>
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Дата рождения</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">{animal.birth_date}</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Статус</dt>
					<dd>
						<span
							class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {animal.active
								? 'bg-green-100 dark:bg-green-900/50 text-green-700'
								: 'bg-slate-100 dark:bg-slate-900 text-slate-500 dark:text-slate-400'}"
						>
							{animal.active ? 'Активно' : 'Неактивно'}
						</span>
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Локация</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">
						{animal.location || '—'}
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Группа</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">
						{animal.group_number ?? '—'}
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Код масти</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">
						{animal.hair_color_code || '—'}
					</dd>
				</div>
			</dl>
		</div>

		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4">
				Номера и идентификация
			</h2>
			<dl class="space-y-3">
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Жизненный номер</dt>
					<dd class="text-sm font-mono text-slate-800 dark:text-slate-100">
						{animal.life_number || '—'}
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Пользовательский номер</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">
						{animal.user_number ?? '—'}
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">UCN номер</dt>
					<dd class="text-sm font-mono text-slate-800 dark:text-slate-100">
						{animal.ucn_number || '—'}
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Номер отца</dt>
					<dd class="text-sm font-mono text-slate-800 dark:text-slate-100">
						{animal.father_life_number || '—'}
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Номер матери</dt>
					<dd class="text-sm font-mono text-slate-800 dark:text-slate-100">
						{animal.mother_life_number || '—'}
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Номер респондера</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">
						{animal.responder_number || '—'}
					</dd>
				</div>
			</dl>
		</div>
	</div>

	{#if animal.description}
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 mt-6"
		>
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">Описание</h2>
			<p class="text-sm text-slate-600 dark:text-slate-400 whitespace-pre-wrap">
				{animal.description}
			</p>
		</div>
	{/if}

	<!-- Timeline -->
	<div
		class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 mt-6"
	>
		<div class="flex items-center justify-between mb-4">
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300">История событий</h2>
			<span class="text-xs text-slate-400">Всего: {timelineTotal}</span>
		</div>

		{#if timelineLoading}
			<div class="animate-pulse space-y-4">
				{#each Array(5) as _, i (i)}
					<div class="flex gap-3">
						<div class="h-8 w-8 bg-slate-200 dark:bg-slate-700 rounded-full"></div>
						<div class="flex-1 space-y-1">
							<div class="h-3 bg-slate-200 dark:bg-slate-700 rounded w-1/4"></div>
							<div class="h-3 bg-slate-200 dark:bg-slate-700 rounded w-2/3"></div>
						</div>
					</div>
				{/each}
			</div>
		{:else if timeline.length === 0}
			<p class="text-sm text-slate-400 text-center py-8">Нет событий</p>
		{:else}
			<div class="space-y-0">
				{#each timeline as event, i (i)}
					{@const Icon = eventIcon(event.event_type)}
					<div class="flex gap-3 relative {i < timeline.length - 1 ? 'pb-6' : ''}">
						{#if i < timeline.length - 1}
							<div class="absolute left-4 top-9 bottom-0 w-px bg-slate-200 dark:bg-slate-700"></div>
						{/if}
						<div
							class="flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center {eventColor(
								event.event_type,
							)}"
						>
							<Icon size={14} />
						</div>
						<div class="flex-1 min-w-0 pt-0.5">
							<div class="flex items-center gap-2 mb-0.5">
								<span class="text-sm font-medium text-slate-700 dark:text-slate-300"
									>{event.event_type}</span
								>
								<span class="text-xs text-slate-400">{event.date}</span>
							</div>
							<p class="text-sm text-slate-500 dark:text-slate-400 truncate">
								{event.description || '—'}
							</p>
						</div>
					</div>
				{/each}
			</div>

			{#if timelineTotal > 30}
				<div
					class="flex justify-center gap-2 mt-4 pt-4 border-t border-slate-200 dark:border-slate-700"
				>
					<button
						onclick={() => {
							timelinePage--;
							loadTimeline();
						}}
						disabled={timelinePage <= 1}
						class="px-3 py-1 text-sm border border-slate-300 dark:border-slate-600 rounded hover:bg-slate-100 dark:hover:bg-slate-700 disabled:opacity-50 cursor-pointer"
					>
						Назад
					</button>
					<span class="px-3 py-1 text-sm text-slate-600 dark:text-slate-400"
						>{timelinePage} / {Math.ceil(timelineTotal / 30)}</span
					>
					<button
						onclick={() => {
							timelinePage++;
							loadTimeline();
						}}
						disabled={timelinePage >= Math.ceil(timelineTotal / 30)}
						class="px-3 py-1 text-sm border border-slate-300 dark:border-slate-600 rounded hover:bg-slate-100 dark:hover:bg-slate-700 disabled:opacity-50 cursor-pointer"
					>
						Вперёд
					</button>
				</div>
			{/if}
		{/if}
	</div>
{/if}

<ConfirmDialog
	open={showDelete}
	title="Удалить животное?"
	message="Это действие нельзя отменить."
	loading={deleteLoading}
	onconfirm={handleDelete}
	oncancel={() => (showDelete = false)}
/>
