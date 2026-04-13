<script lang="ts">
	import type { TransitionResponse } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls, badgeYellow } from '../_shared';

	let { data }: { data: TransitionResponse } = $props();
</script>

<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Животное</th><th class={thCls}>День (относ.)</th><th class={thCls}>Надой 24ч</th>
				<th class={thCls}>Жвачка 3д разн.</th><th class={thCls}>Жвачка (мин)</th>
				<th class={thCls}>Корм (кг)</th><th class={thCls}>Ост. корм</th><th class={thCls}>SCC</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each data.rows as row (row.animal_id)}
				<tr>
					<td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
					<td class={tdCls}><span class={row.days_relative === 0 ? badgeYellow : row.days_relative < 0 ? 'text-blue-600 dark:text-blue-400' : ''}>{row.days_relative > 0 ? '+' : ''}{row.days_relative}</span></td>
					<td class={tdCls}>{fmtNum(row.milk_24h)}</td>
					<td class={tdCls}><span class={row.rumination_3day_diff && row.rumination_3day_diff < -60 ? 'text-red-600 dark:text-red-400 font-medium' : ''}>{row.rumination_3day_diff ?? '—'}</span></td>
					<td class={tdCls}>{row.rumination_minutes ?? '—'}</td>
					<td class={tdCls}>{fmtNum(row.feed_total)}</td>
					<td class={tdCls}>{row.feed_rest ?? '—'}</td>
					<td class={tdCls}>{row.latest_scc ?? '—'}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
