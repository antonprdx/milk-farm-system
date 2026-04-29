<script lang="ts">
	import type { HerdOverviewResponse } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tdHidden, thHidden, tblCls } from '../_shared';
	import ReportChart from '$lib/components/ReportChart.svelte';

	let { data }: { data: HerdOverviewResponse } = $props();

	let showChart = $state(true);

	function btnClass(active: boolean): string {
		return 'px-3 py-1 text-xs border rounded cursor-pointer dark:border-gray-600' + (active ? ' bg-blue-50 dark:bg-blue-900/30' : '');
	}

	let chartLabels = $derived(data.period.map((r) => r.date.slice(5)));
	let chartDatasets = $derived([
		{ label: 'Надой (л)', data: data.period.map((r) => r.total_milk), color: '#2563eb' },
		{ label: 'Средн./корова', data: data.period.map((r) => r.avg_day_production), color: '#f59e0b' },
	]);
</script>

<div class="mb-4 grid grid-cols-2 md:grid-cols-5 gap-3">
	<div class="bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3">
		<div class="text-xs text-slate-500 dark:text-slate-400">Среднее поголовье</div>
		<div class="text-lg font-semibold">{fmtNum(data.avg_cow_count, 0)}</div>
	</div>
	<div class="bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3">
		<div class="text-xs text-slate-500 dark:text-slate-400">Средний надой/день</div>
		<div class="text-lg font-semibold">{fmtNum(data.avg_milk)} л</div>
	</div>
	<div class="bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3">
		<div class="text-xs text-slate-500 dark:text-slate-400">Среднее доек/день</div>
		<div class="text-lg font-semibold">{fmtNum(data.avg_milkings, 0)}</div>
	</div>
	<div class="bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3">
		<div class="text-xs text-slate-500 dark:text-slate-400">Среднее отказов/день</div>
		<div class="text-lg font-semibold">{fmtNum(data.avg_failures, 0)}</div>
	</div>
	<div class="bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3">
		<div class="text-xs text-slate-500 dark:text-slate-400">Средний SCC</div>
		<div class="text-lg font-semibold">{fmtNum(data.avg_scc, 0)}</div>
	</div>
</div>

<div class="flex gap-2 mb-3">
	<button class={btnClass(showChart)} onclick={() => showChart = true}>График</button>
	<button class={btnClass(!showChart)} onclick={() => showChart = false}>Таблица</button>
</div>

{#if showChart}
	<div class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4">
		<ReportChart labels={chartLabels} datasets={chartDatasets} />
	</div>
{:else}
	<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
		<table class={tblCls}>
			<thead class="bg-slate-50 dark:bg-slate-900/50">
				<tr>
					<th class={thCls}>Дата</th><th class={thCls}>Коров</th><th class={thCls}>Надой (л)</th>
					<th class={thCls}>Средн./корова</th><th class={thHidden}>Доек</th><th class={thHidden}>Отказов</th>
					<th class={thHidden}>Неудач</th><th class={thHidden}>Сепарация</th><th class={thHidden}>SCC</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
				{#each data.period as row (row.date)}
					<tr>
						<td class={tdCls}>{row.date}</td><td class={tdCls}>{row.cow_count}</td>
						<td class={tdCls}>{fmtNum(row.total_milk)}</td><td class={tdCls}>{fmtNum(row.avg_day_production)}</td>
						<td class={tdHidden}>{row.total_milkings ?? '—'}</td><td class={tdHidden}>{row.total_refusals ?? '—'}</td>
						<td class={tdHidden}>{row.total_failures ?? '—'}</td><td class={tdHidden}>{row.milk_separated ?? '—'}</td>
						<td class={tdHidden}>{fmtNum(row.avg_scc, 0)}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}
