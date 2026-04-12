<script lang="ts">
	import { todayStr, daysAgoStr, monthStartStr } from '$lib/utils/datePresets';

	let {
		fromDate = $bindable(''),
		tillDate = $bindable(''),
		animalId = $bindable(''),
		showAnimal = true,
		onsearch,
	}: {
		fromDate?: string;
		tillDate?: string;
		animalId?: string;
		showAnimal?: boolean;
		onsearch: () => void;
	} = $props();

	function presetRange(days: number) {
		fromDate = daysAgoStr(days);
		tillDate = todayStr();
		onsearch();
	}

	function presetThisMonth() {
		fromDate = monthStartStr();
		tillDate = todayStr();
		onsearch();
	}

	function presetToday() {
		const t = todayStr();
		fromDate = t;
		tillDate = t;
		onsearch();
	}

	function resetFilters() {
		fromDate = '';
		tillDate = '';
		animalId = '';
		onsearch();
	}

	let presetBase = 'px-2 py-1 text-xs rounded-md border cursor-pointer transition-colors';
	let presetDefault =
		'border-slate-200 dark:border-slate-600 text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-700';
	let presetActive =
		'bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 border-blue-200 dark:border-blue-800';

	function isPresetActive(from: string, till: string): boolean {
		return fromDate === from && tillDate === till;
	}
</script>

<div
	class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-6"
>
	<div class="flex flex-wrap gap-3 items-end">
		{#if showAnimal}
			<div>
				<label for="filter-animal-id" class="block text-xs text-slate-500 dark:text-slate-400 mb-1"
					>ID животного</label
				>
				<input
					id="filter-animal-id"
					type="number"
					bind:value={animalId}
					placeholder="Все"
					class="w-32 px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
				/>
			</div>
		{/if}
		<div>
			<label for="filter-from" class="block text-xs text-slate-500 dark:text-slate-400 mb-1"
				>С</label
			>
			<input
				id="filter-from"
				type="date"
				bind:value={fromDate}
				class="px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
			/>
		</div>
		<div>
			<label for="filter-till" class="block text-xs text-slate-500 dark:text-slate-400 mb-1"
				>По</label
			>
			<input
				id="filter-till"
				type="date"
				bind:value={tillDate}
				class="px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
			/>
		</div>
		<div class="flex flex-wrap gap-1.5 items-center">
			<button
				onclick={presetToday}
				class="{presetBase} {isPresetActive(todayStr(), todayStr()) ? presetActive : presetDefault}"
				>Сегодня</button
			>
			<button
				onclick={() => presetRange(7)}
				class="{presetBase} {isPresetActive(daysAgoStr(7), todayStr())
					? presetActive
					: presetDefault}">7 дней</button
			>
			<button
				onclick={() => presetRange(30)}
				class="{presetBase} {isPresetActive(daysAgoStr(30), todayStr())
					? presetActive
					: presetDefault}">30 дней</button
			>
			<button
				onclick={presetThisMonth}
				class="{presetBase} {isPresetActive(monthStartStr(), todayStr())
					? presetActive
					: presetDefault}">Этот месяц</button
			>
		</div>
		<button
			onclick={onsearch}
			class="px-4 py-2 bg-slate-100 dark:bg-slate-900 hover:bg-slate-200 dark:bg-slate-700 text-slate-700 dark:text-slate-300 text-sm rounded-lg transition-colors cursor-pointer"
			>Найти</button
		>
		<button
			onclick={resetFilters}
			class="px-4 py-2 text-sm text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300 transition-colors cursor-pointer"
			>Сбросить</button
		>
	</div>
</div>
