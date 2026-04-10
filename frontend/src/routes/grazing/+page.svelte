<script lang="ts">
	import { listGrazing, type GrazingData } from '$lib/api/grazing';
	import { onMount } from 'svelte';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import FilterBar from '$lib/components/ui/FilterBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';

	let data = $state<GrazingData[]>([]);
	let loading = $state(true);
	let error = $state('');

	let fromDate = $state('');
	let tillDate = $state('');

	let page = $state(1);
	let total = $state(0);
	let initialized = $state(false);

	async function load() {
		try {
			loading = true;
			error = '';
			const res = await listGrazing({
				from_date: fromDate || undefined,
				till_date: tillDate || undefined,
				page,
				per_page: 50,
			});
			data = res.data;
			total = res.total;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		initialized = true;
		load();
	});
	$effect(() => {
		page;
		if (initialized) load();
	});
</script>

<svelte:head>
	<title>Пастьба — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-6">Пастьба</h1>

<FilterBar bind:fromDate bind:tillDate showAnimal={false} onsearch={load} />
<ErrorAlert message={error} />

<DataTable
	columns={[
		{ key: 'date', label: 'Дата' },
		{ key: 'animal_count', label: 'Животных', align: 'right' },
		{ key: 'pasture_time', label: 'Время на пастбище, мин', align: 'right' },
		{ key: 'lactation_period', label: 'Период лактации' },
	]}
	{loading}
>
	{#if data.length === 0}
		<tr
			><td colspan="4" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
				>Нет данных</td
			></tr
		>
	{:else}
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
	{/if}
</DataTable>
<Pagination bind:page {total} perPage={50} />
