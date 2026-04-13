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
	import {
		type TabId,
		groupedTabs,
		noFilterTabs,
		tabExportType,
	} from './_shared';

	import SummaryTab from './tabs/SummaryTab.svelte';
	import HerdTab from './tabs/HerdTab.svelte';
	import RestFeedTab from './tabs/RestFeedTab.svelte';
	import RobotTab from './tabs/RobotTab.svelte';
	import FailedTab from './tabs/FailedTab.svelte';
	import UdderHealthTab from './tabs/UdderHealthTab.svelte';
	import MilkTimeTab from './tabs/MilkTimeTab.svelte';
	import VisitTab from './tabs/VisitTab.svelte';
	import CalendarTab from './tabs/CalendarTab.svelte';
	import HealthActivityTab from './tabs/HealthActivityTab.svelte';
	import EfficiencyTab from './tabs/EfficiencyTab.svelte';
	import LactationTab from './tabs/LactationTab.svelte';
	import FeedTypeTab from './tabs/FeedTypeTab.svelte';
	import FeedCowTab from './tabs/FeedCowTab.svelte';
	import HealthTaskTab from './tabs/HealthTaskTab.svelte';
	import PregnancyTab from './tabs/PregnancyTab.svelte';
	import TransitionTab from './tabs/TransitionTab.svelte';

	let activeTab: TabId = $state('summary');
	let loading = $state(false);
	let error = $state('');
	let fromDate = $state('');
	let tillDate = $state('');

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

	async function load() {
		try {
			loading = true;
			error = '';
			const from = fromDate || undefined;
			const till = tillDate || undefined;

			if (activeTab === 'summary') {
				[milk, repro, feed] = await Promise.all([getMilkSummary(from, till), getReproductionSummary(from, till), getFeedSummary(from, till)]);
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

	onMount(load);
</script>

<svelte:head>
	<title>Отчёты — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-4">Отчёты</h1>

<div class="mb-4 flex flex-wrap gap-1">
	{#each groupedTabs() as grp (grp.group)}
		<div class="flex items-center gap-1">
			<span class="text-xs font-semibold text-slate-400 dark:text-slate-500 mr-1">{grp.group}:</span>
			{#each grp.items as tab (tab.id)}
				<button
					class="px-2 py-1 text-xs rounded-md border transition-colors cursor-pointer {activeTab === tab.id
						? 'bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 border-blue-200 dark:border-blue-800'
						: 'border-slate-200 dark:border-slate-600 text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-700'}"
					onclick={() => switchTab(tab.id)}
				>{tab.label}</button>
			{/each}
			<span class="mx-2 text-slate-300 dark:text-slate-600">|</span>
		</div>
	{/each}
</div>

{#if !noFilterTabs.includes(activeTab)}
	<FilterBar bind:fromDate bind:tillDate showAnimal={false} onsearch={load} />
{/if}

{#if activeTab !== 'summary' && tabExportType[activeTab]}
	<div class="mb-3 flex gap-2">
		<a
			href={getReportExportUrl(tabExportType[activeTab], fromDate || undefined, tillDate || undefined, 'csv')}
			class="text-xs px-2 py-1 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
		>CSV</a>
		<a
			href={getReportExportUrl(tabExportType[activeTab], fromDate || undefined, tillDate || undefined, 'pdf')}
			class="text-xs px-2 py-1 bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors"
		>PDF</a>
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
	{#if activeTab === 'summary'}
		<SummaryTab {milk} {repro} {feed} {fromDate} {tillDate} />
	{:else if activeTab === 'herd' && herdData}
		<HerdTab data={herdData} />
	{:else if activeTab === 'rest-feed' && restFeedData}
		<RestFeedTab data={restFeedData} />
	{:else if activeTab === 'robot'}
		<RobotTab rows={robotData} />
	{:else if activeTab === 'failed'}
		<FailedTab rows={failedData} />
	{:else if (activeTab === 'udder-work' || activeTab === 'udder-analyze')}
		<UdderHealthTab rows={activeTab === 'udder-work' ? udderWork : udderAnalyze} />
	{:else if activeTab === 'milk-time'}
		<MilkTimeTab rows={milkTime} />
	{:else if activeTab === 'visit'}
		<VisitTab rows={visitData} />
	{:else if activeTab === 'calendar' && calendarData}
		<CalendarTab data={calendarData} />
	{:else if activeTab === 'health-act'}
		<HealthActivityTab rows={healthAct} />
	{:else if activeTab === 'efficiency'}
		<EfficiencyTab rows={efficiencyData} />
	{:else if activeTab === 'lactation'}
		<LactationTab rows={lactationData} />
	{:else if activeTab === 'feed-type' && feedTypeData}
		<FeedTypeTab data={feedTypeData} />
	{:else if activeTab === 'feed-cow'}
		<FeedCowTab rows={feedCowData} />
	{:else if activeTab === 'health-task' && healthTaskData}
		<HealthTaskTab data={healthTaskData} />
	{:else if activeTab === 'pregnancy' && pregnancyData}
		<PregnancyTab data={pregnancyData} />
	{:else if activeTab === 'transition' && transitionData}
		<TransitionTab data={transitionData} />
	{/if}
{/if}
