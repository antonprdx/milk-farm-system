<script lang="ts">
	import { listAnimals, type Animal } from '$lib/api/animals';
	import { debounce } from '$lib/utils/debounce';

	let {
		value = $bindable<number | undefined>(undefined),
		error = '',
		required = false,
		id = '',
		label = 'Животное',
		disabled = false,
	}: {
		value?: number | undefined;
		error?: string;
		required?: boolean;
		id?: string;
		label?: string;
		disabled?: boolean;
	} = $props();

	let query = $state('');
	let results = $state<Animal[]>([]);
	let open = $state(false);
	let loading = $state(false);
	let selectedAnimal = $state<Animal | null>(null);

	let containerRef = $state<HTMLElement | null>(null);
	let inputRef = $state<HTMLInputElement | null>(null);
	let highlightIndex = $state(-1);

	const baseClass =
		'w-full px-3 py-2 border rounded-lg text-sm focus:ring-2 focus:border-blue-500 outline-none transition-colors bg-white dark:bg-slate-900 text-slate-800 dark:text-slate-200';
	let inputClass = $derived(
		error
			? baseClass + ' border-red-300 dark:border-red-500 focus:ring-red-500'
			: baseClass + ' border-slate-300 dark:border-slate-600 focus:ring-blue-500',
	);

	function formatAnimal(animal: Animal): string {
		const number = animal.life_number || animal.user_number;
		const gender = animal.gender === 'female' ? 'Корова' : 'Бык';
		return `${number} — ${animal.name ?? ''} (${gender})`;
	}

	const debouncedSearch = debounce(async (q: string) => {
		loading = true;
		try {
			const res = await listAnimals({ search: q, per_page: 10 });
			results = res.data;
			open = results.length > 0;
			highlightIndex = -1;
		} catch {
			results = [];
		} finally {
			loading = false;
		}
	}, 300);

	function search(q: string) {
		if (q.length < 1) {
			results = [];
			open = false;
			return;
		}
		debouncedSearch(q);
	}

	function selectAnimal(animal: Animal) {
		selectedAnimal = animal;
		value = animal.id;
		query = '';
		open = false;
	}

	function clearSelection() {
		selectedAnimal = null;
		value = undefined;
		query = '';
		open = false;
	}

	function onInput(e: Event) {
		const target = e.target as HTMLInputElement;
		query = target.value;
		if (query.length === 0 && selectedAnimal) {
			clearSelection();
			return;
		}
		search(query);
	}

	function onFocus() {
		if (selectedAnimal) return;
		if (query.length >= 1 && results.length > 0) {
			open = true;
		}
	}

	function onKeyDown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			open = false;
			return;
		}
		if (!open || results.length === 0) return;
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			highlightIndex = Math.min(highlightIndex + 1, results.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			highlightIndex = Math.max(highlightIndex - 1, 0);
		} else if (e.key === 'Enter' && highlightIndex >= 0) {
			e.preventDefault();
			selectAnimal(results[highlightIndex]);
		}
	}

	$effect(() => {
		if (value && !selectedAnimal) {
			query = String(value);
		}
	});

	function handleClickOutside(e: MouseEvent) {
		if (containerRef && !containerRef.contains(e.target as Node)) {
			open = false;
		}
	}

	$effect(() => {
		document.addEventListener('mousedown', handleClickOutside);
		return () => {
			document.removeEventListener('mousedown', handleClickOutside);
		};
	});
</script>

<div bind:this={containerRef} class="relative">
	<label for={id} class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
		>{label}{required ? ' *' : ''}</label
	>
	<div class="relative">
		<input
			bind:this={inputRef}
			{id}
			type="text"
			{disabled}
			class={inputClass + (selectedAnimal ? ' pr-8' : '')}
			placeholder={selectedAnimal ? '' : 'Поиск по имени или номеру...'}
			value={selectedAnimal ? formatAnimal(selectedAnimal) : query}
			oninput={onInput}
			onfocus={onFocus}
			onkeydown={onKeyDown}
			autocomplete="off"
		/>
		{#if selectedAnimal && !disabled}
			<button
				type="button"
				class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-300"
				onclick={clearSelection}
				aria-label="Очистить"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-4 w-4"
					viewBox="0 0 20 20"
					fill="currentColor"
				>
					<path
						fill-rule="evenodd"
						d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
						clip-rule="evenodd"
					/>
				</svg>
			</button>
		{/if}
	</div>
	{#if open && !selectedAnimal}
		<ul
			class="absolute z-50 mt-1 w-full max-h-60 overflow-auto rounded-lg border border-slate-300 dark:border-slate-600 bg-white dark:bg-slate-900 shadow-lg"
		>
			{#if loading}
				<li class="px-3 py-2 text-sm text-slate-500 dark:text-slate-400">Загрузка...</li>
			{:else if results.length === 0}
				<li class="px-3 py-2 text-sm text-slate-500 dark:text-slate-400">Ничего не найдено</li>
			{:else}
				{#each results as animal, i (animal.id)}
					<li>
						<button
							type="button"
							class="w-full text-left px-3 py-2 text-sm text-slate-800 dark:text-slate-200 hover:bg-blue-50 dark:hover:bg-slate-800 transition-colors {i ===
							highlightIndex
								? 'bg-blue-50 dark:bg-slate-800'
								: ''}"
							onclick={() => selectAnimal(animal)}
						>
							{formatAnimal(animal)}
						</button>
					</li>
				{/each}
			{/if}
		</ul>
	{/if}
	{#if error}
		<p class="mt-1 text-xs text-red-500 dark:text-red-400">{error}</p>
	{/if}
</div>
