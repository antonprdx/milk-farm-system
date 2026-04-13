<script lang="ts">
	import type { RobotPerformanceRow } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tdHidden, thHidden, tblCls } from '../_shared';

	let { rows }: { rows: RobotPerformanceRow[] } = $props();
</script>

<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Робот</th><th class={thCls}>Дата</th><th class={thCls}>Ср. скорость</th>
				<th class={thHidden}>Макс. скорость</th><th class={thCls}>Доек</th>
				<th class={thHidden}>LF время</th><th class={thHidden}>LR время</th><th class={thHidden}>RF время</th><th class={thHidden}>RR время</th>
				<th class={thHidden}>LF DMT</th><th class={thHidden}>LR DMT</th><th class={thHidden}>RF DMT</th><th class={thHidden}>RR DMT</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each rows as row (row.device_address + '-' + row.date)}
				<tr>
					<td class={tdCls}>{row.device_address ?? '—'}</td><td class={tdCls}>{row.date}</td>
					<td class={tdCls}>{fmtNum(row.avg_milk_speed)}</td><td class={tdHidden}>{fmtNum(row.max_milk_speed)}</td>
					<td class={tdCls}>{row.milkings}</td>
					<td class={tdHidden}>{fmtNum(row.avg_lf_milk_time, 0)}</td><td class={tdHidden}>{fmtNum(row.avg_lr_milk_time, 0)}</td>
					<td class={tdHidden}>{fmtNum(row.avg_rf_milk_time, 0)}</td><td class={tdHidden}>{fmtNum(row.avg_rr_milk_time, 0)}</td>
					<td class={tdHidden}><span class={row.avg_lf_dead_milk_time && row.avg_lf_dead_milk_time > 60 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{fmtNum(row.avg_lf_dead_milk_time, 0)}</span></td>
					<td class={tdHidden}><span class={row.avg_lr_dead_milk_time && row.avg_lr_dead_milk_time > 60 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{fmtNum(row.avg_lr_dead_milk_time, 0)}</span></td>
					<td class={tdHidden}><span class={row.avg_rf_dead_milk_time && row.avg_rf_dead_milk_time > 60 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{fmtNum(row.avg_rf_dead_milk_time, 0)}</span></td>
					<td class={tdHidden}><span class={row.avg_rr_dead_milk_time && row.avg_rr_dead_milk_time > 60 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{fmtNum(row.avg_rr_dead_milk_time, 0)}</span></td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
