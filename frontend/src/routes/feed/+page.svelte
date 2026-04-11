<script lang="ts">
	import {
		listDayAmounts,
		listVisits,
		listTypes,
		listGroups,
		type FeedDayAmount,
		type FeedVisit,
		type FeedType,
		type FeedGroup,
	} from '$lib/api/feed';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import FilterBar from '$lib/components/ui/FilterBar.svelte';
	import TabBar from '$lib/components/ui/TabBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { formatDatetime } from '$lib/utils/format';
	import { usePaginatedList } from '$lib/utils/usePaginatedList.svelte';

	type Tab = 'amounts' | 'visits' | 'types' | 'groups';

	let tab = $state<Tab>('amounts');

	let dtAmounts: DataTable | undefined = $state();
	let dtVisits: DataTable | undefined = $state();
	let dtTypes: DataTable | undefined = $state();
	let dtGroups: DataTable | undefined = $state();
	let amounts = $state<FeedDayAmount[]>([]);
	let visits = $state<FeedVisit[]>([]);
	let types = $state<FeedType[]>([]);
	let groups = $state<FeedGroup[]>([]);

	const list = usePaginatedList({ perPage: 50 });

	async function load() {
		const params = {
			animal_id: list.animalId || undefined,
			from_date: list.fromDate || undefined,
			till_date: list.tillDate || undefined,
			page: list.page,
			per_page: list.perPage,
		};
		switch (tab) {
			case 'amounts':
				await list.load(() => listDayAmounts(params), (d) => { amounts = d; }, dtAmounts);
				break;
			case 'visits':
				await list.load(() => listVisits(params), (d) => { visits = d; }, dtVisits);
				break;
			case 'types': {
				try {
					list.error = '';
					const res = await listTypes();
					types = res.data;
					dtTypes?.setHasRows(types.length > 0);
				} catch (e) {
					list.error = e instanceof Error ? e.message : 'Ошибка загрузки';
				}
				break;
			}
			case 'groups': {
				try {
					list.error = '';
					const res = await listGroups();
					groups = res.data;
					dtGroups?.setHasRows(groups.length > 0);
				} catch (e) {
					list.error = e instanceof Error ? e.message : 'Ошибка загрузки';
				}
				break;
			}
		}
	}

	function switchTab(t: Tab) {
		tab = t;
		list.resetPage();
		load();
	}

	$effect(() => {
		list.page;
		load();
	});
</script>

<svelte:head>
	<title>Кормление — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-6">Кормление</h1>

<TabBar
	tabs={[
		{ key: 'amounts', label: 'Дневные нормы' },
		{ key: 'visits', label: 'Визиты' },
		{ key: 'types', label: 'Типы кормов' },
		{ key: 'groups', label: 'Группы' },
	]}
	bind:active={tab}
	onchange={(t: string) => switchTab(t as Tab)}
/>

{#if tab === 'amounts' || tab === 'visits'}
	<FilterBar bind:fromDate={list.fromDate} bind:tillDate={list.tillDate} bind:animalId={list.animalId} onsearch={load} />
{/if}

<ErrorAlert message={list.error} />

{#if tab === 'amounts'}
	<DataTable
		columns={[
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'feed_date', label: 'Дата' },
			{ key: 'feed_number', label: '№ корма', align: 'right' },
			{ key: 'total', label: 'Всего, кг', align: 'right' },
			{ key: 'rest_feed', label: 'Остаток', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtAmounts}
		emptyText="Нет данных"
	>
		{#each amounts as a, i (i)}
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
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{a.feed_date}</td>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400">{a.feed_number}</td>
				<td class="px-4 py-3 text-right font-medium">{a.total.toFixed(1)}</td>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{a.rest_feed ?? '—'}</td
				>
			</tr>
		{/each}
	</DataTable>
	<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />
{:else if tab === 'visits'}
	<DataTable
		columns={[
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'visit_datetime', label: 'Время' },
			{ key: 'feed_number', label: '№ корма', align: 'right' },
			{ key: 'amount', label: 'Кол-во, кг', align: 'right' },
			{ key: 'duration_seconds', label: 'Длительность, с', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtVisits}
		emptyText="Нет данных"
	>
		{#each visits as v, i (i)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3"
					><a
						href="/animals/{v.animal_id}"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
						>#{v.animal_id}</a
					></td
				>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400"
					>{formatDatetime(v.visit_datetime)}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{v.feed_number ?? '—'}</td
				>
				<td class="px-4 py-3 text-right font-medium"
					>{v.amount != null ? v.amount.toFixed(1) : '—'}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{v.duration_seconds ?? '—'}</td
				>
			</tr>
		{/each}
	</DataTable>
	<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />
{:else if tab === 'types'}
	<DataTable
		columns={[
			{ key: 'number_of_feed_type', label: '№' },
			{ key: 'name', label: 'Название' },
			{ key: 'feed_type', label: 'Тип' },
			{ key: 'dry_matter_percentage', label: 'Сухое вещество, %', align: 'right' },
			{ key: 'price', label: 'Цена', align: 'right' },
			{ key: 'stock_attention_level', label: 'Уровень запаса', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtTypes}
		emptyText="Нет данных"
	>
		{#each types as t (t.number_of_feed_type)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{t.number_of_feed_type}</td>
				<td class="px-4 py-3 font-medium text-slate-800 dark:text-slate-100">{t.name}</td>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{t.feed_type}</td>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{t.dry_matter_percentage.toFixed(0)}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{t.price.toFixed(2)}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{t.stock_attention_level ?? '—'}</td
				>
			</tr>
		{/each}
	</DataTable>
{:else}
	<DataTable
		columns={[
			{ key: 'name', label: 'Название' },
			{ key: 'min_milk_yield', label: 'Надой мин', align: 'right' },
			{ key: 'max_milk_yield', label: 'Надой макс', align: 'right' },
			{ key: 'avg_milk_yield', label: 'Средний надой', align: 'right' },
			{ key: 'avg_weight', label: 'Средний вес', align: 'right' },
			{ key: 'number_of_cows', label: 'Коров', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtGroups}
		emptyText="Нет данных"
	>
		{#each groups as g (g.name)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3 font-medium text-slate-800 dark:text-slate-100">{g.name}</td>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{g.min_milk_yield?.toFixed(1) ?? '—'}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{g.max_milk_yield?.toFixed(1) ?? '—'}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{g.avg_milk_yield?.toFixed(1) ?? '—'}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{g.avg_weight?.toFixed(0) ?? '—'}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{g.number_of_cows ?? '—'}</td
				>
			</tr>
		{/each}
	</DataTable>
{/if}
