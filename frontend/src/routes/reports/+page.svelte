<script lang="ts">
	import {
		getMilkSummary,
		getReproductionSummary,
		getFeedSummary,
		getExportUrl,
		type MilkSummary,
		type ReproductionSummary,
		type FeedSummary,
	} from '$lib/api/reports';
	import { onMount } from 'svelte';
	import FilterBar from '$lib/components/ui/FilterBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import { fmtNum } from '$lib/utils/format';

	let loading = $state(true);
	let error = $state('');
	let fromDate = $state('');
	let tillDate = $state('');
	let milk = $state<MilkSummary | null>(null);
	let repro = $state<ReproductionSummary | null>(null);
	let feed = $state<FeedSummary | null>(null);

	async function load() {
		try {
			loading = true;
			error = '';
			const from = fromDate || undefined;
			const till = tillDate || undefined;
			[milk, repro, feed] = await Promise.all([
				getMilkSummary(from, till),
				getReproductionSummary(from, till),
				getFeedSummary(from, till),
			]);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			loading = false;
		}
	}

	onMount(load);
</script>

<svelte:head>
	<title>Отчёты — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-6">Отчёты</h1>

<FilterBar bind:fromDate bind:tillDate showAnimal={false} onsearch={load} />
<ErrorAlert message={error} />

{#if loading}
	<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
		{#each Array(3) as _, i (i)}
			<div
				class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 space-y-3"
			>
				<div class="h-5 bg-slate-100 dark:bg-slate-900 rounded animate-pulse w-1/2"></div>
				<div class="h-8 bg-slate-100 dark:bg-slate-900 rounded animate-pulse w-3/4"></div>
				<div class="h-4 bg-slate-100 dark:bg-slate-900 rounded animate-pulse w-2/3"></div>
			</div>
		{/each}
	</div>
{:else}
	<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 space-y-4"
		>
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">Молоко</h2>
			<a
				href={getExportUrl('milk', fromDate || undefined, tillDate || undefined)}
				class="float-right text-xs px-2 py-1 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
				>CSV</a
			>
			{#if milk}
				<div class="space-y-2">
					<div class="flex justify-between text-sm">
						<span class="text-slate-500 dark:text-slate-400">Общий надой</span>
						<span class="font-medium">{fmtNum(milk.total_milk)} л</span>
					</div>
					<div class="flex justify-between text-sm">
						<span class="text-slate-500 dark:text-slate-400">Записей</span>
						<span class="font-medium">{milk.count_days}</span>
					</div>
					<div class="flex justify-between text-sm">
						<span class="text-slate-500 dark:text-slate-400">Среднее на животное</span>
						<span class="font-medium">{fmtNum(milk.avg_per_animal)} л</span>
					</div>
				</div>
			{:else}
				<p class="text-sm text-slate-400 dark:text-slate-500">Нет данных</p>
			{/if}
		</div>

		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 space-y-4"
		>
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">Воспроизводство</h2>
			<a
				href={getExportUrl('reproduction', fromDate || undefined, tillDate || undefined)}
				class="float-right text-xs px-2 py-1 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
				>CSV</a
			>
			{#if repro}
				<div class="space-y-2">
					<div class="flex justify-between text-sm">
						<span class="text-slate-500 dark:text-slate-400">Отёлы</span>
						<span class="font-medium">{repro.total_calvings}</span>
					</div>
					<div class="flex justify-between text-sm">
						<span class="text-slate-500 dark:text-slate-400">Осеменения</span>
						<span class="font-medium">{repro.total_inseminations}</span>
					</div>
					<div class="flex justify-between text-sm">
						<span class="text-slate-500 dark:text-slate-400">Стельности</span>
						<span class="font-medium">{repro.total_pregnancies}</span>
					</div>
					<div class="flex justify-between text-sm">
						<span class="text-slate-500 dark:text-slate-400">Охота</span>
						<span class="font-medium">{repro.total_heats}</span>
					</div>
					<div class="flex justify-between text-sm">
						<span class="text-slate-500 dark:text-slate-400">Запуски</span>
						<span class="font-medium">{repro.total_dry_offs}</span>
					</div>
				</div>
			{:else}
				<p class="text-sm text-slate-400 dark:text-slate-500">Нет данных</p>
			{/if}
		</div>

		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 space-y-4"
		>
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">Кормление</h2>
			<a
				href={getExportUrl('feed', fromDate || undefined, tillDate || undefined)}
				class="float-right text-xs px-2 py-1 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
				>CSV</a
			>
			{#if feed}
				<div class="space-y-2">
					<div class="flex justify-between text-sm">
						<span class="text-slate-500 dark:text-slate-400">Общий корм</span>
						<span class="font-medium">{fmtNum(feed.total_feed_kg)} кг</span>
					</div>
					<div class="flex justify-between text-sm">
						<span class="text-slate-500 dark:text-slate-400">Визитов</span>
						<span class="font-medium">{feed.total_visits}</span>
					</div>
				</div>
			{:else}
				<p class="text-sm text-slate-400 dark:text-slate-500">Нет данных</p>
			{/if}
		</div>
	</div>
{/if}
