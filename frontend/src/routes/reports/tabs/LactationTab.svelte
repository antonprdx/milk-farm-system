<script lang="ts">
	import type { LactationAnalysisResponse } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls } from '../_shared';
	import ReportChart from '$lib/components/ReportChart.svelte';

	let { rows }: { rows: LactationAnalysisResponse[] } = $props();

	let showChart = $state(true);

	function btnClass(active: boolean): string {
		return 'px-3 py-1 text-xs border rounded cursor-pointer dark:border-gray-600' + (active ? ' bg-blue-50 dark:bg-blue-900/30' : '');
	}
</script>

{#each rows as lac (lac.lac_number)}
	<div class="mb-6">
		<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">Лактация {lac.lac_number}</h2>

		<div class="flex gap-2 mb-3">
			<button class={btnClass(showChart)} onclick={() => showChart = true}>График</button>
			<button class={btnClass(!showChart)} onclick={() => showChart = false}>Таблица</button>
		</div>

		{#if showChart}
			<div class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border p-4">
				<ReportChart
					labels={lac.points.map((p) => String(p.dim))}
					datasets={[
						{ label: 'Средн. надой', data: lac.points.map((p) => p.avg_milk), color: '#2563eb' },
						{ label: 'Корм', data: lac.points.map((p) => p.avg_feed), color: '#f59e0b' },
					]}
				/>
			</div>
		{:else}
			<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
				<table class={tblCls}>
					<thead class="bg-slate-50 dark:bg-slate-900/50">
						<tr>
							<th class={thCls}>DIM</th><th class={thCls}>Средн. надой</th><th class={thCls}>Визитов</th>
							<th class={thCls}>Корм</th><th class={thCls}>Вес</th><th class={thCls}>Жир%</th>
							<th class={thCls}>Белок%</th><th class={thCls}>Коров</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
						{#each lac.points as pt (pt.dim)}
							<tr>
								<td class={tdCls}>{pt.dim}</td><td class={tdCls}>{fmtNum(pt.avg_milk)}</td>
								<td class={tdCls}>{fmtNum(pt.avg_visits, 1)}</td><td class={tdCls}>{fmtNum(pt.avg_feed)}</td>
								<td class={tdCls}>{fmtNum(pt.avg_weight)}</td><td class={tdCls}>{fmtNum(pt.avg_fat)}</td>
								<td class={tdCls}>{fmtNum(pt.avg_protein)}</td><td class={tdCls}>{pt.cow_count}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>
{/each}
