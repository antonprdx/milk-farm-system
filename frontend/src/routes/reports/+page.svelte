<script lang="ts">
	import {
		getMilkSummary,
		getReproductionSummary,
		getFeedSummary,
		getHerdOverview,
		getRestFeed,
		getRobotPerformance,
		getFailedMilkings,
		getUdderHealthWorklist,
		getUdderHealthAnalyze,
		getMilkDayProductionTime,
		getVisitBehavior,
		getCalendar,
		getHealthActivityRumination,
		getCowRobotEfficiency,
		getLactationAnalysis,
		getFeedPerTypeDay,
		getFeedPerCowDay,
		getHealthTask,
		getPregnancyRate,
		getTransitionReport,
		getExportUrl,
		getReportExportUrl,
		type MilkSummary,
		type ReproductionSummary,
		type FeedSummary,
		type HerdOverviewResponse,
		type RestFeedResponse,
		type RobotPerformanceRow,
		type FailedMilkingRow,
		type UdderHealthRow,
		type MilkDayProductionTimeRow,
		type VisitBehaviorRow,
		type CalendarResponse,
		type HealthActivityRow,
		type CowRobotEfficiencyRow,
		type LactationAnalysisResponse,
		type FeedPerTypeResponse,
		type FeedPerCowDayRow,
		type HealthTaskResponse,
		type PregnancyRateResponse,
		type TransitionResponse,
	} from '$lib/api/reports';
	import { onMount } from 'svelte';
	import FilterBar from '$lib/components/ui/FilterBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import { fmtNum } from '$lib/utils/format';

	type TabId =
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

	interface TabDef {
		id: TabId;
		label: string;
		group: string;
	}

	let tabs: TabDef[] = [
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

	let activeTab: TabId = $state('summary');
	let loading = $state(false);
	let error = $state('');
	let fromDate = $state('');
	let tillDate = $state('');

	const tabExportType: Record<string, string> = {
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
	let milk: MilkSummary | null = $state(null);
	let repro: ReproductionSummary | null = $state(null);
	let feed: FeedSummary | null = $state(null);
	let herdData: HerdOverviewResponse | null = $state(null);
	let restFeedData: RestFeedResponse | null = $state(null);
	let robotData: RobotPerformanceRow[] = $state([]);
	let failedData: FailedMilkingRow[] = $state([]);
	let udderWork: UdderHealthRow[] = $state([]);
	let udderAnalyze: UdderHealthRow[] = $state([]);
	let milkTime: MilkDayProductionTimeRow[] = $state([]);
	let visitData: VisitBehaviorRow[] = $state([]);
	let calendarData: CalendarResponse | null = $state(null);
	let healthAct: HealthActivityRow[] = $state([]);
	let efficiencyData: CowRobotEfficiencyRow[] = $state([]);
	let lactationData: LactationAnalysisResponse[] = $state([]);
	let feedTypeData: FeedPerTypeResponse | null = $state(null);
	let feedCowData: FeedPerCowDayRow[] = $state([]);
	let healthTaskData: HealthTaskResponse | null = $state(null);
	let pregnancyData: PregnancyRateResponse | null = $state(null);
	let transitionData: TransitionResponse | null = $state(null);

	function groupedTabs(): { group: string; items: TabDef[] }[] {
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

	async function load() {
		try {
			loading = true;
			error = '';
			const from = fromDate || undefined;
			const till = tillDate || undefined;

			if (activeTab === 'summary') {
				[milk, repro, feed] = await Promise.all([
					getMilkSummary(from, till),
					getReproductionSummary(from, till),
					getFeedSummary(from, till),
				]);
			} else if (activeTab === 'herd') {
				herdData = await getHerdOverview(from, till);
			} else if (activeTab === 'rest-feed') {
				restFeedData = await getRestFeed(from, till);
			} else if (activeTab === 'robot') {
				robotData = await getRobotPerformance(from, till);
			} else if (activeTab === 'failed') {
				failedData = await getFailedMilkings(from, till);
			} else if (activeTab === 'udder-work') {
				udderWork = (await getUdderHealthWorklist()).rows;
			} else if (activeTab === 'udder-analyze') {
				udderAnalyze = (await getUdderHealthAnalyze()).rows;
			} else if (activeTab === 'milk-time') {
				milkTime = await getMilkDayProductionTime(from, till);
			} else if (activeTab === 'visit') {
				visitData = await getVisitBehavior(from, till);
			} else if (activeTab === 'calendar') {
				calendarData = await getCalendar();
			} else if (activeTab === 'health-act') {
				healthAct = await getHealthActivityRumination();
			} else if (activeTab === 'efficiency') {
				efficiencyData = await getCowRobotEfficiency();
			} else if (activeTab === 'lactation') {
				lactationData = await getLactationAnalysis();
			} else if (activeTab === 'feed-type') {
				feedTypeData = await getFeedPerTypeDay(from, till);
			} else if (activeTab === 'feed-cow') {
				feedCowData = await getFeedPerCowDay(from, till);
			} else if (activeTab === 'health-task') {
				healthTaskData = await getHealthTask();
			} else if (activeTab === 'pregnancy') {
				pregnancyData = await getPregnancyRate();
			} else if (activeTab === 'transition') {
				transitionData = await getTransitionReport();
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			loading = false;
		}
	}

	function switchTab(tab: TabId) {
		activeTab = tab;
		load();
	}

	let thCls =
		'px-3 py-2 text-left text-xs font-medium text-slate-500 dark:text-slate-400 uppercase tracking-wider';
	let tdCls = 'px-3 py-2 text-sm text-slate-700 dark:text-slate-300 whitespace-nowrap';
	let tblCls = 'min-w-full divide-y divide-slate-200 dark:divide-slate-700';
	let badgeRed =
		'px-1.5 py-0.5 text-xs rounded bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400';
	let badgeYellow =
		'px-1.5 py-0.5 text-xs rounded bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400';
	let badgeGreen =
		'px-1.5 py-0.5 text-xs rounded bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400';

	function statusBadge(status: string) {
		if (status === 'critical') return badgeRed;
		if (status === 'warning') return badgeYellow;
		return badgeGreen;
	}

	onMount(load);
</script>

<svelte:head>
	<title>Отчёты — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-4">Отчёты</h1>

<div class="mb-4 flex flex-wrap gap-1">
	{#each groupedTabs() as grp (grp.group)}
		<div class="flex items-center gap-1">
			<span class="text-xs font-semibold text-slate-400 dark:text-slate-500 mr-1">{grp.group}:</span
			>
			{#each grp.items as tab (tab.id)}
				<button
					class="px-2 py-1 text-xs rounded-md border transition-colors cursor-pointer {activeTab ===
					tab.id
						? 'bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 border-blue-200 dark:border-blue-800'
						: 'border-slate-200 dark:border-slate-600 text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-700'}"
					onclick={() => switchTab(tab.id)}>{tab.label}</button
				>
			{/each}
			<span class="mx-2 text-slate-300 dark:text-slate-600">|</span>
		</div>
	{/each}
</div>

{#if activeTab !== 'udder-work' && activeTab !== 'udder-analyze' && activeTab !== 'calendar' && activeTab !== 'health-act' && activeTab !== 'efficiency' && activeTab !== 'lactation' && activeTab !== 'health-task' && activeTab !== 'pregnancy' && activeTab !== 'transition'}
	<FilterBar bind:fromDate bind:tillDate showAnimal={false} onsearch={load} />
{/if}

{#if activeTab !== 'summary' && tabExportType[activeTab]}
	<div class="mb-3 flex gap-2">
		<a
			href={getReportExportUrl(
				tabExportType[activeTab],
				fromDate || undefined,
				tillDate || undefined,
				'csv',
			)}
			class="text-xs px-2 py-1 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
			>CSV</a
		>
		<a
			href={getReportExportUrl(
				tabExportType[activeTab],
				fromDate || undefined,
				tillDate || undefined,
				'pdf',
			)}
			class="text-xs px-2 py-1 bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors"
			>PDF</a
		>
	</div>
{/if}

<ErrorAlert message={error} />

{#if loading}
	<div class="space-y-3">
		{#each Array(3) as _, i (i)}
			<div class="h-10 bg-slate-100 dark:bg-slate-900 rounded animate-pulse"></div>
		{/each}
	</div>
{:else}
	<!-- SUMMARY -->
	{#if activeTab === 'summary'}
		<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 space-y-4"
			>
				<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300">Молоко</h2>
				<div class="float-right flex gap-1">
					<a
						href={getExportUrl('milk', fromDate || undefined, tillDate || undefined, 'pdf')}
						class="text-xs px-2 py-1 bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors"
						>PDF</a
					>
					<a
						href={getExportUrl('milk', fromDate || undefined, tillDate || undefined)}
						class="text-xs px-2 py-1 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
						>CSV</a
					>
				</div>
				{#if milk}
					<div class="space-y-2">
						<div class="flex justify-between text-sm">
							<span class="text-slate-500 dark:text-slate-400">Общий надой</span><span
								class="font-medium">{fmtNum(milk.total_milk)} л</span
							>
						</div>
						<div class="flex justify-between text-sm">
							<span class="text-slate-500 dark:text-slate-400">Записей</span><span
								class="font-medium">{milk.count_days}</span
							>
						</div>
						<div class="flex justify-between text-sm">
							<span class="text-slate-500 dark:text-slate-400">Среднее на животное</span><span
								class="font-medium">{fmtNum(milk.avg_per_animal)} л</span
							>
						</div>
					</div>
				{/if}
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 space-y-4"
			>
				<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300">Воспроизводство</h2>
				<div class="float-right flex gap-1">
					<a
						href={getExportUrl('reproduction', fromDate || undefined, tillDate || undefined, 'pdf')}
						class="text-xs px-2 py-1 bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors"
						>PDF</a
					>
					<a
						href={getExportUrl('reproduction', fromDate || undefined, tillDate || undefined)}
						class="text-xs px-2 py-1 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
						>CSV</a
					>
				</div>
				{#if repro}
					<div class="space-y-2">
						<div class="flex justify-between text-sm">
							<span class="text-slate-500 dark:text-slate-400">Отёлы</span><span class="font-medium"
								>{repro.total_calvings}</span
							>
						</div>
						<div class="flex justify-between text-sm">
							<span class="text-slate-500 dark:text-slate-400">Осеменения</span><span
								class="font-medium">{repro.total_inseminations}</span
							>
						</div>
						<div class="flex justify-between text-sm">
							<span class="text-slate-500 dark:text-slate-400">Стельности</span><span
								class="font-medium">{repro.total_pregnancies}</span
							>
						</div>
						<div class="flex justify-between text-sm">
							<span class="text-slate-500 dark:text-slate-400">Охота</span><span class="font-medium"
								>{repro.total_heats}</span
							>
						</div>
						<div class="flex justify-between text-sm">
							<span class="text-slate-500 dark:text-slate-400">Запуски</span><span
								class="font-medium">{repro.total_dry_offs}</span
							>
						</div>
					</div>
				{/if}
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 space-y-4"
			>
				<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300">Кормление</h2>
				<div class="float-right flex gap-1">
					<a
						href={getExportUrl('feed', fromDate || undefined, tillDate || undefined, 'pdf')}
						class="text-xs px-2 py-1 bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors"
						>PDF</a
					>
					<a
						href={getExportUrl('feed', fromDate || undefined, tillDate || undefined)}
						class="text-xs px-2 py-1 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
						>CSV</a
					>
				</div>
				{#if feed}
					<div class="space-y-2">
						<div class="flex justify-between text-sm">
							<span class="text-slate-500 dark:text-slate-400">Общий корм</span><span
								class="font-medium">{fmtNum(feed.total_feed_kg)} кг</span
							>
						</div>
						<div class="flex justify-between text-sm">
							<span class="text-slate-500 dark:text-slate-400">Визитов</span><span
								class="font-medium">{feed.total_visits}</span
							>
						</div>
					</div>
				{/if}
			</div>
		</div>

		<!-- HERD OVERVIEW -->
	{:else if activeTab === 'herd' && herdData}
		<div class="mb-4 grid grid-cols-2 md:grid-cols-5 gap-3">
			<div
				class="bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">Среднее поголовье</div>
				<div class="text-lg font-semibold">{fmtNum(herdData.avg_cow_count, 0)}</div>
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">Средний надой/день</div>
				<div class="text-lg font-semibold">{fmtNum(herdData.avg_milk)} л</div>
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">Среднее доек/день</div>
				<div class="text-lg font-semibold">{fmtNum(herdData.avg_milkings, 0)}</div>
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">Среднее отказов/день</div>
				<div class="text-lg font-semibold">{fmtNum(herdData.avg_failures, 0)}</div>
			</div>
			<div
				class="bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3"
			>
				<div class="text-xs text-slate-500 dark:text-slate-400">Средний SCC</div>
				<div class="text-lg font-semibold">{fmtNum(herdData.avg_scc, 0)}</div>
			</div>
		</div>
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr>
						<th class={thCls}>Дата</th><th class={thCls}>Коров</th><th class={thCls}>Надой (л)</th>
						<th class={thCls}>Средн./корова</th><th class={thCls}>Доек</th><th class={thCls}
							>Отказов</th
						>
						<th class={thCls}>Неудач</th><th class={thCls}>Сепарация</th><th class={thCls}>SCC</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each herdData.period as row (row.date)}
						<tr
							><td class={tdCls}>{row.date}</td><td class={tdCls}>{row.cow_count}</td>
							<td class={tdCls}>{fmtNum(row.total_milk)}</td><td class={tdCls}
								>{fmtNum(row.avg_day_production)}</td
							>
							<td class={tdCls}>{row.total_milkings ?? '—'}</td><td class={tdCls}
								>{row.total_refusals ?? '—'}</td
							>
							<td class={tdCls}>{row.total_failures ?? '—'}</td><td class={tdCls}
								>{row.milk_separated ?? '—'}</td
							>
							<td class={tdCls}>{fmtNum(row.avg_scc, 0)}</td></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- REST FEED -->
	{:else if activeTab === 'rest-feed' && restFeedData}
		{#if restFeedData.total_rest_feed_pct != null}
			<div
				class="mb-4 bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3"
			>
				<span class="text-sm text-slate-500 dark:text-slate-400">Общий % остатка корма: </span>
				<span
					class="font-semibold {restFeedData.total_rest_feed_pct > 5
						? 'text-red-600 dark:text-red-400'
						: 'text-green-600 dark:text-green-400'}"
				>
					{fmtNum(restFeedData.total_rest_feed_pct)}%
				</span>
			</div>
		{/if}
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Животное</th><th class={thCls}>Дата</th><th class={thCls}>№ Корма</th
						>
						<th class={thCls}>План (кг)</th><th class={thCls}>Остаток</th><th class={thCls}>%</th
						></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each restFeedData.rows as row (row.animal_id + '-' + row.feed_date + '-' + row.feed_number)}
						<tr
							><td class={tdCls}>{row.animal_name ?? row.animal_id}</td><td class={tdCls}
								>{row.feed_date}</td
							>
							<td class={tdCls}>{row.feed_number}</td><td class={tdCls}
								>{fmtNum(row.total_planned)}</td
							>
							<td class={tdCls}>{row.rest_feed ?? '—'}</td>
							<td class={tdCls}
								><span
									class={row.rest_feed_pct && row.rest_feed_pct > 10
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{fmtNum(row.rest_feed_pct)}%</span
								></td
							></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- ROBOT PERFORMANCE -->
	{:else if activeTab === 'robot'}
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Робот</th><th class={thCls}>Дата</th><th class={thCls}
							>Ср. скорость</th
						>
						<th class={thCls}>Макс. скорость</th><th class={thCls}>Доек</th>
						<th class={thCls}>LF время</th><th class={thCls}>LR время</th><th class={thCls}
							>RF время</th
						><th class={thCls}>RR время</th>
						<th class={thCls}>LF DMT</th><th class={thCls}>LR DMT</th><th class={thCls}>RF DMT</th
						><th class={thCls}>RR DMT</th></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each robotData as row (row.device_address + '-' + row.date)}
						<tr
							><td class={tdCls}>{row.device_address ?? '—'}</td><td class={tdCls}>{row.date}</td>
							<td class={tdCls}>{fmtNum(row.avg_milk_speed)}</td><td class={tdCls}
								>{fmtNum(row.max_milk_speed)}</td
							>
							<td class={tdCls}>{row.milkings}</td>
							<td class={tdCls}>{fmtNum(row.avg_lf_milk_time, 0)}</td><td class={tdCls}
								>{fmtNum(row.avg_lr_milk_time, 0)}</td
							>
							<td class={tdCls}>{fmtNum(row.avg_rf_milk_time, 0)}</td><td class={tdCls}
								>{fmtNum(row.avg_rr_milk_time, 0)}</td
							>
							<td class={tdCls}
								><span
									class={row.avg_lf_dead_milk_time && row.avg_lf_dead_milk_time > 60
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{fmtNum(row.avg_lf_dead_milk_time, 0)}</span
								></td
							>
							<td class={tdCls}
								><span
									class={row.avg_lr_dead_milk_time && row.avg_lr_dead_milk_time > 60
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{fmtNum(row.avg_lr_dead_milk_time, 0)}</span
								></td
							>
							<td class={tdCls}
								><span
									class={row.avg_rf_dead_milk_time && row.avg_rf_dead_milk_time > 60
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{fmtNum(row.avg_rf_dead_milk_time, 0)}</span
								></td
							>
							<td class={tdCls}
								><span
									class={row.avg_rr_dead_milk_time && row.avg_rr_dead_milk_time > 60
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{fmtNum(row.avg_rr_dead_milk_time, 0)}</span
								></td
							></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- FAILED MILKINGS -->
	{:else if activeTab === 'failed'}
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Животное</th><th class={thCls}>Время</th><th class={thCls}>Робот</th>
						<th class={thCls}>Надой</th><th class={thCls}>LF конд.</th><th class={thCls}
							>LR конд.</th
						>
						<th class={thCls}>RF конд.</th><th class={thCls}>RR конд.</th></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each failedData as row (row.animal_id + '-' + row.visit_datetime)}
						<tr
							><td class={tdCls}>{row.animal_name ?? row.animal_id}</td><td class={tdCls}
								>{new Date(row.visit_datetime).toLocaleString('ru-RU')}</td
							>
							<td class={tdCls}>{row.device_address ?? '—'}</td><td class={tdCls}
								>{fmtNum(row.milk_yield)}</td
							>
							<td class={tdCls}
								><span
									class={row.lf_conductivity && row.lf_conductivity > 83
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{row.lf_conductivity ?? '—'}</span
								></td
							>
							<td class={tdCls}
								><span
									class={row.lr_conductivity && row.lr_conductivity > 83
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{row.lr_conductivity ?? '—'}</span
								></td
							>
							<td class={tdCls}
								><span
									class={row.rf_conductivity && row.rf_conductivity > 83
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{row.rf_conductivity ?? '—'}</span
								></td
							>
							<td class={tdCls}
								><span
									class={row.rr_conductivity && row.rr_conductivity > 83
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{row.rr_conductivity ?? '—'}</span
								></td
							></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- UDDER HEALTH (worklist & analyze) -->
	{:else if activeTab === 'udder-work' || activeTab === 'udder-analyze'}
		{@const udderRows = activeTab === 'udder-work' ? udderWork : udderAnalyze}
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Животное</th><th class={thCls}>Время</th>
						<th class={thCls}>LF конд.</th><th class={thCls}>LR конд.</th><th class={thCls}
							>RF конд.</th
						><th class={thCls}>RR конд.</th>
						<th class={thCls}>Цвет</th><th class={thCls}>SCC</th><th class={thCls}>Отклон. удоя</th>
						<th class={thCls}>Attention</th></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each udderRows as row (row.animal_id + '-' + row.visit_datetime)}
						<tr
							><td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
							<td class={tdCls}>{new Date(row.visit_datetime).toLocaleString('ru-RU')}</td>
							<td class={tdCls}
								><span
									class={row.lf_conductivity && row.lf_conductivity > 80
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{row.lf_conductivity ?? '—'}</span
								></td
							>
							<td class={tdCls}
								><span
									class={row.lr_conductivity && row.lr_conductivity > 80
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{row.lr_conductivity ?? '—'}</span
								></td
							>
							<td class={tdCls}
								><span
									class={row.rf_conductivity && row.rf_conductivity > 80
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{row.rf_conductivity ?? '—'}</span
								></td
							>
							<td class={tdCls}
								><span
									class={row.rr_conductivity && row.rr_conductivity > 80
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{row.rr_conductivity ?? '—'}</span
								></td
							>
							<td class={tdCls}
								>{[row.lf_colour, row.lr_colour, row.rf_colour, row.rr_colour]
									.filter(Boolean)
									.join(', ') || '—'}</td
							>
							<td class={tdCls}>{row.latest_scc ?? '—'}</td>
							<td class={tdCls}
								><span
									class={row.deviation_day_prod && row.deviation_day_prod < -3
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{fmtNum(row.deviation_day_prod)}</span
								></td
							>
							<td class={tdCls}>{row.attention_quarters.join('; ') || '—'}</td></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- MILK DAY PRODUCTION IN TIME -->
	{:else if activeTab === 'milk-time'}
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Дата</th><th class={thCls}>Коров</th><th class={thCls}>Надой (л)</th>
						<th class={thCls}>Средн./корова</th><th class={thCls}>Доек</th><th class={thCls}
							>Отказов</th
						>
						<th class={thCls}>Неудач</th><th class={thCls}>Вес</th><th class={thCls}>Корм (кг)</th
						><th class={thCls}>Ост. корм</th></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each milkTime as row (row.date)}
						<tr
							><td class={tdCls}>{row.date}</td><td class={tdCls}>{row.cow_count}</td>
							<td class={tdCls}>{fmtNum(row.total_milk)}</td><td class={tdCls}
								>{fmtNum(row.avg_milk_per_cow)}</td
							>
							<td class={tdCls}>{row.milkings ?? '—'}</td><td class={tdCls}
								>{row.refusals ?? '—'}</td
							>
							<td class={tdCls}>{row.failures ?? '—'}</td><td class={tdCls}
								>{fmtNum(row.avg_weight)}</td
							>
							<td class={tdCls}>{fmtNum(row.total_feed)}</td><td class={tdCls}
								>{row.total_rest_feed ?? '—'}</td
							></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- VISIT BEHAVIOR -->
	{:else if activeTab === 'visit'}
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Животное</th><th class={thCls}>Доек</th><th class={thCls}>Отказов</th
						>
						<th class={thCls}>Средн./доение (л)</th><th class={thCls}>Средн. время (с)</th><th
							class={thCls}>Посл. визит</th
						></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each visitData as row (row.animal_id)}
						<tr
							><td class={tdCls}>{row.animal_name ?? row.animal_id}</td><td class={tdCls}
								>{row.total_milkings}</td
							>
							<td class={tdCls}>{row.total_refusals}</td>
							<td class={tdCls}
								><span
									class={row.avg_milk_per_milking && row.avg_milk_per_milking < 8
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{fmtNum(row.avg_milk_per_milking)}</span
								></td
							>
							<td class={tdCls}>{fmtNum(row.avg_duration_seconds, 0)}</td>
							<td class={tdCls}
								>{row.last_visit ? new Date(row.last_visit).toLocaleString('ru-RU') : '—'}</td
							></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- CALENDAR -->
	{:else if activeTab === 'calendar' && calendarData}
		<div class="space-y-6">
			<div>
				<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">
					Ожидаемые отёлы (R31)
				</h2>
				<div
					class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
				>
					<table class={tblCls}>
						<thead class="bg-slate-50 dark:bg-slate-900/50">
							<tr
								><th class={thCls}>Животное</th><th class={thCls}>Лактация</th><th class={thCls}
									>Последняя инсем.</th
								>
								<th class={thCls}>Ожид. отёл</th><th class={thCls}>Дней до отёла</th><th
									class={thCls}>Бык</th
								><th class={thCls}>Дней стельности</th></tr
							>
						</thead>
						<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
							{#each calendarData.expected_calvings as row (row.animal_id)}
								<tr
									><td class={tdCls}>{row.animal_name ?? row.animal_id}</td><td class={tdCls}
										>{row.lac_number ?? '—'}</td
									>
									<td class={tdCls}>{row.last_insemination_date ?? '—'}</td><td class={tdCls}
										>{row.expected_calving_date ?? '—'}</td
									>
									<td class={tdCls}
										><span
											class={row.days_until_calving && row.days_until_calving < 14
												? badgeRed
												: row.days_until_calving && row.days_until_calving < 30
													? badgeYellow
													: ''}>{row.days_until_calving ?? '—'}</span
										></td
									>
									<td class={tdCls}>{row.sire_code ?? '—'}</td><td class={tdCls}
										>{row.days_pregnant ?? '—'}</td
									></tr
								>
							{/each}
						</tbody>
					</table>
				</div>
			</div>
			<div>
				<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">
					Ожидаемые запуски (R32)
				</h2>
				<div
					class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
				>
					<table class={tblCls}>
						<thead class="bg-slate-50 dark:bg-slate-900/50">
							<tr
								><th class={thCls}>Животное</th><th class={thCls}>Ожид. отёл</th><th class={thCls}
									>Реком. запуск</th
								>
								<th class={thCls}>Дней до запуска</th></tr
							>
						</thead>
						<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
							{#each calendarData.expected_dry_offs as row (row.animal_id)}
								<tr
									><td class={tdCls}>{row.animal_name ?? row.animal_id}</td><td class={tdCls}
										>{row.expected_calving_date ?? '—'}</td
									>
									<td class={tdCls}>{row.recommended_dry_off_date ?? '—'}</td>
									<td class={tdCls}
										><span
											class={row.days_until_dry_off && row.days_until_dry_off < 7
												? badgeRed
												: row.days_until_dry_off && row.days_until_dry_off < 14
													? badgeYellow
													: ''}>{row.days_until_dry_off ?? '—'}</span
										></td
									></tr
								>
							{/each}
						</tbody>
					</table>
				</div>
			</div>
			<div>
				<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">
					Ожидаемая охота (R33)
				</h2>
				<div
					class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
				>
					<table class={tblCls}>
						<thead class="bg-slate-50 dark:bg-slate-900/50">
							<tr
								><th class={thCls}>Животное</th><th class={thCls}>Посл. охота</th><th class={thCls}
									>Ожид. охота</th
								>
								<th class={thCls}>Дней до</th><th class={thCls}>Дней в лакт.</th><th class={thCls}
									>Осеменена</th
								></tr
							>
						</thead>
						<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
							{#each calendarData.expected_heats as row (row.animal_id)}
								<tr
									><td class={tdCls}>{row.animal_name ?? row.animal_id}</td><td class={tdCls}
										>{row.last_heat_date ?? '—'}</td
									>
									<td class={tdCls}>{row.expected_heat_date ?? '—'}</td>
									<td class={tdCls}
										><span
											class={row.overdue
												? badgeRed
												: row.days_until_heat && row.days_until_heat < 3
													? badgeYellow
													: ''}>{row.days_until_heat ?? '—'}</span
										></td
									>
									<td class={tdCls}>{row.days_in_lactation ?? '—'}</td>
									<td class={tdCls}>{row.inseminated ? 'Да' : 'Нет'}</td></tr
								>
							{/each}
						</tbody>
					</table>
				</div>
			</div>
			<div>
				<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">
					Проверка стельности (R34)
				</h2>
				<div
					class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
				>
					<table class={tblCls}>
						<thead class="bg-slate-50 dark:bg-slate-900/50">
							<tr
								><th class={thCls}>Животное</th><th class={thCls}>Дата инсем.</th><th class={thCls}
									>Бык</th
								>
								<th class={thCls}>Дней после инсем.</th></tr
							>
						</thead>
						<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
							{#each calendarData.pregnancy_checks as row (row.animal_id)}
								<tr
									><td class={tdCls}>{row.animal_name ?? row.animal_id}</td><td class={tdCls}
										>{row.insemination_date ?? '—'}</td
									>
									<td class={tdCls}>{row.sire_code ?? '—'}</td>
									<td class={tdCls}
										><span class={badgeYellow}>{row.days_since_insemination ?? '—'}</span></td
									></tr
								>
							{/each}
						</tbody>
					</table>
				</div>
			</div>
		</div>

		<!-- HEALTH ACTIVITY/RUMINATION -->
	{:else if activeTab === 'health-act'}
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Животное</th><th class={thCls}>Health Index</th><th class={thCls}
							>Откл. активн.</th
						>
						<th class={thCls}>Жвачка (мин)</th><th class={thCls}>Макс.изм. 24ч</th><th class={thCls}
							>Разн. 3 дня</th
						>
						<th class={thCls}>Надой</th><th class={thCls}>Ср. 7д</th><th class={thCls}>Откл. %</th
						></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each healthAct as row (row.animal_id)}
						<tr
							><td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
							<td class={tdCls}
								><span
									class={row.health_index && row.health_index < 75
										? badgeRed
										: row.health_index && row.health_index < 80
											? badgeYellow
											: ''}>{fmtNum(row.health_index, 0)}</span
								></td
							>
							<td class={tdCls}>{fmtNum(row.activity_deviation, 0)}</td>
							<td class={tdCls}>{row.rumination_minutes ?? '—'}</td>
							<td class={tdCls}>{row.max_rumination_change_24h ?? '—'}</td>
							<td class={tdCls}
								><span
									class={row.rumination_3day_diff && row.rumination_3day_diff < -60
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{row.rumination_3day_diff ?? '—'}</span
								></td
							>
							<td class={tdCls}>{fmtNum(row.latest_milk)}</td><td class={tdCls}
								>{fmtNum(row.avg_milk_7d)}</td
							>
							<td class={tdCls}>{fmtNum(row.milk_deviation_pct)}%</td></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- COW ROBOT EFFICIENCY -->
	{:else if activeTab === 'efficiency'}
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Животное</th><th class={thCls}>Молоко/мин/нед</th><th class={thCls}
							>Ср. скорость</th
						>
						<th class={thCls}>Время обработки</th><th class={thCls}>Время доения</th><th
							class={thCls}>Доек/7д</th
						>
						<th class={thCls}>Надой/7д</th><th class={thCls}>Средн./доение</th></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each efficiencyData as row (row.animal_id)}
						<tr
							><td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
							<td class={tdCls}
								><span
									class={row.milk_per_box_time_week && row.milk_per_box_time_week < 0.5
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{fmtNum(row.milk_per_box_time_week, 3)}</span
								></td
							>
							<td class={tdCls}>{fmtNum(row.avg_milk_speed)}</td>
							<td class={tdCls}>{fmtNum(row.avg_treatment_time)} мин</td>
							<td class={tdCls}>{fmtNum(row.avg_milking_time)} мин</td>
							<td class={tdCls}>{row.milkings_7d}</td>
							<td class={tdCls}>{fmtNum(row.total_milk_7d)}</td>
							<td class={tdCls}>{fmtNum(row.avg_milk_per_milking)}</td></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- LACTATION ANALYSIS -->
	{:else if activeTab === 'lactation'}
		{#each lactationData as lac (lac.lac_number)}
			<div class="mb-6">
				<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">
					Лактация {lac.lac_number}
				</h2>
				<div
					class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
				>
					<table class={tblCls}>
						<thead class="bg-slate-50 dark:bg-slate-900/50">
							<tr
								><th class={thCls}>DIM</th><th class={thCls}>Средн. надой</th><th class={thCls}
									>Визитов</th
								>
								<th class={thCls}>Корм</th><th class={thCls}>Вес</th><th class={thCls}>Жир%</th>
								<th class={thCls}>Белок%</th><th class={thCls}>Коров</th></tr
							>
						</thead>
						<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
							{#each lac.points as pt (pt.dim)}
								<tr
									><td class={tdCls}>{pt.dim}</td><td class={tdCls}>{fmtNum(pt.avg_milk)}</td>
									<td class={tdCls}>{fmtNum(pt.avg_visits, 1)}</td><td class={tdCls}
										>{fmtNum(pt.avg_feed)}</td
									>
									<td class={tdCls}>{fmtNum(pt.avg_weight)}</td><td class={tdCls}
										>{fmtNum(pt.avg_fat)}</td
									>
									<td class={tdCls}>{fmtNum(pt.avg_protein)}</td><td class={tdCls}
										>{pt.cow_count}</td
									></tr
								>
							{/each}
						</tbody>
					</table>
				</div>
			</div>
		{/each}

		<!-- FEED PER TYPE DAY -->
	{:else if activeTab === 'feed-type' && feedTypeData}
		<div
			class="mb-4 bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3"
		>
			<span class="text-sm text-slate-500 dark:text-slate-400">Общие затраты: </span>
			<span class="font-semibold">{fmtNum(feedTypeData.total_cost)} руб.</span>
			<span class="text-sm text-slate-500 dark:text-slate-400 ml-4">Ср. на 100 л: </span>
			<span class="font-semibold">{fmtNum(feedTypeData.avg_cost_per_100milk)} руб.</span>
		</div>
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Дата</th><th class={thCls}>Тип</th><th class={thCls}>Название</th>
						<th class={thCls}>Продукт (кг)</th><th class={thCls}>Сухое в-во (кг)</th>
						<th class={thCls}>Стоимость</th><th class={thCls}>На 100л молока</th></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each feedTypeData.rows as row (row.date + '-' + row.feed_type)}
						<tr
							><td class={tdCls}>{row.date}</td><td class={tdCls}>{row.feed_type}</td>
							<td class={tdCls}>{row.feed_type_name}</td><td class={tdCls}
								>{fmtNum(row.total_amount_product)}</td
							>
							<td class={tdCls}>{fmtNum(row.total_amount_dm)}</td><td class={tdCls}
								>{fmtNum(row.total_cost)}</td
							>
							<td class={tdCls}>{fmtNum(row.cost_per_100milk)}</td></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- FEED PER COW DAY -->
	{:else if activeTab === 'feed-cow'}
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Дата</th><th class={thCls}>Коров</th><th class={thCls}
							>Корм/корова</th
						>
						<th class={thCls}>Концентрат</th><th class={thCls}>Грубый</th><th class={thCls}
							>Стоим./корова</th
						>
						<th class={thCls}>Жвачка (мин)</th><th class={thCls}>Надой (л)</th><th class={thCls}
							>Дни лакт.</th
						>
						<th class={thCls}>Эфф. корма</th></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each feedCowData as row (row.date)}
						<tr
							><td class={tdCls}>{row.date}</td><td class={tdCls}>{row.animal_count}</td>
							<td class={tdCls}>{fmtNum(row.avg_total_per_cow)}</td><td class={tdCls}
								>{fmtNum(row.avg_concentrate_per_cow)}</td
							>
							<td class={tdCls}>{fmtNum(row.avg_roughage_per_cow)}</td><td class={tdCls}
								>{fmtNum(row.avg_cost_per_cow)}</td
							>
							<td class={tdCls}>{fmtNum(row.avg_rumination_minutes, 0)}</td><td class={tdCls}
								>{fmtNum(row.avg_day_production)}</td
							>
							<td class={tdCls}>{fmtNum(row.avg_lactation_days, 0)}</td><td class={tdCls}
								>{fmtNum(row.feed_efficiency)}</td
							></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- HEALTH TASK -->
	{:else if activeTab === 'health-task' && healthTaskData}
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Животное</th><th class={thCls}>Sick Chance</th><th class={thCls}
							>Статус</th
						>
						<th class={thCls}>Падение удоя</th><th class={thCls}>Конд.</th><th class={thCls}>SCC</th
						>
						<th class={thCls}>Откл. активн.</th><th class={thCls}>Откл. жвачки</th><th class={thCls}
							>Жир/Белок</th
						>
						<th class={thCls}>Ост. корм %</th><th class={thCls}>Темп.</th><th class={thCls}>Цвет</th
						>
						<th class={thCls}>Дни лакт.</th></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each healthTaskData.rows as row (row.animal_id)}
						<tr
							><td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
							<td class={tdCls}
								><span class={statusBadge(row.sick_chance_status)}
									>{fmtNum(row.sick_chance, 0)}</span
								></td
							>
							<td class={tdCls}
								><span class={statusBadge(row.sick_chance_status)}>{row.sick_chance_status}</span
								></td
							>
							<td class={tdCls}>{fmtNum(row.milk_drop_kg)}</td>
							<td class={tdCls}>{row.conductivity_highest ?? '—'}</td>
							<td class={tdCls}>{row.scc_indication ?? '—'}</td>
							<td class={tdCls}>{fmtNum(row.activity_deviation, 0)}</td>
							<td class={tdCls}>{row.rumination_deviation ?? '—'}</td>
							<td class={tdCls}>{fmtNum(row.fat_protein_ratio)}</td>
							<td class={tdCls}>{fmtNum(row.feed_rest_pct)}%</td>
							<td class={tdCls}>{fmtNum(row.temperature_highest)}</td>
							<td class={tdCls}>{row.colour_attentions.join(', ') || '—'}</td>
							<td class={tdCls}>{row.days_in_lactation ?? '—'}</td></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- PREGNANCY RATE -->
	{:else if activeTab === 'pregnancy' && pregnancyData}
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Конец периода</th><th class={thCls}>Пригодных</th><th class={thCls}
							>Осеменено</th
						>
						<th class={thCls}>Стельных</th><th class={thCls}>% осеменения</th><th class={thCls}
							>% зачатия</th
						>
						<th class={thCls}>Коэфф. стельности</th></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each pregnancyData.periods as row (row.end_date)}
						<tr
							><td class={tdCls}>{row.end_date}</td><td class={tdCls}>{row.eligible}</td>
							<td class={tdCls}>{row.inseminated}</td><td class={tdCls}>{row.pregnant}</td>
							<td class={tdCls}>{fmtNum(row.insemination_rate)}%</td>
							<td class={tdCls}>{fmtNum(row.conception_rate)}%</td>
							<td class={tdCls}
								><span
									class={row.pregnancy_rate && row.pregnancy_rate < 25
										? 'text-red-600 dark:text-red-400 font-medium'
										: 'font-medium'}>{fmtNum(row.pregnancy_rate)}%</span
								></td
							></tr
						>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- TRANSITION REPORT -->
	{:else if activeTab === 'transition' && transitionData}
		<div
			class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700"
		>
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr
						><th class={thCls}>Животное</th><th class={thCls}>День (относ.)</th><th class={thCls}
							>Надой 24ч</th
						>
						<th class={thCls}>Жвачка 3д разн.</th><th class={thCls}>Жвачка (мин)</th>
						<th class={thCls}>Корм (кг)</th><th class={thCls}>Ост. корм</th><th class={thCls}
							>SCC</th
						></tr
					>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each transitionData.rows as row (row.animal_id)}
						<tr
							><td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
							<td class={tdCls}
								><span
									class={row.days_relative === 0
										? badgeYellow
										: row.days_relative < 0
											? 'text-blue-600 dark:text-blue-400'
											: ''}>{row.days_relative > 0 ? '+' : ''}{row.days_relative}</span
								></td
							>
							<td class={tdCls}>{fmtNum(row.milk_24h)}</td>
							<td class={tdCls}
								><span
									class={row.rumination_3day_diff && row.rumination_3day_diff < -60
										? 'text-red-600 dark:text-red-400 font-medium'
										: ''}>{row.rumination_3day_diff ?? '—'}</span
								></td
							>
							<td class={tdCls}>{row.rumination_minutes ?? '—'}</td>
							<td class={tdCls}>{fmtNum(row.feed_total)}</td><td class={tdCls}
								>{row.feed_rest ?? '—'}</td
							>
							<td class={tdCls}>{row.latest_scc ?? '—'}</td></tr
						>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
{/if}
