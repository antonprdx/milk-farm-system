<script lang="ts">
	let {
		page = $bindable(1),
		total = 0,
		perPage = 20,
	}: {
		page?: number;
		total?: number;
		perPage?: number;
	} = $props();

	let totalPages = $derived(Math.max(1, Math.ceil(total / perPage)));
</script>

{#if total > 0}
	<div
		class="flex items-center justify-between px-4 py-3 border-t border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/50"
	>
		<span class="text-sm text-slate-500 dark:text-slate-400">Всего: {total}</span>
		<div class="flex gap-1">
			<button
				onclick={() => page--}
				disabled={page <= 1}
				class="px-3 py-1 text-sm border border-slate-300 dark:border-slate-600 rounded hover:bg-slate-100 dark:hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
				>Назад</button
			>
			<span class="px-3 py-1 text-sm text-slate-600 dark:text-slate-400">{page} / {totalPages}</span
			>
			<button
				onclick={() => page++}
				disabled={page >= totalPages}
				class="px-3 py-1 text-sm border border-slate-300 dark:border-slate-600 rounded hover:bg-slate-100 dark:hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
				>Вперёд</button
			>
		</div>
	</div>
{/if}
