<script lang="ts">
	import type { HealthTaskResponse } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';
	import { thCls, tdCls, tblCls, statusBadge } from '../_shared';

	let { data }: { data: HealthTaskResponse } = $props();
</script>

<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
	<table class={tblCls}>
		<thead class="bg-slate-50 dark:bg-slate-900/50">
			<tr>
				<th class={thCls}>Животное</th><th class={thCls}>Sick Chance</th><th class={thCls}>Статус</th>
				<th class={thCls}>Падение удоя</th><th class={thCls}>Конд.</th><th class={thCls}>SCC</th>
				<th class={thCls}>Откл. активн.</th><th class={thCls}>Откл. жвачки</th><th class={thCls}>Жир/Белок</th>
				<th class={thCls}>Ост. корм %</th><th class={thCls}>Темп.</th><th class={thCls}>Цвет</th>
				<th class={thCls}>Дни лакт.</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
			{#each data.rows as row (row.animal_id)}
				<tr>
					<td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
					<td class={tdCls}><span class={statusBadge(row.sick_chance_status)}>{fmtNum(row.sick_chance, 0)}</span></td>
					<td class={tdCls}><span class={statusBadge(row.sick_chance_status)}>{row.sick_chance_status}</span></td>
					<td class={tdCls}>{fmtNum(row.milk_drop_kg)}</td>
					<td class={tdCls}>{row.conductivity_highest ?? '—'}</td>
					<td class={tdCls}>{row.scc_indication ?? '—'}</td>
					<td class={tdCls}>{fmtNum(row.activity_deviation, 0)}</td>
					<td class={tdCls}>{row.rumination_deviation ?? '—'}</td>
					<td class={tdCls}>{fmtNum(row.fat_protein_ratio)}</td>
					<td class={tdCls}>{fmtNum(row.feed_rest_pct)}%</td>
					<td class={tdCls}>{fmtNum(row.temperature_highest)}</td>
					<td class={tdCls}>{row.colour_attentions.join(', ') || '—'}</td>
					<td class={tdCls}>{row.days_in_lactation ?? '—'}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
