<script lang="ts">
	import type { PregnancyRateResponse } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls } from '../_shared';

	let { data }: { data: PregnancyRateResponse } = $props();
</script>

<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Конец периода</th><th class={thCls}>Пригодных</th><th class={thCls}>Осеменено</th>
				<th class={thCls}>Стельных</th><th class={thCls}>% осеменения</th><th class={thCls}>% зачатия</th>
				<th class={thCls}>Коэфф. стельности</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each data.periods as row (row.end_date)}
				<tr>
					<td class={tdCls}>{row.end_date}</td><td class={tdCls}>{row.eligible}</td>
					<td class={tdCls}>{row.inseminated}</td><td class={tdCls}>{row.pregnant}</td>
					<td class={tdCls}>{fmtNum(row.insemination_rate)}%</td>
					<td class={tdCls}>{fmtNum(row.conception_rate)}%</td>
					<td class={tdCls}><span class={row.pregnancy_rate && row.pregnancy_rate < 25 ? 'text-red-600 dark:text-red-400 font-medium' : 'font-medium'}>{fmtNum(row.pregnancy_rate)}%</span></td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
