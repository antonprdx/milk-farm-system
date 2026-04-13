<script lang="ts">
	import { type MilkSummary, type ReproductionSummary, type FeedSummary, getExportUrl } from '$lib/api/reports';
	import { fmtNum } from '$lib/utils/format';

	let { milk, repro, feed, fromDate = '', tillDate = '' }: {
		milk: MilkSummary | null;
		repro: ReproductionSummary | null;
		feed: FeedSummary | null;
		fromDate?: string;
		tillDate?: string;
	} = $props();
</script>

<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
	<div
		class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 space-y-4"
	>
		<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300">Молоко</h2>
		<div class="float-right flex gap-1">
			<a
				href={getExportUrl('milk', fromDate || undefined, tillDate || undefined, 'pdf')}
				class="text-xs px-2 py-1 bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors"
				>PDF</a
			>
			<a
				href={getExportUrl('milk', fromDate || undefined, tillDate || undefined)}
				class="text-xs px-2 py-1 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
				>CSV</a
			>
		</div>
		{#if milk}
			<div class="space-y-2">
				<div class="flex justify-between text-sm">
					<span class="text-slate-500 dark:text-slate-400">Общий надой</span><span
						class="font-medium">{fmtNum(milk.total_milk)} л</span
					>
				</div>
				<div class="flex justify-between text-sm">
					<span class="text-slate-500 dark:text-slate-400">Записей</span><span
						class="font-medium">{milk.count_days}</span
					>
				</div>
				<div class="flex justify-between text-sm">
					<span class="text-slate-500 dark:text-slate-400">Среднее на животное</span><span
						class="font-medium">{fmtNum(milk.avg_per_animal)} л</span
					>
				</div>
			</div>
		{/if}
	</div>
	<div
		class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 space-y-4"
	>
		<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300">Воспроизводство</h2>
		<div class="float-right flex gap-1">
			<a
				href={getExportUrl('reproduction', fromDate || undefined, tillDate || undefined, 'pdf')}
				class="text-xs px-2 py-1 bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors"
				>PDF</a
			>
			<a
				href={getExportUrl('reproduction', fromDate || undefined, tillDate || undefined)}
				class="text-xs px-2 py-1 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
				>CSV</a
			>
		</div>
		{#if repro}
			<div class="space-y-2">
				<div class="flex justify-between text-sm">
					<span class="text-slate-500 dark:text-slate-400">Отёлы</span><span class="font-medium"
						>{repro.total_calvings}</span
					>
				</div>
				<div class="flex justify-between text-sm">
					<span class="text-slate-500 dark:text-slate-400">Осеменения</span><span
						class="font-medium">{repro.total_inseminations}</span
					>
				</div>
				<div class="flex justify-between text-sm">
					<span class="text-slate-500 dark:text-slate-400">Стельности</span><span
						class="font-medium">{repro.total_pregnancies}</span
					>
				</div>
				<div class="flex justify-between text-sm">
					<span class="text-slate-500 dark:text-slate-400">Охота</span><span class="font-medium"
						>{repro.total_heats}</span
					>
				</div>
				<div class="flex justify-between text-sm">
					<span class="text-slate-500 dark:text-slate-400">Запуски</span><span
						class="font-medium">{repro.total_dry_offs}</span
					>
				</div>
			</div>
		{/if}
	</div>
	<div
		class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 space-y-4"
	>
		<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300">Кормление</h2>
		<div class="float-right flex gap-1">
			<a
				href={getExportUrl('feed', fromDate || undefined, tillDate || undefined, 'pdf')}
				class="text-xs px-2 py-1 bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors"
				>PDF</a
			>
			<a
				href={getExportUrl('feed', fromDate || undefined, tillDate || undefined)}
				class="text-xs px-2 py-1 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
				>CSV</a
			>
		</div>
		{#if feed}
			<div class="space-y-2">
				<div class="flex justify-between text-sm">
					<span class="text-slate-500 dark:text-slate-400">Общий корм</span><span
						class="font-medium">{fmtNum(feed.total_feed_kg)} кг</span
					>
				</div>
				<div class="flex justify-between text-sm">
					<span class="text-slate-500 dark:text-slate-400">Визитов</span><span
						class="font-medium">{feed.total_visits}</span
					>
				</div>
			</div>
		{/if}
	</div>
</div>
