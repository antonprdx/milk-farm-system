<script lang="ts">
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
</script>

<div
	class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-6"
>
	<div class="flex flex-wrap gap-3 items-end">
		{#if showAnimal}
			<div>
				<label class="block text-xs text-slate-500 dark:text-slate-400 mb-1">ID животного</label>
				<input
					type="number"
					bind:value={animalId}
					placeholder="Все"
					class="w-32 px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
				/>
			</div>
		{/if}
		<div>
			<label class="block text-xs text-slate-500 dark:text-slate-400 mb-1">С</label>
			<input
				type="date"
				bind:value={fromDate}
				class="px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
			/>
		</div>
		<div>
			<label class="block text-xs text-slate-500 dark:text-slate-400 mb-1">По</label>
			<input
				type="date"
				bind:value={tillDate}
				class="px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
			/>
		</div>
		<button
			onclick={onsearch}
			class="px-4 py-2 bg-slate-100 dark:bg-slate-900 hover:bg-slate-200 dark:bg-slate-700 text-slate-700 dark:text-slate-300 text-sm rounded-lg transition-colors cursor-pointer"
			>Найти</button
		>
	</div>
</div>
