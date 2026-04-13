<script lang="ts">
	import type { FailedMilkingRow } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls } from '../_shared';

	let { rows }: { rows: FailedMilkingRow[] } = $props();
</script>

<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Животное</th><th class={thCls}>Время</th><th class={thCls}>Робот</th>
				<th class={thCls}>Надой</th><th class={thCls}>LF конд.</th><th class={thCls}>LR конд.</th>
				<th class={thCls}>RF конд.</th><th class={thCls}>RR конд.</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each rows as row (row.animal_id + '-' + row.visit_datetime)}
				<tr>
					<td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
					<td class={tdCls}>{new Date(row.visit_datetime).toLocaleString('ru-RU')}</td>
					<td class={tdCls}>{row.device_address ?? '—'}</td>
					<td class={tdCls}>{fmtNum(row.milk_yield)}</td>
					<td class={tdCls}><span class={row.lf_conductivity && row.lf_conductivity > 83 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{row.lf_conductivity ?? '—'}</span></td>
					<td class={tdCls}><span class={row.lr_conductivity && row.lr_conductivity > 83 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{row.lr_conductivity ?? '—'}</span></td>
					<td class={tdCls}><span class={row.rf_conductivity && row.rf_conductivity > 83 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{row.rf_conductivity ?? '—'}</span></td>
					<td class={tdCls}><span class={row.rr_conductivity && row.rr_conductivity > 83 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{row.rr_conductivity ?? '—'}</span></td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
