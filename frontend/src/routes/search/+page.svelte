<script lang="ts">
	import { globalSearch, type SearchResult } from '$lib/api/tasks';

	let query = $state('');
	let results = $state<SearchResult | null>(null);
	let loading = $state(false);
	let searched = $state(false);

	async function search() {
		const q = query.trim();
		if (!q) return;
		loading = true;
		try {
			results = await globalSearch(q);
			searched = true;
		} catch (e) {
			results = null;
		} finally {
			loading = false;
		}
	}

	function handleKey(e: KeyboardEvent) {
		if (e.key === 'Enter') search();
	}
</script>

<svelte:head>
	<title>Поиск — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-4">Поиск</h1>

<div class="flex gap-3 mb-6">
	<input
		type="text"
		bind:value={query}
		onkeydown={handleKey}
		placeholder="Имя, номер жизни, компания..."
		class="flex-1 max-w-md px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
	/>
	<button
		onclick={search}
		disabled={loading || !query.trim()}
		class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50 cursor-pointer"
	>
		{loading ? 'Поиск...' : 'Найти'}
	</button>
</div>

{#if results}
	{#if results.animals.length > 0}
		<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-3">Животные ({results.animals.length})</h2>
		<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 mb-6">
			{#each results.animals as a (a.id)}
				<a
					href="/animals/{a.id}"
					class="block bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4 hover:border-blue-300 dark:hover:border-blue-600 transition-colors"
				>
					<div class="font-medium text-slate-800 dark:text-slate-200">{a.name ?? `#${a.id}`}</div>
					<div class="text-sm text-slate-500 dark:text-slate-400">ID: {a.id}{a.life_number ? ` · ${a.life_number}` : ''}</div>
				</a>
			{/each}
		</div>
	{/if}

	{#if results.contacts.length > 0}
		<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-3">Контакты ({results.contacts.length})</h2>
		<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 mb-6">
			{#each results.contacts as c (c.id)}
				<a
					href="/contacts"
					class="block bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4 hover:border-blue-300 dark:hover:border-blue-600 transition-colors"
				>
					<div class="font-medium text-slate-800 dark:text-slate-200">{c.name}</div>
					<div class="text-sm text-slate-500 dark:text-slate-400">{c.company_name ?? '—'}</div>
				</a>
			{/each}
		</div>
	{/if}

	{#if results.animals.length === 0 && results.contacts.length === 0}
		<div class="text-sm text-slate-500 dark:text-slate-400 py-8 text-center">Ничего не найдено</div>
	{/if}
{:else if searched}
	<div class="text-sm text-slate-500 dark:text-slate-400 py-8 text-center">Ничего не найдено</div>
{/if}
