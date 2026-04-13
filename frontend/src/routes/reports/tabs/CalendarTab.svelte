<script lang="ts">
	import type { CalendarResponse } from '$lib/api/reports';
	import { thCls, tdCls, tblCls, badgeRed, badgeYellow } from '../_shared';

	let { data }: { data: CalendarResponse } = $props();
</script>

<div class="space-y-6">
	<div>
		<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">Ожидаемые отёлы (R31)</h2>
		<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr>
						<th class={thCls}>Животное</th><th class={thCls}>Лактация</th><th class={thCls}>Последняя инсем.</th>
						<th class={thCls}>Ожид. отёл</th><th class={thCls}>Дней до отёла</th><th class={thCls}>Бык</th><th class={thCls}>Дней стельности</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each data.expected_calvings as row (row.animal_id)}
						<tr>
							<td class={tdCls}>{row.animal_name ?? row.animal_id}</td><td class={tdCls}>{row.lac_number ?? '—'}</td>
							<td class={tdCls}>{row.last_insemination_date ?? '—'}</td><td class={tdCls}>{row.expected_calving_date ?? '—'}</td>
							<td class={tdCls}><span class={row.days_until_calving && row.days_until_calving < 14 ? badgeRed : row.days_until_calving && row.days_until_calving < 30 ? badgeYellow : ''}>{row.days_until_calving ?? '—'}</span></td>
							<td class={tdCls}>{row.sire_code ?? '—'}</td><td class={tdCls}>{row.days_pregnant ?? '—'}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
	<div>
		<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">Ожидаемые запуски (R32)</h2>
		<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr>
						<th class={thCls}>Животное</th><th class={thCls}>Ожид. отёл</th><th class={thCls}>Реком. запуск</th>
						<th class={thCls}>Дней до запуска</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each data.expected_dry_offs as row (row.animal_id)}
						<tr>
							<td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
							<td class={tdCls}>{row.expected_calving_date ?? '—'}</td>
							<td class={tdCls}>{row.recommended_dry_off_date ?? '—'}</td>
							<td class={tdCls}><span class={row.days_until_dry_off && row.days_until_dry_off < 7 ? badgeRed : row.days_until_dry_off && row.days_until_dry_off < 14 ? badgeYellow : ''}>{row.days_until_dry_off ?? '—'}</span></td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
	<div>
		<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">Ожидаемая охота (R33)</h2>
		<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr>
						<th class={thCls}>Животное</th><th class={thCls}>Посл. охота</th><th class={thCls}>Ожид. охота</th>
						<th class={thCls}>Дней до</th><th class={thCls}>Дней в лакт.</th><th class={thCls}>Осеменена</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each data.expected_heats as row (row.animal_id)}
						<tr>
							<td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
							<td class={tdCls}>{row.last_heat_date ?? '—'}</td>
							<td class={tdCls}>{row.expected_heat_date ?? '—'}</td>
							<td class={tdCls}><span class={row.overdue ? badgeRed : row.days_until_heat && row.days_until_heat < 3 ? badgeYellow : ''}>{row.days_until_heat ?? '—'}</span></td>
							<td class={tdCls}>{row.days_in_lactation ?? '—'}</td>
							<td class={tdCls}>{row.inseminated ? 'Да' : 'Нет'}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
	<div>
		<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">Проверка стельности (R34)</h2>
		<div class="overflow-x-auto bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700">
			<table class={tblCls}>
				<thead class="bg-slate-50 dark:bg-slate-900/50">
					<tr>
						<th class={thCls}>Животное</th><th class={thCls}>Дата инсем.</th><th class={thCls}>Бык</th>
						<th class={thCls}>Дней после инсем.</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-200 dark:divide-slate-700">
					{#each data.pregnancy_checks as row (row.animal_id)}
						<tr>
							<td class={tdCls}>{row.animal_name ?? row.animal_id}</td>
							<td class={tdCls}>{row.insemination_date ?? '—'}</td>
							<td class={tdCls}>{row.sire_code ?? '—'}</td>
							<td class={tdCls}><span class={badgeYellow}>{row.days_since_insemination ?? '—'}</span></td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
</div>
