<script lang="ts">
	import type { FeedPerTypeResponse } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls } from '../_shared';

	let { data }: { data: FeedPerTypeResponse } = $props();
</script>

<div class="mb-4 bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3">
	<span class="text-sm text-slate-500 dark:text-slate-400">Общие затраты: </span>
	<span class="font-semibold">{fmtNum(data.total_cost)} руб.</span>
	<span class="text-sm text-slate-500 dark:text-slate-400 ml-4">Ср. на 100 л: </span>
	<span class="font-semibold">{fmtNum(data.avg_cost_per_100milk)} руб.</span>
</div>
<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Дата</th><th class={thCls}>Тип</th><th class={thCls}>Название</th>
				<th class={thCls}>Продукт (кг)</th><th class={thCls}>Сухое в-во (кг)</th>
				<th class={thCls}>Стоимость</th><th class={thCls}>На 100л молока</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each data.rows as row (row.date + '-' + row.feed_type)}
				<tr>
					<td class={tdCls}>{row.date}</td><td class={tdCls}>{row.feed_type}</td>
					<td class={tdCls}>{row.feed_type_name}</td>
					<td class={tdCls}>{fmtNum(row.total_amount_product)}</td>
					<td class={tdCls}>{fmtNum(row.total_amount_dm)}</td>
					<td class={tdCls}>{fmtNum(row.total_cost)}</td>
					<td class={tdCls}>{fmtNum(row.cost_per_100milk)}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
