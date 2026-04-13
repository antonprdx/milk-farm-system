<script lang="ts">
	import {
		listActivities,
		listRuminations,
		type Activity,
		type Rumination,
	} from '$lib/api/fitness';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import FilterBar from '$lib/components/ui/FilterBar.svelte';
	import TabBar from '$lib/components/ui/TabBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { formatDatetime } from '$lib/utils/format';
	import { usePaginatedList } from '$lib/utils/usePaginatedList.svelte';

	let { data: pageData } = $props();

	type Tab = 'activities' | 'ruminations';

	let tab = $state<Tab>('activities');

	let dtActivities: DataTable | undefined = $state();
	let dtRuminations: DataTable | undefined = $state();
	let activities = $state<Activity[]>([]);
	let ruminations = $state<Rumination[]>([]);

	const list = usePaginatedList({ perPage: 50 });
	let _skipLoad = !!pageData.initialData;
	let _hasInitial = $state(!!pageData.initialData);

	if (pageData.initialData) {
		activities = pageData.initialData.data;
	}

	async function load() {
		const params = {
			animal_id: list.animalId || undefined,
			from_date: list.fromDate || undefined,
			till_date: list.tillDate || undefined,
			page: list.page,
			per_page: list.perPage,
		};
		if (tab === 'activities') {
			await list.load(
				(signal) => listActivities(params, signal),
				(d) => {
					activities = d;
				},
				dtActivities,
			);
		} else {
			await list.load(
				(signal) => listRuminations(params, signal),
				(d) => {
					ruminations = d;
				},
				dtRuminations,
			);
		}
	}

	function switchTab(t: Tab) {
		tab = t;
		list.resetPage();
		load();
	}

	$effect(() => {
		list.page;
		if (_skipLoad) {
			_skipLoad = false;
			return;
		}
		load();
	});
</script>

<svelte:head>
	<title>Фитнес — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-6">Фитнес</h1>

<TabBar
	tabs={[
		{ key: 'activities', label: 'Активность' },
		{ key: 'ruminations', label: 'Жвачка' },
	]}
	bind:active={tab}
	onchange={(t: string) => switchTab(t as Tab)}
/>

<FilterBar
	bind:fromDate={list.fromDate}
	bind:tillDate={list.tillDate}
	bind:animalId={list.animalId}
	onsearch={load}
/>
<ErrorAlert message={list.error} />

{#if tab === 'activities'}
	<DataTable
		columns={[
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'activity_datetime', label: 'Время' },
			{ key: 'activity_counter', label: 'Счётчик активности', align: 'right' },
			{ key: 'heat_attention', label: 'Внимание (охота)', align: 'center' },
		]}
		loading={list.loading}
		bind:this={dtActivities}
		emptyText="Нет данных"
	>
		{#each activities as a, i (i)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3"
					><a
						href="/animals/{a.animal_id}"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
						>#{a.animal_id}</a
					></td
				>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400"
					>{formatDatetime(a.activity_datetime)}</td
				>
				<td class="px-4 py-3 text-right font-medium">{a.activity_counter ?? '—'}</td>
				<td class="px-4 py-3 text-center">
					{#if a.heat_attention}
						<span
							class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-orange-100 dark:bg-orange-900/50 text-orange-700"
							>Да</span
						>
					{:else}
						<span class="text-slate-400 dark:text-slate-500">—</span>
					{/if}
				</td>
			</tr>
		{/each}
	</DataTable>
	<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />
{:else}
	<DataTable
		columns={[
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'date', label: 'Дата' },
			{ key: 'eating_seconds', label: 'Приём пищи, мин', align: 'right' },
			{ key: 'rumination_minutes', label: 'Жвачка, мин', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtRuminations}
		emptyText="Нет данных"
	>
		{#each ruminations as r, i (i)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3"
					><a
						href="/animals/{r.animal_id}"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
						>#{r.animal_id}</a
					></td
				>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{r.date}</td>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{r.eating_seconds != null ? Math.round(r.eating_seconds / 60) : '—'}</td
				>
				<td class="px-4 py-3 text-right font-medium">{r.rumination_minutes ?? '—'}</td>
			</tr>
		{/each}
	</DataTable>
	<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />
{/if}
