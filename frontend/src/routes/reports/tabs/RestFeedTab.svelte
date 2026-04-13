<script lang="ts">
	import type { RestFeedResponse } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls } from '../_shared';

	let { data }: { data: RestFeedResponse } = $props();
</script>

{#if data.total_rest_feed_pct != null}
	<div class="mb-4 bg-white dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-700 p-3">
		<span class="text-sm text-slate-500 dark:text-slate-400">Общий % остатка корма: </span>
		<span
			class="font-semibold {data.total_rest_feed_pct > 5
				? 'text-red-600 dark:text-red-400'
				: 'text-green-600 dark:text-green-400'}"
		>
			{fmtNum(data.total_rest_feed_pct)}%
		</span>
	</div>
{/if}
<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Животное</th><th class={thCls}>Дата</th><th class={thCls}>№ Корма</th>
				<th class={thCls}>План (кг)</th><th class={thCls}>Остаток</th><th class={thCls}>%</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each data.rows as row (row.animal_id + '-' + row.feed_date + '-' + row.feed_number)}
				<tr>
					<td class={tdCls}>{row.animal_name ?? row.animal_id}</td><td class={tdCls}>{row.feed_date}</td>
					<td class={tdCls}>{row.feed_number}</td><td class={tdCls}>{fmtNum(row.total_planned)}</td>
					<td class={tdCls}>{row.rest_feed ?? '—'}</td>
					<td class={tdCls}>
						<span class={row.rest_feed_pct && row.rest_feed_pct > 10 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{fmtNum(row.rest_feed_pct)}%</span>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
