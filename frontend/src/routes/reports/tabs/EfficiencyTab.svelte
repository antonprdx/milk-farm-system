<script lang="ts">
	import type { CowRobotEfficiencyRow } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls } from '../_shared';

	let { rows }: { rows: CowRobotEfficiencyRow[] } = $props();
</script>

<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Животное</th><th class={thCls}>Молоко/мин/нед</th><th class={thCls}>Ср. скорость</th>
				<th class={thCls}>Время обработки</th><th class={thCls}>Время доения</th><th class={thCls}>Доек/7д</th>
				<th class={thCls}>Надой/7д</th><th class={thCls}>Средн./доение</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each rows as row (row.animal_id)}
				<tr>
					<td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
					<td class={tdCls}><span class={row.milk_per_box_time_week && row.milk_per_box_time_week < 0.5 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{fmtNum(row.milk_per_box_time_week, 3)}</span></td>
					<td class={tdCls}>{fmtNum(row.avg_milk_speed)}</td>
					<td class={tdCls}>{fmtNum(row.avg_treatment_time)} мин</td>
					<td class={tdCls}>{fmtNum(row.avg_milking_time)} мин</td>
					<td class={tdCls}>{row.milkings_7d}</td>
					<td class={tdCls}>{fmtNum(row.total_milk_7d)}</td>
					<td class={tdCls}>{fmtNum(row.avg_milk_per_milking)}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
