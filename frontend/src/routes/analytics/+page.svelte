<script lang="ts">
	import { onMount } from 'svelte';
	import TabBar from '$lib/components/ui/TabBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import LactationCurveChart from '$lib/components/LactationCurveChart.svelte';
	import SeasonalChart from '$lib/components/SeasonalChart.svelte';
	import {
		getLactationCurves,
		getHealthIndex,
		getFertilityWindow,
		getProfitability,
		getSeasonal,
		getMastitisRisk,
		getCullingSurvival,
		getEnergyBalance,
		getQuarterHealth,
		getMilkForecast,
		getCowClusters,
		getEstrusDetection,
		getEquipmentAnomaly,
		getFeedRecommendation,
		getKetosisWarning,
		getFeedEfficiency,
		getDryOffOptimizer,
		getLifetimeValue,
		type LactationCurveResponse,
		type HealthIndexResponse,
		type FertilityWindowResponse,
		type ProfitabilityResponse,
		type SeasonalResponse,
		type MastitisRiskResponse,
		type CullingSurvivalResponse,
		type EnergyBalanceResponse,
		type QuarterHealthResponse,
		type MilkForecastResponse,
		type ClusterResponse,
		type EstrusResponse,
		type EquipmentAnomalyResponse,
		type FeedRecommendationResponse,
		type KetosisWarningResponse,
		type FeedEfficiencyResponse,
		type DryOffOptimizerResponse,
		type LifetimeValueResponse,
	} from '$lib/api/analytics';

	type AnalyticsTab =
		| 'lactation'
		| 'health'
		| 'fertility'
		| 'profit'
		| 'seasonal'
		| 'mastitis'
		| 'culling'
		| 'energy'
		| 'udder'
		| 'forecast'
		| 'clusters'
		| 'estrus'
		| 'equipment'
		| 'feedRec'
		| 'ketosis'
		| 'feedEff'
		| 'dryOff'
		| 'lifetime';

	const tabs: { key: AnalyticsTab; label: string }[] = [
		{ key: 'lactation', label: 'Кривые лактации' },
		{ key: 'health', label: 'Индекс здоровья' },
		{ key: 'fertility', label: 'Окно фертильности' },
		{ key: 'profit', label: 'Рентабельность' },
		{ key: 'seasonal', label: 'Сезонность' },
		{ key: 'mastitis', label: 'Риск мастита' },
		{ key: 'culling', label: 'Выбраковка' },
		{ key: 'energy', label: 'Энергобаланс' },
		{ key: 'udder', label: 'Здоровье вымени' },
		{ key: 'forecast', label: 'Прогноз 30д' },
		{ key: 'clusters', label: 'Кластеры' },
		{ key: 'estrus', label: 'Детекция охоты' },
		{ key: 'equipment', label: 'Оборудование' },
		{ key: 'feedRec', label: 'Рекомендация корма' },
		{ key: 'ketosis', label: 'Кетоз/ацидоз' },
		{ key: 'feedEff', label: 'Конверсия корма' },
		{ key: 'dryOff', label: 'Запуск' },
		{ key: 'lifetime', label: 'Ценность коровы' },
	];

	let activeTab: AnalyticsTab = $state('lactation');
	let loading = $state(false);
	let error = $state('');

	let lactationData: LactationCurveResponse[] = $state([]);
	let healthData: HealthIndexResponse | null = $state(null);
	let fertilityData: FertilityWindowResponse | null = $state(null);
	let profitData: ProfitabilityResponse | null = $state(null);
	let seasonalData: SeasonalResponse | null = $state(null);
	let mastitisData: MastitisRiskResponse | null = $state(null);
	let cullingData: CullingSurvivalResponse | null = $state(null);
	let energyData: EnergyBalanceResponse | null = $state(null);
	let udderData: QuarterHealthResponse | null = $state(null);
	let forecastData: MilkForecastResponse | null = $state(null);
	let clusterData: ClusterResponse | null = $state(null);
	let estrusData: EstrusResponse | null = $state(null);
	let equipmentData: EquipmentAnomalyResponse | null = $state(null);
	let feedRecData: FeedRecommendationResponse | null = $state(null);
	let ketosisData: KetosisWarningResponse | null = $state(null);
	let feedEffData: FeedEfficiencyResponse | null = $state(null);
	let dryOffData: DryOffOptimizerResponse | null = $state(null);
	let lifetimeData: LifetimeValueResponse | null = $state(null);

	let selectedAnimalId = $state<number | ''>('');
	let forecastAnimalId = $state<number | ''>('');
	let milkPrice = $state(25);
	let feedPrice = $state(12);

	let lactationDetailAnimal = $state<number | null>(null);

	async function load() {
		try {
			loading = true;
			error = '';

			if (activeTab === 'lactation') {
				lactationData = await getLactationCurves(
					selectedAnimalId || undefined,
				);
			} else if (activeTab === 'health') {
				healthData = await getHealthIndex();
			} else if (activeTab === 'fertility') {
				fertilityData = await getFertilityWindow();
			} else if (activeTab === 'profit') {
				profitData = await getProfitability(milkPrice, feedPrice);
			} else if (activeTab === 'seasonal') {
				seasonalData = await getSeasonal();
			} else if (activeTab === 'mastitis') {
				mastitisData = await getMastitisRisk();
			} else if (activeTab === 'culling') {
				cullingData = await getCullingSurvival();
			} else if (activeTab === 'energy') {
				energyData = await getEnergyBalance();
			} else if (activeTab === 'udder') {
				udderData = await getQuarterHealth();
			} else if (activeTab === 'forecast') {
				if (forecastAnimalId) {
					forecastData = await getMilkForecast(Number(forecastAnimalId), 30);
				}
			} else if (activeTab === 'clusters') {
				clusterData = await getCowClusters();
			} else if (activeTab === 'estrus') {
				estrusData = await getEstrusDetection();
			} else if (activeTab === 'equipment') {
				equipmentData = await getEquipmentAnomaly();
			} else if (activeTab === 'feedRec') {
				feedRecData = await getFeedRecommendation();
			} else if (activeTab === 'ketosis') {
				ketosisData = await getKetosisWarning();
			} else if (activeTab === 'feedEff') {
				feedEffData = await getFeedEfficiency();
			} else if (activeTab === 'dryOff') {
				dryOffData = await getDryOffOptimizer();
			} else if (activeTab === 'lifetime') {
				lifetimeData = await getLifetimeValue();
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			loading = false;
		}
	}

	function switchTab(key: string) {
		activeTab = key as AnalyticsTab;
		lactationDetailAnimal = null;
		load();
	}

	onMount(load);

	const thCls =
		'px-3 py-2 text-left text-xs font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider';
	const tdCls = 'px-3 py-2 text-sm text-slate-700 dark:text-slate-300 whitespace-nowrap';
	const tblCls =
		'min-w-full divide-y divide-slate-200 dark:divide-slate-700';

	function riskBadge(level: string) {
		const m: Record<string, string> = {
			critical:
				'bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-400',
			high: 'bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-400',
			medium:
				'bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-400',
			moderate:
				'bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-400',
			low: 'bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-400',
			optimal:
				'bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-400',
			approaching:
				'bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-400',
			in_window:
				'bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-400',
			outside_window:
				'bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-500',
		};
		return m[level] ?? m['low'];
	}

	function scoreColor(score: number, max = 100): string {
		const pct = score / max;
		if (pct >= 0.8) return 'text-green-600 dark:text-green-400';
		if (pct >= 0.6) return 'text-yellow-600 dark:text-yellow-400';
		if (pct >= 0.4) return 'text-orange-600 dark:text-orange-400';
		return 'text-red-600 dark:text-red-400';
	}
</script>

<svelte:head>
	<title>Предиктивная аналитика — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-4"
	>Предиктивная аналитика</h1
>

<TabBar {tabs} bind:active={activeTab} onchange={switchTab} />

<ErrorAlert message={error} />

{#if loading}
	<div class="space-y-3">
		{#each Array(5) as _, i (i)}
			<div
				class="h-10 bg-slate-100 dark:bg-slate-900 rounded animate-pulse"
			></div>
		{/each}
	</div>
{:else}
	<!-- LACTATION CURVES -->
	{#if activeTab === 'lactation'}
		<div class="mb-4 flex items-center gap-3">
			<label class="text-sm text-slate-600 dark:text-slate-400"
				>ID коровы:</label
			>
			<input
				type="number"
				bind:value={selectedAnimalId}
				placeholder="Все"
				class="w-32 px-3 py-1.5 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
			/>
			<button
				onclick={load}
				class="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors cursor-pointer"
				>Показать</button
			>
		</div>

		{#if lactationDetailAnimal !== null}
			{@const curve = lactationData.find((c) => c.animal_id === lactationDetailAnimal)}
			{#if curve}
				<button
					onclick={() => (lactationDetailAnimal = null)}
					class="mb-3 text-sm text-blue-600 dark:text-blue-400 hover:underline cursor-pointer"
					>&larr; Назад к списку</button
				>
				<div
					class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-4"
				>
					<h3 class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-2"
						>{curve.animal_name ?? `#${curve.animal_id}`}
						— Лактация {curve.lac_number} (DIM {curve.current_dim})</h3
					>
					<div
						class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-4 text-sm"
					>
						<div>
							<span class="text-slate-500 dark:text-slate-400">Пик</span>
							<p class="font-semibold">{curve.peak_milk?.toFixed(1) ?? '—'} л (DIM {curve.peak_dim ?? '—'})</p>
						</div>
						<div>
							<span class="text-slate-500 dark:text-slate-400">Прогноз 305д</span>
							<p class="font-semibold">{curve.predicted_total_305d?.toFixed(0) ?? '—'} л</p>
						</div>
						<div>
							<span class="text-slate-500 dark:text-slate-400">Отёл</span>
							<p class="font-semibold">{curve.calving_date}</p>
						</div>
						<div>
							<span class="text-slate-500 dark:text-slate-400">DIM</span>
							<p class="font-semibold">{curve.current_dim}</p>
						</div>
					</div>
					<div class="h-80">
						<LactationCurveChart data={curve} />
					</div>
				</div>
			{/if}
		{:else}
			<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
				<table class={tblCls}>
					<thead class="bg-slate-50 dark:bg-slate-900">
						<tr>
							<th class={thCls}>Корова</th>
							<th class={thCls}>Лакт.</th>
							<th class={thCls}>DIM</th>
							<th class={thCls}>Пик, л</th>
							<th class={thCls}>Пик DIM</th>
							<th class={thCls}>305д прогноз, л</th>
							<th class={thCls}>Отёл</th>
						</tr>
					</thead>
					<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
						{#each lactationData as c (c.animal_id)}
							<tr
								class="hover:bg-slate-50 dark:hover:bg-slate-700/50 cursor-pointer"
								onclick={() => (lactationDetailAnimal = c.animal_id)}
							>
								<td class={tdCls}>{c.animal_name ?? `#${c.animal_id}`}</td>
								<td class={tdCls}>{c.lac_number}</td>
								<td class={tdCls}>{c.current_dim}</td>
								<td class={tdCls}>{c.peak_milk?.toFixed(1) ?? '—'}</td>
								<td class={tdCls}>{c.peak_dim ?? '—'}</td>
								<td class={tdCls}>{c.predicted_total_305d?.toFixed(0) ?? '—'}</td>
								<td class={tdCls}>{c.calving_date}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}

	<!-- HEALTH INDEX -->
	{:else if activeTab === 'health' && healthData}
		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Корова</th>
						<th class={thCls}>Score</th>
						<th class={thCls}>Риск</th>
						<th class={thCls}>Надой Z</th>
						<th class={thCls}>Жвачка Z</th>
						<th class={thCls}>Активн. Z</th>
						<th class={thCls}>SCC Z</th>
						<th class={thCls}>Главная проблема</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each healthData.cows as c (c.animal_id)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{c.animal_name ?? `#${c.animal_id}`}</td>
							<td class={`${tdCls} font-semibold ${scoreColor(c.health_score)}`}>{c.health_score.toFixed(1)}</td>
							<td class={tdCls}>
								<span class="px-2 py-0.5 rounded-full text-xs font-medium {riskBadge(c.risk_level)}">{c.risk_level}</span>
							</td>
							<td class={tdCls}>{c.milk_deviation_zscore?.toFixed(2) ?? '—'}</td>
							<td class={tdCls}>{c.rumination_deviation_zscore?.toFixed(2) ?? '—'}</td>
							<td class={tdCls}>{c.activity_deviation_zscore?.toFixed(2) ?? '—'}</td>
							<td class={tdCls}>{c.scc_deviation_zscore?.toFixed(2) ?? '—'}</td>
							<td class={tdCls}>{c.top_concern ?? '—'}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

	<!-- FERTILITY WINDOW -->
	{:else if activeTab === 'fertility' && fertilityData}
		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Корова</th>
						<th class={thCls}>DIM</th>
						<th class={thCls}>Активн.</th>
						<th class={thCls}>Жвачка</th>
						<th class={thCls}>Надой</th>
						<th class={thCls}>Скор</th>
						<th class={thCls}>Статус</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each fertilityData.cows as c (c.animal_id)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{c.animal_name ?? `#${c.animal_id}`}</td>
							<td class={tdCls}>{c.days_since_calving ?? '—'}</td>
							<td class={tdCls}>{c.activity_signal !== null ? (c.activity_signal > 1 ? `↑${c.activity_signal.toFixed(2)}` : c.activity_signal.toFixed(2)) : '—'}</td>
							<td class={tdCls}>{c.rumination_signal !== null ? (c.rumination_signal < 1 ? `↓${c.rumination_signal.toFixed(2)}` : c.rumination_signal.toFixed(2)) : '—'}</td>
							<td class={tdCls}>{c.milk_signal !== null ? (c.milk_signal < 1 ? `↓${c.milk_signal.toFixed(2)}` : c.milk_signal.toFixed(2)) : '—'}</td>
							<td class={`${tdCls} font-semibold ${scoreColor(c.combined_score, 100)}`}>{c.combined_score.toFixed(0)}</td>
							<td class={tdCls}>
								<span class="px-2 py-0.5 rounded-full text-xs font-medium {riskBadge(c.window_status)}">{c.window_status}</span>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

	<!-- PROFITABILITY -->
	{:else if activeTab === 'profit' && profitData}
		<div class="mb-4 flex items-center gap-4">
			<label class="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-400">
				Цена молока:
				<input
					type="number"
					bind:value={milkPrice}
					min="1"
					class="w-20 px-2 py-1 text-sm border border-slate-300 dark:border-slate-600 rounded bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
				/>
				руб/л
			</label>
			<label class="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-400">
				Цена корма:
				<input
					type="number"
					bind:value={feedPrice}
					min="1"
					class="w-20 px-2 py-1 text-sm border border-slate-300 dark:border-slate-600 rounded bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
				/>
				руб/кг
			</label>
			<button
				onclick={load}
				class="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors cursor-pointer"
				>Пересчитать</button
			>
		</div>

		{#if profitData.herd_avg_margin_day !== null}
			<div class="mb-4 grid grid-cols-1 sm:grid-cols-3 gap-3">
				<div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4">
					<div class="text-xs text-slate-500 dark:text-slate-400 mb-1">Средняя маржа/день</div>
					<div class="text-lg font-bold {profitData.herd_avg_margin_day >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}">{profitData.herd_avg_margin_day.toFixed(1)} руб</div>
				</div>
			</div>
		{/if}

		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Корова</th>
						<th class={thCls}>Надой, л/д</th>
						<th class={thCls}>Корм, кг/д</th>
						<th class={thCls}>Выручка</th>
						<th class={thCls}>Расход</th>
						<th class={thCls}>Маржа/день</th>
						<th class={thCls}>Маржа 30д</th>
						<th class={thCls}>Доля корма</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each profitData.cows as c (c.animal_id)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{c.animal_name ?? `#${c.animal_id}`}</td>
							<td class={tdCls}>{c.avg_daily_milk?.toFixed(1) ?? '—'}</td>
							<td class={tdCls}>{c.avg_daily_feed?.toFixed(1) ?? '—'}</td>
							<td class={tdCls}>{c.estimated_milk_revenue_day?.toFixed(1) ?? '—'}</td>
							<td class={tdCls}>{c.estimated_feed_cost_day?.toFixed(1) ?? '—'}</td>
							<td class={`${tdCls} font-semibold ${(c.estimated_margin_day ?? 0) >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}`}>{c.estimated_margin_day?.toFixed(1) ?? '—'}</td>
							<td class={tdCls}>{c.margin_30d?.toFixed(0) ?? '—'}</td>
							<td class={tdCls}>{c.feed_cost_ratio !== null ? `${(c.feed_cost_ratio * 100).toFixed(0)}%` : '—'}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

	<!-- SEASONAL -->
	{:else if activeTab === 'seasonal' && seasonalData}
		<div class="mb-4 grid grid-cols-1 sm:grid-cols-3 gap-3">
			<div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4">
				<div class="text-xs text-slate-500 dark:text-slate-400 mb-1">Тренд 7д</div>
				<div class="text-lg font-bold">{seasonalData.trend_7d?.toFixed(1) ?? '—'} л/д</div>
			</div>
			<div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4">
				<div class="text-xs text-slate-500 dark:text-slate-400 mb-1">Тренд 30д</div>
				<div class="text-lg font-bold">{seasonalData.trend_30d?.toFixed(1) ?? '—'} л/д</div>
			</div>
			<div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4">
				<div class="text-xs text-slate-500 dark:text-slate-400 mb-1">Текущий сезонный фактор</div>
				<div class="text-lg font-bold">{seasonalData.current_seasonal_factor?.toFixed(2) ?? '—'}</div>
			</div>
		</div>

		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-4"
		>
			<h3 class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3"
				>Сезонный индекс надоев</h3
			>
			<div class="h-72">
				<SeasonalChart data={seasonalData} />
			</div>
		</div>

		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Месяц</th>
						<th class={thCls}>Средний надой, л/д</th>
						<th class={thCls}>Сезонный индекс</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each seasonalData.monthly_indices as m (m.month)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{m.month_name}</td>
							<td class={tdCls}>{m.avg_daily_milk?.toFixed(1) ?? '—'}</td>
							<td class={tdCls}>
								{#if m.seasonal_index !== null}
									<span class="font-medium {(m.seasonal_index ?? 1) >= 1 ? 'text-green-600 dark:text-green-400' : 'text-orange-600 dark:text-orange-400'}">{m.seasonal_index.toFixed(2)}</span>
								{:else}
									—
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

	<!-- MASTITIS RISK -->
	{:else if activeTab === 'mastitis' && mastitisData}
		<div class="mb-2 text-xs text-slate-400 dark:text-slate-500"
			>Модель: {mastitisData.model_version}</div
		>
		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Корова</th>
						<th class={thCls}>Риск</th>
						<th class={thCls}>Уровень</th>
						<th class={thCls}>Факторы</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each mastitisData.cows as c (c.animal_id)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{c.animal_name ?? `#${c.animal_id}`}</td>
							<td class={`${tdCls} font-semibold ${(c.risk_score) >= 0.6 ? 'text-red-600 dark:text-red-400' : (c.risk_score >= 0.3 ? 'text-orange-600 dark:text-orange-400' : 'text-yellow-600 dark:text-yellow-400')}`}>{(c.risk_score * 100).toFixed(0)}%</td>
							<td class={tdCls}>
								<span class="px-2 py-0.5 rounded-full text-xs font-medium {riskBadge(c.risk_level)}">{c.risk_level}</span>
							</td>
							<td class={tdCls}>
								{#each c.contributing_factors as f, i}
									{#if i > 0}<span class="text-slate-300 dark:text-slate-600 mx-1">·</span>{/if}
									<span class="text-xs">{f}</span>
								{/each}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

	<!-- CULLING SURVIVAL -->
	{:else if activeTab === 'culling' && cullingData}
		<div class="mb-2 text-xs text-slate-400 dark:text-slate-500"
			>Модель: {cullingData.model_version}</div
		>
		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Корова</th>
						<th class={thCls}>Риск</th>
						<th class={thCls}>Осталось дней</th>
						<th class={thCls}>Факторы</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each cullingData.cows as c (c.animal_id)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{c.animal_name ?? `#${c.animal_id}`}</td>
							<td class={`${tdCls} font-semibold ${(c.risk_score) >= 0.6 ? 'text-red-600 dark:text-red-400' : (c.risk_score >= 0.3 ? 'text-orange-600 dark:text-orange-400' : 'text-yellow-600 dark:text-yellow-400')}`}>{(c.risk_score * 100).toFixed(0)}%</td>
							<td class={tdCls}>{c.expected_days_remaining !== null ? `${c.expected_days_remaining} д` : '—'}</td>
							<td class={tdCls}>
								{#each c.risk_factors as f, i}
									{#if i > 0}<span class="text-slate-300 dark:text-slate-600 mx-1">·</span>{/if}
									<span class="text-xs">{f}</span>
								{/each}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

	<!-- ENERGY BALANCE -->
	{:else if activeTab === 'energy' && energyData}
		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Корова</th>
						<th class={thCls}>Жир, %</th>
						<th class={thCls}>Белок, %</th>
						<th class={thCls}>FPR</th>
						<th class={thCls}>Статус</th>
						<th class={thCls}>Тренд 7д</th>
						<th class={thCls}>Тренд 30д</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each energyData.cows as c (c.animal_id)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{c.animal_name ?? `#${c.animal_id}`}</td>
							<td class={tdCls}>{c.avg_fat_pct?.toFixed(2) ?? '—'}</td>
							<td class={tdCls}>{c.avg_protein_pct?.toFixed(2) ?? '—'}</td>
							<td class={`${tdCls} font-semibold`}>{c.fat_protein_ratio?.toFixed(2) ?? '—'}</td>
							<td class={tdCls}>
								<span class="px-2 py-0.5 rounded-full text-xs font-medium {riskBadge(c.status === 'optimal' ? 'optimal' : c.status === 'ketosis_risk' || c.status === 'acidosis_risk' ? 'high' : c.status === 'normal' ? 'low' : 'medium')}">{c.status}</span>
							</td>
							<td class={tdCls}>{c.trend_7d?.toFixed(2) ?? '—'}</td>
							<td class={tdCls}>{c.trend_30d !== null ? `${c.trend_30d >= 0 ? '+' : ''}${(c.trend_30d * 100).toFixed(1)}%` : '—'}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

	<!-- QUARTER HEALTH -->
	{:else if activeTab === 'udder' && udderData}
		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Корова</th>
						<th class={thCls}>LF</th>
						<th class={thCls}>LR</th>
						<th class={thCls}>RF</th>
						<th class={thCls}>RR</th>
						<th class={thCls}>Среднее</th>
						<th class={thCls}>Асимметрия</th>
						<th class={thCls}>Худшая доля</th>
						<th class={thCls}>Риск</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each udderData.cows as c (c.animal_id)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{c.animal_name ?? `#${c.animal_id}`}</td>
							<td class={tdCls}>{c.lf_conductivity?.toFixed(1) ?? '—'}</td>
							<td class={tdCls}>{c.lr_conductivity?.toFixed(1) ?? '—'}</td>
							<td class={tdCls}>{c.rf_conductivity?.toFixed(1) ?? '—'}</td>
							<td class={tdCls}>{c.rr_conductivity?.toFixed(1) ?? '—'}</td>
							<td class={tdCls}>{c.avg_conductivity?.toFixed(1) ?? '—'}</td>
							<td class={`${tdCls} ${(c.max_asymmetry ?? 0) > 5 ? 'text-orange-600 dark:text-orange-400 font-semibold' : ''}`}>{c.max_asymmetry?.toFixed(1) ?? '—'}</td>
							<td class={tdCls}>{c.worst_quarter ?? '—'}</td>
							<td class={tdCls}>
								<span class="px-2 py-0.5 rounded-full text-xs font-medium {riskBadge(c.risk_level)}">{c.risk_level}</span>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

	<!-- MILK FORECAST 30D -->
	{:else if activeTab === 'forecast'}
		<div class="mb-4 flex items-center gap-3">
			<label class="text-sm text-slate-600 dark:text-slate-400">ID коровы:</label>
			<input
				type="number"
				bind:value={forecastAnimalId}
				placeholder="Обязательное поле"
				class="w-32 px-3 py-1.5 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
			/>
			<button
				onclick={load}
				class="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors cursor-pointer"
				disabled={!forecastAnimalId}
			>Прогноз</button>
		</div>

		{#if forecastData}
			<div class="mb-4 grid grid-cols-1 sm:grid-cols-3 gap-3">
				<div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4">
					<div class="text-xs text-slate-500 dark:text-slate-400 mb-1">Текущий средний надой</div>
					<div class="text-lg font-bold">{forecastData.current_daily_avg?.toFixed(1) ?? '—'} л/д</div>
				</div>
				<div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4">
					<div class="text-xs text-slate-500 dark:text-slate-400 mb-1">Корова</div>
					<div class="text-lg font-bold">{forecastData.animal_name ?? `#${forecastData.animal_id}`}</div>
				</div>
				<div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4">
					<div class="text-xs text-slate-500 dark:text-slate-400 mb-1">Модель</div>
					<div class="text-lg font-bold">{forecastData.model_version}</div>
				</div>
			</div>
			<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
				<table class={tblCls}>
					<thead class="bg-slate-50 dark:bg-slate-900">
						<tr>
							<th class={thCls}>День</th>
							<th class={thCls}>Прогноз, л</th>
							<th class={thCls}>Нижняя граница</th>
							<th class={thCls}>Верхняя граница</th>
						</tr>
					</thead>
					<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
						{#each forecastData.forecast as d (d.day_offset)}
							<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
								<td class={tdCls}>+{d.day_offset}</td>
								<td class={`${tdCls} font-semibold`}>{d.predicted_milk.toFixed(1)}</td>
								<td class={tdCls}>{d.lower_bound.toFixed(1)}</td>
								<td class={tdCls}>{d.upper_bound.toFixed(1)}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{:else if !forecastAnimalId}
			<div class="text-sm text-slate-500 dark:text-slate-400 py-8 text-center">Введите ID коровы для получения прогноза</div>
		{/if}

	<!-- COW CLUSTERS -->
	{:else if activeTab === 'clusters'}
		{#if clusterData && clusterData.clusters.length > 0}
			{#if Object.keys(clusterData.cluster_names).length > 0}
				<div class="mb-4 flex flex-wrap gap-2">
					{#each Object.entries(clusterData.cluster_names) as [id, name] (id)}
						<span class="px-3 py-1 rounded-full text-xs font-medium bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-400">
							{name} (#{id})
						</span>
					{/each}
				</div>
			{/if}

			<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
				<table class={tblCls}>
					<thead class="bg-slate-50 dark:bg-slate-900">
						<tr>
							<th class={thCls}>Корова</th>
							<th class={thCls}>Кластер</th>
							<th class={thCls}>Средний надой</th>
							<th class={thCls}>Средняя жвачка</th>
							<th class={thCls}>Расст. до центра</th>
						</tr>
					</thead>
					<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
						{#each clusterData.clusters.toSorted((a, b) => a.cluster_id - b.cluster_id) as c (c.animal_id)}
							<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
								<td class={tdCls}>{c.animal_name ?? `#${c.animal_id}`}</td>
								<td class={tdCls}>
									<span class="px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-400">{c.cluster_name}</span>
								</td>
								<td class={`${tdCls} font-semibold`}>{c.avg_milk.toFixed(1)} л</td>
								<td class={tdCls}>{c.avg_rumination.toFixed(0)} мин</td>
								<td class={tdCls}>{c.distance_to_center.toFixed(2)}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{:else if clusterData}
			<div class="text-sm text-slate-500 dark:text-slate-400 py-8 text-center">Нет данных для кластеризации</div>
		{:else}
			<div class="text-sm text-slate-500 dark:text-slate-400 py-8 text-center">Загрузка кластеров...</div>
		{/if}

	<!-- ESTRUS DETECTION -->
	{:else if activeTab === 'estrus' && estrusData}
		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Корова</th>
						<th class={thCls}>Вероятность</th>
						<th class={thCls}>Статус</th>
						<th class={thCls}>Окно</th>
						<th class={thCls}>Сигналы</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each estrusData.predictions.toSorted((a, b) => b.estrus_probability - a.estrus_probability) as p (p.animal_id)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{p.animal_name ?? `#${p.animal_id}`}</td>
							<td class={`${tdCls} font-semibold ${p.estrus_probability >= 0.7 ? 'text-red-600 dark:text-red-400' : p.estrus_probability >= 0.4 ? 'text-orange-600 dark:text-orange-400' : ''}`}>{(p.estrus_probability * 100).toFixed(0)}%</td>
							<td class={tdCls}>
								<span class="px-2 py-0.5 rounded-full text-xs font-medium {riskBadge(p.status === 'in_heat' ? 'high' : p.status === 'approaching' ? 'medium' : 'low')}">{p.status}</span>
							</td>
							<td class={tdCls}>{p.optimal_window ?? '—'}</td>
							<td class={tdCls}>
								{#each p.contributing_signals as s, i}
									{#if i > 0}<span class="text-slate-300 dark:text-slate-600 mx-1">·</span>{/if}
									<span class="text-xs">{s}</span>
								{/each}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

	<!-- EQUIPMENT ANOMALY -->
	{:else if activeTab === 'equipment' && equipmentData}
		{#if equipmentData.entries.filter(e => e.is_anomaly).length > 0}
			<div class="mb-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
				<div class="text-sm font-semibold text-red-700 dark:text-red-400">Обнаружено аномалий: {equipmentData.entries.filter(e => e.is_anomaly).length}</div>
			</div>
		{/if}
		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Корова</th>
						<th class={thCls}>Станок</th>
						<th class={thCls}>Аномалия</th>
						<th class={thCls}>Score</th>
						<th class={thCls}>Серьёзность</th>
						<th class={thCls}>Флаги</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each equipmentData.entries.toSorted((a, b) => a.anomaly_score - b.anomaly_score) as e (e.animal_id)}
						{#if e.is_anomaly}
							<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50 bg-red-50/50 dark:bg-red-900/10">
								<td class={tdCls}>{e.animal_name ?? `#${e.animal_id}`}</td>
								<td class={tdCls}>#{e.device_address ?? '—'}</td>
								<td class={`${tdCls} font-semibold text-red-600 dark:text-red-400`}>Да</td>
								<td class={tdCls}>{e.anomaly_score.toFixed(4)}</td>
								<td class={tdCls}>
									<span class="px-2 py-0.5 rounded-full text-xs font-medium {riskBadge(e.severity)}">{e.severity}</span>
								</td>
								<td class={tdCls}>
									{#each e.flags as f, i}
										{#if i > 0}<span class="text-slate-300 dark:text-slate-600 mx-1">·</span>{/if}
										<span class="text-xs">{f}</span>
									{/each}
								</td>
							</tr>
						{/if}
					{/each}
				</tbody>
			</table>
		</div>

	<!-- FEED RECOMMENDATION -->
	{:else if activeTab === 'feedRec' && feedRecData}
		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Корова</th>
						<th class={thCls}>DIM</th>
						<th class={thCls}>Лакт.</th>
						<th class={thCls}>Текущий корм</th>
						<th class={thCls}>Рекомендация</th>
						<th class={thCls}>Δ кг</th>
						<th class={thCls}>Действие</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each feedRecData.recommendations.toSorted((a, b) => Math.abs(b.difference_kg) - Math.abs(a.difference_kg)) as r (r.animal_id)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{r.animal_name ?? `#${r.animal_id}`}</td>
							<td class={tdCls}>{r.dim_days}</td>
							<td class={tdCls}>{r.lactation_number}</td>
							<td class={tdCls}>{r.current_feed_avg.toFixed(1)} кг</td>
							<td class={`${tdCls} font-semibold`}>{r.recommended_feed.toFixed(1)} кг</td>
							<td class={`${tdCls} ${r.difference_kg > 0 ? 'text-orange-600 dark:text-orange-400' : r.difference_kg < 0 ? 'text-green-600 dark:text-green-400' : ''}`}>{r.difference_kg > 0 ? '+' : ''}{r.difference_kg.toFixed(1)}</td>
							<td class={tdCls}>
								<span class="px-2 py-0.5 rounded-full text-xs font-medium {r.suggestion === 'increase' ? 'bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-400' : r.suggestion === 'decrease' ? 'bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-400' : 'bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-400'}">{r.suggestion === 'increase' ? 'Увеличить' : r.suggestion === 'decrease' ? 'Уменьшить' : 'Держать'}</span>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

	<!-- KETOSIS WARNING -->
	{:else if activeTab === 'ketosis' && ketosisData}
		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Корова</th>
						<th class={thCls}>Риск</th>
						<th class={thCls}>Тип</th>
						<th class={thCls}>Серьёзность</th>
						<th class={thCls}>FPR</th>
						<th class={thCls}>Тренд FPR</th>
						<th class={thCls}>Факторы</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each ketosisData.predictions.toSorted((a, b) => b.risk_probability - a.risk_probability) as k (k.animal_id)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{k.animal_name ?? `#${k.animal_id}`}</td>
							<td class={`${tdCls} font-semibold ${k.risk_probability >= 0.7 ? 'text-red-600 dark:text-red-400' : k.risk_probability >= 0.4 ? 'text-orange-600 dark:text-orange-400' : ''}`}>{(k.risk_probability * 100).toFixed(0)}%</td>
							<td class={tdCls}>
								<span class="px-2 py-0.5 rounded-full text-xs font-medium {k.risk_type === 'ketosis_risk' ? 'bg-purple-100 dark:bg-purple-900/40 text-purple-700 dark:text-purple-400' : k.risk_type === 'acidosis_risk' ? 'bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-400' : 'bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-400'}">{k.risk_type}</span>
							</td>
							<td class={tdCls}>
								<span class="px-2 py-0.5 rounded-full text-xs font-medium {riskBadge(k.severity)}">{k.severity}</span>
							</td>
							<td class={tdCls}>{k.fpr_current.toFixed(3)}</td>
							<td class={`${tdCls} ${k.fpr_trend > 0.05 ? 'text-red-600 dark:text-red-400' : k.fpr_trend < -0.05 ? 'text-green-600 dark:text-green-400' : ''}`}>{k.fpr_trend >= 0 ? '+' : ''}{(k.fpr_trend * 100).toFixed(1)}%</td>
							<td class={tdCls}>
								{#each k.contributing_factors as f, i}
									{#if i > 0}<span class="text-slate-300 dark:text-slate-600 mx-1">·</span>{/if}
									<span class="text-xs">{f}</span>
								{/each}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

	<!-- FEED EFFICIENCY -->
	{:else if activeTab === 'feedEff' && feedEffData}
		{#if feedEffData.herd_avg_efficiency !== null}
			<div class="mb-4 grid grid-cols-1 sm:grid-cols-3 gap-3">
				<div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4">
					<div class="text-xs text-slate-500 dark:text-slate-400 mb-1">Средняя конверсия стада</div>
					<div class="text-lg font-bold">{feedEffData.herd_avg_efficiency?.toFixed(3) ?? '—'}</div>
				</div>
			</div>
		{/if}
		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>#</th>
						<th class={thCls}>Корова</th>
						<th class={thCls}>Надой, л/д</th>
						<th class={thCls}>Корм, кг/д</th>
						<th class={thCls}>Конверсия</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each feedEffData.cows as c (c.animal_id)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{c.efficiency_rank ?? '—'}</td>
							<td class={tdCls}>{c.animal_name ?? `#${c.animal_id}`}</td>
							<td class={tdCls}>{c.avg_daily_milk?.toFixed(1) ?? '—'}</td>
							<td class={tdCls}>{c.avg_daily_feed?.toFixed(1) ?? '—'}</td>
							<td class={`${tdCls} font-semibold ${scoreColor((c.feed_efficiency ?? 0) * 2, 100)}`}>{c.feed_efficiency?.toFixed(3) ?? '—'}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

	<!-- DRY OFF OPTIMIZER -->
	{:else if activeTab === 'dryOff' && dryOffData}
		{#if dryOffData.cows.length === 0}
			<div class="text-sm text-slate-500 dark:text-slate-400 py-8 text-center">Нет стельных коров с подтверждённой стельностью в ближайшие 120 дней</div>
		{:else}
			<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
				<table class={tblCls}>
					<thead class="bg-slate-50 dark:bg-slate-900">
						<tr>
							<th class={thCls}>Корова</th>
							<th class={thCls}>Ожид. отёл</th>
							<th class={thCls}>Надой, л/д</th>
							<th class={thCls}>Реком. запуск</th>
							<th class={thCls}>Дней до запуска</th>
							<th class={thCls}>SCC</th>
							<th class={thCls}>Готовность</th>
						</tr>
					</thead>
					<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
						{#each dryOffData.cows.toSorted((a, b) => (a.days_until_dry_off ?? 999) - (b.days_until_dry_off ?? 999)) as c (c.animal_id)}
							<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
								<td class={tdCls}>{c.animal_name ?? `#${c.animal_id}`}</td>
								<td class={tdCls}>{c.expected_calving_date ?? '—'}</td>
								<td class={tdCls}>{c.current_daily_milk?.toFixed(1) ?? '—'}</td>
								<td class={tdCls}>{c.recommended_dry_off_date ?? '—'}</td>
								<td class={`${tdCls} font-semibold ${(c.days_until_dry_off ?? 999) <= 0 ? 'text-red-600 dark:text-red-400' : (c.days_until_dry_off ?? 999) <= 7 ? 'text-orange-600 dark:text-orange-400' : ''}`}>{c.days_until_dry_off !== null ? `${c.days_until_dry_off} д` : '—'}</td>
								<td class={tdCls}>
									<span class="px-2 py-0.5 rounded-full text-xs font-medium {riskBadge(c.scc_status === 'elevated' ? 'high' : c.scc_status === 'moderate' ? 'medium' : 'low')}">{c.scc_status}</span>
								</td>
								<td class={tdCls}>
									<span class="px-2 py-0.5 rounded-full text-xs font-medium {c.readiness === 'overdue' ? 'bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-400' : c.readiness === 'now' ? 'bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-400' : c.readiness === 'soon' ? 'bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-400' : 'bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-400'}">{c.readiness}</span>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}

	<!-- LIFETIME VALUE -->
	{:else if activeTab === 'lifetime' && lifetimeData}
		<div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900">
					<tr>
						<th class={thCls}>Корова</th>
						<th class={thCls}>Возраст</th>
						<th class={thCls}>Лакт.</th>
						<th class={thCls}>Всего молока, л</th>
						<th class={thCls}>Осталось лакт.</th>
						<th class={thCls}>Прогноз выручка</th>
						<th class={thCls}>Прогноз расход</th>
						<th class={thCls}>Чистая ценность</th>
						<th class={thCls}>Рекомендация</th>
					</tr>
				</thead>
				<tbody class="bg-white dark:bg-slate-800 divide-y divide-slate-200 dark:divide-slate-700">
					{#each lifetimeData.cows.toSorted((a, b) => (b.projected_net_value ?? 0) - (a.projected_net_value ?? 0)) as c (c.animal_id)}
						<tr class="hover:bg-slate-50 dark:hover:bg-slate-700/50">
							<td class={tdCls}>{c.animal_name ?? `#${c.animal_id}`}</td>
							<td class={tdCls}>{c.age_years?.toFixed(1) ?? '—'}</td>
							<td class={tdCls}>{c.lactation_count}</td>
							<td class={tdCls}>{c.total_milk_produced?.toFixed(0) ?? '—'}</td>
							<td class={tdCls}>{c.estimated_remaining_lactations}</td>
							<td class={tdCls}>{c.projected_milk_value !== null ? `${(c.projected_milk_value / 1000).toFixed(0)}k` : '—'}</td>
							<td class={tdCls}>{c.projected_feed_cost !== null ? `${(c.projected_feed_cost / 1000).toFixed(0)}k` : '—'}</td>
							<td class={`${tdCls} font-semibold ${(c.projected_net_value ?? 0) >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}`}>{c.projected_net_value !== null ? `${(c.projected_net_value / 1000).toFixed(0)}k` : '—'}</td>
							<td class={tdCls}>
								<span class="px-2 py-0.5 rounded-full text-xs font-medium {c.recommendation === 'culling_candidate' ? 'bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-400' : c.recommendation === 'review' ? 'bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-400' : c.recommendation === 'last_lactation' ? 'bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-400' : 'bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-400'}">{c.recommendation}</span>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
{/if}
