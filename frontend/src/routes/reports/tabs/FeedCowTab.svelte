<script lang="ts">
	import type { FeedPerCowDayRow } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls } from '../_shared';

	let { rows }: { rows: FeedPerCowDayRow[] } = $props();
</script>

<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Дата</th><th class={thCls}>Коров</th><th class={thCls}>Корм/корова</th>
				<th class={thCls}>Концентрат</th><th class={thCls}>Грубый</th><th class={thCls}>Стоим./корова</th>
				<th class={thCls}>Жвачка (мин)</th><th class={thCls}>Надой (л)</th><th class={thCls}>Дни лакт.</th>
				<th class={thCls}>Эфф. корма</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each rows as row (row.date)}
				<tr>
					<td class={tdCls}>{row.date}</td><td class={tdCls}>{row.animal_count}</td>
					<td class={tdCls}>{fmtNum(row.avg_total_per_cow)}</td>
					<td class={tdCls}>{fmtNum(row.avg_concentrate_per_cow)}</td>
					<td class={tdCls}>{fmtNum(row.avg_roughage_per_cow)}</td>
					<td class={tdCls}>{fmtNum(row.avg_cost_per_cow)}</td>
					<td class={tdCls}>{fmtNum(row.avg_rumination_minutes, 0)}</td>
					<td class={tdCls}>{fmtNum(row.avg_day_production)}</td>
					<td class={tdCls}>{fmtNum(row.avg_lactation_days, 0)}</td>
					<td class={tdCls}>{fmtNum(row.feed_efficiency)}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
