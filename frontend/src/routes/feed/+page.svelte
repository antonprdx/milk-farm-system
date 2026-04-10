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
	import { onMount } from 'svelte';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import FilterBar from '$lib/components/ui/FilterBar.svelte';
	import TabBar from '$lib/components/ui/TabBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { formatDatetime } from '$lib/utils/format';

	type Tab = 'amounts' | 'visits' | 'types' | 'groups';

	let tab = $state<Tab>('amounts');
	let loading = $state(true);
	let error = $state('');

	let amounts = $state<FeedDayAmount[]>([]);
	let visits = $state<FeedVisit[]>([]);
	let types = $state<FeedType[]>([]);
	let groups = $state<FeedGroup[]>([]);

	let fromDate = $state('');
	let tillDate = $state('');
	let animalId = $state('');

	let page = $state(1);
	let total = $state(0);
	let initialized = $state(false);

	function getFilter() {
		return {
			animal_id: animalId ? Number(animalId) : undefined,
			from_date: fromDate || undefined,
			till_date: tillDate || undefined,
			page,
			per_page: 50,
		};
	}

	async function load() {
		try {
			loading = true;
			error = '';
			switch (tab) {
				case 'amounts': {
					const res = await listDayAmounts(getFilter());
					amounts = res.data;
					total = res.total;
					break;
				}
				case 'visits': {
					const res = await listVisits(getFilter());
					visits = res.data;
					total = res.total;
					break;
				}
				case 'types':
					types = (await listTypes()).data;
					break;
				case 'groups':
					groups = (await listGroups()).data;
					break;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			loading = false;
		}
	}

	function switchTab(t: Tab) {
		tab = t;
		page = 1;
		load();
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
	<FilterBar bind:fromDate bind:tillDate bind:animalId onsearch={load} />
{/if}

<ErrorAlert message={error} />

{#if tab === 'amounts'}
	<DataTable
		columns={[
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'feed_date', label: 'Дата' },
			{ key: 'feed_number', label: '№ корма', align: 'right' },
			{ key: 'total', label: 'Всего, кг', align: 'right' },
			{ key: 'rest_feed', label: 'Остаток', align: 'right' },
		]}
		{loading}
	>
		{#if amounts.length === 0}
			<tr
				><td colspan="5" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
					>Нет данных</td
				></tr
			>
		{:else}
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
		{/if}
	</DataTable>
	<Pagination bind:page {total} perPage={50} />
{:else if tab === 'visits'}
	<DataTable
		columns={[
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'visit_datetime', label: 'Время' },
			{ key: 'feed_number', label: '№ корма', align: 'right' },
			{ key: 'amount', label: 'Кол-во, кг', align: 'right' },
			{ key: 'duration_seconds', label: 'Длительность, с', align: 'right' },
		]}
		{loading}
	>
		{#if visits.length === 0}
			<tr
				><td colspan="5" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
					>Нет данных</td
				></tr
			>
		{:else}
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
		{/if}
	</DataTable>
	<Pagination bind:page {total} perPage={50} />
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
		{loading}
	>
		{#if types.length === 0}
			<tr
				><td colspan="6" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
					>Нет данных</td
				></tr
			>
		{:else}
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
		{/if}
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
		{loading}
	>
		{#if groups.length === 0}
			<tr
				><td colspan="6" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
					>Нет данных</td
				></tr
			>
		{:else}
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
		{/if}
	</DataTable>
{/if}
