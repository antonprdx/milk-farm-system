<script lang="ts">
	import { listLocations, type Location } from '$lib/api/locations';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { usePaginatedList } from '$lib/utils/usePaginatedList.svelte';
	import { MapPin } from 'lucide-svelte';

	let { data } = $props();

	let dataTable: DataTable;
	let locations = $state<Location[]>([]);

	const list = usePaginatedList();
	let _skipLoad = !!data.initialData;
	let _hasInitial = $state(!!data.initialData);

	if (data.initialData) {
		locations = data.initialData.data;
	}

	async function load() {
		await list.load(
			(signal) => listLocations({ page: list.page, per_page: list.perPage }, signal),
			(data) => {
				locations = data;
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
	<title>Локации — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Локации</h1>
</div>

<ErrorAlert message={list.error} />

<DataTable
	columns={[
		{ key: 'name', label: 'Название' },
		{ key: 'location_type', label: 'Тип' },
		{ key: 'created_at', label: 'Создано' },
	]}
	loading={list.loading}
	bind:this={dataTable}
	emptyText="Нет локаций"
>
	{#each locations as loc (loc.id)}
		<tr
			class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
		>
			<td class="px-4 py-3 font-medium text-slate-800 dark:text-slate-100">
				<span class="inline-flex items-center gap-2">
					<MapPin size={14} class="text-slate-400" />
					{loc.name}
				</span>
			</td>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{loc.location_type || '—'}</td>
			<td class="px-4 py-3 text-slate-500 dark:text-slate-400 text-sm">
				{new Date(loc.created_at).toLocaleDateString('ru-RU')}
			</td>
		</tr>
	{/each}
</DataTable>

<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />
