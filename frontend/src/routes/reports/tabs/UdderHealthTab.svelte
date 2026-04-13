<script lang="ts">
	import type { UdderHealthRow } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls } from '../_shared';

	let { rows }: { rows: UdderHealthRow[] } = $props();
</script>

<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Животное</th><th class={thCls}>Время</th>
				<th class={thCls}>LF конд.</th><th class={thCls}>LR конд.</th><th class={thCls}>RF конд.</th><th class={thCls}>RR конд.</th>
				<th class={thCls}>Цвет</th><th class={thCls}>SCC</th><th class={thCls}>Отклон. удоя</th>
				<th class={thCls}>Attention</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each rows as row (row.animal_id + '-' + row.visit_datetime)}
				<tr>
					<td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
					<td class={tdCls}>{new Date(row.visit_datetime).toLocaleString('ru-RU')}</td>
					<td class={tdCls}><span class={row.lf_conductivity && row.lf_conductivity > 80 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{row.lf_conductivity ?? '—'}</span></td>
					<td class={tdCls}><span class={row.lr_conductivity && row.lr_conductivity > 80 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{row.lr_conductivity ?? '—'}</span></td>
					<td class={tdCls}><span class={row.rf_conductivity && row.rf_conductivity > 80 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{row.rf_conductivity ?? '—'}</span></td>
					<td class={tdCls}><span class={row.rr_conductivity && row.rr_conductivity > 80 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{row.rr_conductivity ?? '—'}</span></td>
					<td class={tdCls}>{[row.lf_colour, row.lr_colour, row.rf_colour, row.rr_colour].filter(Boolean).join(', ') || '—'}</td>
					<td class={tdCls}>{row.latest_scc ?? '—'}</td>
					<td class={tdCls}><span class={row.deviation_day_prod && row.deviation_day_prod < -3 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{fmtNum(row.deviation_day_prod)}</span></td>
					<td class={tdCls}>{row.attention_quarters.join('; ') || '—'}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
