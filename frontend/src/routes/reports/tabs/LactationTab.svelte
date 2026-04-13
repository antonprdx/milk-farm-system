<script lang="ts">
	import type { LactationAnalysisResponse } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls } from '../_shared';

	let { rows }: { rows: LactationAnalysisResponse[] } = $props();
</script>

{#each rows as lac (lac.lac_number)}
	<div class="mb-6">
		<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">Лактация {lac.lac_number}</h2>
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
	</div>
{/each}
