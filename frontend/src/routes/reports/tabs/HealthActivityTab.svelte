<script lang="ts">
	import type { HealthActivityRow } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tdHidden, thHidden, tblCls, badgeRed, badgeYellow } from '../_shared';
	import ReportChart from '$lib/components/ReportChart.svelte';

	let { rows }: { rows: HealthActivityRow[] } = $props();

	let showChart = $state(true);

	function btnClass(active: boolean): string {
		return 'px-3 py-1 text-xs border rounded cursor-pointer dark:border-gray-600' + (active ? ' bg-blue-50 dark:bg-blue-900/30' : '');
	}

	let chartLabels = $derived(rows.slice(0, 30).map((r) => r.animal_name ?? `#${r.animal_id}`));
	let chartDatasets = $derived([
		{ label: 'Health Index', data: rows.slice(0, 30).map((r) => r.health_index ?? null), color: '#2563eb' },
		{ label: 'Жвачка (мин)', data: rows.slice(0, 30).map((r) => r.rumination_minutes ?? null), color: '#10b981' },
	]);
</script>

<div class="flex gap-2 mb-3">
	<button class={btnClass(showChart)} onclick={() => showChart = true}>График</button>
	<button class={btnClass(!showChart)} onclick={() => showChart = false}>Таблица</button>
</div>

{#if showChart}
	<div class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border p-4">
		<ReportChart labels={chartLabels} datasets={chartDatasets} type="bar" height="300px" />
	</div>
{:else}
	<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
		<table class={tblCls}>
			<thead class="bg-slate-50 dark:bg-slate-900/50">
				<tr>
					<th class={thCls}>Животное</th><th class={thCls}>Health Index</th><th class={thHidden}>Откл. активн.</th>
					<th class={thCls}>Жвачка (мин)</th><th class={thHidden}>Макс.изм. 24ч</th><th class={thHidden}>Разн. 3 дня</th>
					<th class={thCls}>Надой</th><th class={thHidden}>Ср. 7д</th><th class={thHidden}>Откл. %</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
				{#each rows as row (row.animal_id)}
					<tr>
						<td class={tdCls}><a href="/animals/{row.animal_id}" class="text-blue-600 dark:text-blue-400 hover:underline">{row.animal_name ?? row.animal_id}</a></td>
						<td class={tdCls}><span class={row.health_index && row.health_index < 75 ? badgeRed : row.health_index && row.health_index < 80 ? badgeYellow : ''}>{fmtNum(row.health_index, 0)}</span></td>
						<td class={tdHidden}>{fmtNum(row.activity_deviation, 0)}</td>
						<td class={tdCls}>{row.rumination_minutes ?? '—'}</td>
						<td class={tdHidden}>{row.max_rumination_change_24h ?? '—'}</td>
						<td class={tdHidden}><span class={row.rumination_3day_diff && row.rumination_3day_diff < -60 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{row.rumination_3day_diff ?? '—'}</span></td>
						<td class={tdCls}>{fmtNum(row.latest_milk)}</td>
						<td class={tdHidden}>{fmtNum(row.avg_milk_7d)}</td>
						<td class={tdHidden}>{fmtNum(row.milk_deviation_pct)}%</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}
