<script lang="ts">
	import type { VisitBehaviorRow } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls } from '../_shared';

	let { rows }: { rows: VisitBehaviorRow[] } = $props();
</script>

<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Животное</th><th class={thCls}>Доек</th><th class={thCls}>Отказов</th>
				<th class={thCls}>Средн./доение (л)</th><th class={thCls}>Средн. время (с)</th><th class={thCls}>Посл. визит</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each rows as row (row.animal_id)}
				<tr>
					<td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
					<td class={tdCls}>{row.total_milkings}</td>
					<td class={tdCls}>{row.total_refusals}</td>
					<td class={tdCls}><span class={row.avg_milk_per_milking && row.avg_milk_per_milking < 8 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{fmtNum(row.avg_milk_per_milking)}</span></td>
					<td class={tdCls}>{fmtNum(row.avg_duration_seconds, 0)}</td>
					<td class={tdCls}>{row.last_visit ? new Date(row.last_visit).toLocaleString('ru-RU') : '—'}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
