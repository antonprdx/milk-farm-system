<script lang="ts">
	import type { MilkDayProductionTimeRow } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls } from '../_shared';

	let { rows }: { rows: MilkDayProductionTimeRow[] } = $props();
</script>

<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Дата</th><th class={thCls}>Коров</th><th class={thCls}>Надой (л)</th>
				<th class={thCls}>Средн./корова</th><th class={thCls}>Доек</th><th class={thCls}>Отказов</th>
				<th class={thCls}>Неудач</th><th class={thCls}>Вес</th><th class={thCls}>Корм (кг)</th><th class={thCls}>Ост. корм</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each rows as row (row.date)}
				<tr>
					<td class={tdCls}>{row.date}</td><td class={tdCls}>{row.cow_count}</td>
					<td class={tdCls}>{fmtNum(row.total_milk)}</td><td class={tdCls}>{fmtNum(row.avg_milk_per_cow)}</td>
					<td class={tdCls}>{row.milkings ?? '—'}</td><td class={tdCls}>{row.refusals ?? '—'}</td>
					<td class={tdCls}>{row.failures ?? '—'}</td><td class={tdCls}>{fmtNum(row.avg_weight)}</td>
					<td class={tdCls}>{fmtNum(row.total_feed)}</td><td class={tdCls}>{row.total_rest_feed ?? '—'}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
