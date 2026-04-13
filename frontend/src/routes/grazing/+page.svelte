<script lang="ts">
	import { listGrazing, type GrazingData } from '$lib/api/grazing';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import FilterBar from '$lib/components/ui/FilterBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { usePaginatedList } from '$lib/utils/usePaginatedList.svelte';

	let { data: pageData } = $props();

	let dataTable: DataTable;
	let data = $state<GrazingData[]>([]);

	const list = usePaginatedList({ perPage: 50 });
	let _skipLoad = !!pageData.initialData;
	let _hasInitial = $state(!!pageData.initialData);

	if (pageData.initialData) {
		data = pageData.initialData.data;
	}

	async function load() {
		await list.load(
			(signal) =>
				listGrazing(
					{
						from_date: list.fromDate || undefined,
						till_date: list.tillDate || undefined,
						page: list.page,
						per_page: list.perPage,
					},
					signal,
				),
			(d) => {
				data = d;
			},
			dataTable,
		);
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
	<title>Пастьба — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-6">Пастьба</h1>

<FilterBar
	bind:fromDate={list.fromDate}
	bind:tillDate={list.tillDate}
	showAnimal={false}
	onsearch={load}
/>
<ErrorAlert message={list.error} />

<DataTable
	columns={[
		{ key: 'date', label: 'Дата' },
		{ key: 'animal_count', label: 'Животных', align: 'right' },
		{ key: 'pasture_time', label: 'Время на пастбище, мин', align: 'right' },
		{ key: 'lactation_period', label: 'Период лактации' },
	]}
	loading={list.loading}
	bind:this={dataTable}
	emptyText="Нет данных"
>
	{#each data as g, i (i)}
		<tr
			class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
		>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{g.date}</td>
			<td class="px-4 py-3 text-right font-medium">{g.animal_count ?? '—'}</td>
			<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
				>{g.pasture_time ?? '—'}</td
			>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{g.lactation_period || '—'}</td>
		</tr>
	{/each}
</DataTable>
<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />
