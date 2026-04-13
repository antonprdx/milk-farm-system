<script lang="ts">
	import type { HealthActivityRow } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls, badgeRed, badgeYellow } from '../_shared';

	let { rows }: { rows: HealthActivityRow[] } = $props();
</script>

<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Животное</th><th class={thCls}>Health Index</th><th class={thCls}>Откл. активн.</th>
				<th class={thCls}>Жвачка (мин)</th><th class={thCls}>Макс.изм. 24ч</th><th class={thCls}>Разн. 3 дня</th>
				<th class={thCls}>Надой</th><th class={thCls}>Ср. 7д</th><th class={thCls}>Откл. %</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each rows as row (row.animal_id)}
				<tr>
					<td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
					<td class={tdCls}><span class={row.health_index && row.health_index < 75 ? badgeRed : row.health_index && row.health_index < 80 ? badgeYellow : ''}>{fmtNum(row.health_index, 0)}</span></td>
					<td class={tdCls}>{fmtNum(row.activity_deviation, 0)}</td>
					<td class={tdCls}>{row.rumination_minutes ?? '—'}</td>
					<td class={tdCls}>{row.max_rumination_change_24h ?? '—'}</td>
					<td class={tdCls}><span class={row.rumination_3day_diff && row.rumination_3day_diff < -60 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{row.rumination_3day_diff ?? '—'}</span></td>
					<td class={tdCls}>{fmtNum(row.latest_milk)}</td>
					<td class={tdCls}>{fmtNum(row.avg_milk_7d)}</td>
					<td class={tdCls}>{fmtNum(row.milk_deviation_pct)}%</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
