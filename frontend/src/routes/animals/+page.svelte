<script lang="ts">
	import { listAnimals, deleteAnimal, batchDeactivateAnimals, importAnimalsCsv, type Animal } from '$lib/api/animals';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { toasts } from '$lib/stores/toast';
	import { Pencil, Trash2 } from 'lucide-svelte';

	let { data } = $props();

	let animals = $state<Animal[]>([]);
	let total = $state(0);
	let loading = $state(true);
	let error = $state('');

	let page = $state(1);
	let perPage = $state(20);
	let search = $state('');
	let genderFilter = $state<string>('');
	let activeFilter = $state<string>('true');

	let deleteId = $state<number | null>(null);
	let deleteLoading = $state(false);

	let selectedIds = $state<Set<number>>(new Set());
	let showBatchDelete = $state(false);
	let batchDeleteLoading = $state(false);

	function toggleSelect(id: number) {
		const s = new Set(selectedIds);
		if (s.has(id)) s.delete(id); else s.add(id);
		selectedIds = s;
	}

	function toggleSelectAll() {
		if (selectedIds.size === animals.length) {
			selectedIds = new Set();
		} else {
			selectedIds = new Set(animals.map((a) => a.id));
		}
	}

	async function handleBatchDeactivate() {
		try {
			batchDeleteLoading = true;
			await batchDeactivateAnimals([...selectedIds]);
			selectedIds = new Set();
			showBatchDelete = false;
			toasts.success('Животные деактивированы');
			load();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка';
		} finally {
			batchDeleteLoading = false;
		}
	}

	let showImport = $state(false);
	let importCsv = $state('');
	let importLoading = $state(false);
	let importResult = $state<{ created: number; errors: string[] } | null>(null);

	async function handleImport() {
		try {
			importLoading = true;
			importResult = null;
			const res = await importAnimalsCsv(importCsv);
			importResult = { created: res.created, errors: res.errors };
			if (res.created > 0) {
				toasts.success(`Импортировано ${res.created} животных`);
				load();
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка импорта';
		} finally {
			importLoading = false;
		}
	}

	let dataTable: DataTable | undefined = $state();

	let _skipLoad = !!data.initialData;

	if (data.error) error = data.error;

	if (data.initialData) {
		animals = data.initialData.data;
		total = data.initialData.total;
		loading = false;
	}

	async function load() {
		try {
			loading = true;
			error = '';
			const res = await listAnimals({
				search: search || undefined,
				gender: (genderFilter as 'male' | 'female') || undefined,
				active: activeFilter === '' ? undefined : activeFilter === 'true',
				page,
				per_page: perPage,
			});
			animals = res.data;
			total = res.total;
			dataTable?.setHasRows(animals.length > 0);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			loading = false;
			selectedIds = new Set();
		}
	}

	function applyFilters() {
		page = 1;
		load();
	}

	async function handleDelete() {
		if (deleteId === null) return;
		try {
			deleteLoading = true;
			await deleteAnimal(deleteId);
			deleteId = null;
			toasts.success('Животное удалено');
			load();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка удаления';
		} finally {
			deleteLoading = false;
		}
	}

	$effect(() => {
		page;
		if (_skipLoad) {
			_skipLoad = false;
			return;
		}
		load();
	});
</script>

<svelte:head>
	<title>Животные — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Животные</h1>
	<div class="flex gap-2 items-center">
		{#if selectedIds.size > 0}
			<span class="text-sm text-slate-500">Выбрано: {selectedIds.size}</span>
			<button
				onclick={() => (showBatchDelete = true)}
				class="px-3 py-2 bg-red-50 dark:bg-red-900/30 hover:bg-red-100 dark:hover:bg-red-900/50 text-red-600 dark:text-red-400 text-sm font-medium rounded-lg transition-colors cursor-pointer"
			>Деактивировать</button>
		{/if}
		<button
			onclick={() => { showImport = true; importCsv = ''; importResult = null; }}
			class="px-4 py-2 bg-slate-100 dark:bg-slate-700 hover:bg-slate-200 dark:hover:bg-slate-600 text-slate-700 dark:text-slate-300 text-sm font-medium rounded-lg transition-colors cursor-pointer"
		>Импорт CSV</button>
		<a
			href="/animals/new"
			class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors"
		>
			+ Добавить
		</a>
	</div>
</div>

<div
	class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-6"
>
	<div class="flex flex-wrap gap-3 items-end">
		<div class="flex-1 min-w-[200px]">
			<label for="animal-search" class="block text-xs text-slate-500 dark:text-slate-400 mb-1"
				>Поиск</label
			>
			<input
				id="animal-search"
				type="text"
				bind:value={search}
				onkeydown={(e) => e.key === 'Enter' && applyFilters()}
				placeholder="Имя, номер жизни, UCN..."
				class="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-white dark:bg-slate-900 text-slate-800 dark:text-slate-200 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
			/>
		</div>
		<div>
			<label for="animal-gender" class="block text-xs text-slate-500 dark:text-slate-400 mb-1"
				>Пол</label
			>
			<select
				id="animal-gender"
				bind:value={genderFilter}
				onchange={applyFilters}
				class="px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
			>
				<option value="">Все</option>
				<option value="female">Корова</option>
				<option value="male">Бык</option>
			</select>
		</div>
		<div>
			<label for="animal-status" class="block text-xs text-slate-500 dark:text-slate-400 mb-1"
				>Статус</label
			>
			<select
				id="animal-status"
				bind:value={activeFilter}
				onchange={applyFilters}
				class="px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
			>
				<option value="">Все</option>
				<option value="true">Активные</option>
				<option value="false">Неактивные</option>
			</select>
		</div>
		<button
			onclick={applyFilters}
			class="px-4 py-2 bg-slate-100 dark:bg-slate-900 hover:bg-slate-200 dark:bg-slate-700 text-slate-700 dark:text-slate-300 text-sm rounded-lg transition-colors cursor-pointer"
		>
			Найти
		</button>
	</div>
</div>

<ErrorAlert message={error} />

<DataTable
	bind:this={dataTable}
	columns={[
		{ key: 'select', label: '' },
		{ key: 'life_number', label: '№' },
		{ key: 'name', label: 'Имя' },
		{ key: 'gender', label: 'Пол' },
		{ key: 'birth_date', label: 'Дата рождения' },
		{ key: 'location', label: 'Локация' },
		{ key: 'group_number', label: 'Группа' },
		{ key: 'active', label: 'Статус' },
		{ key: 'actions', label: 'Действия', align: 'right' },
	]}
	{loading}
	emptyText="Животные не найдены"
	initialRows={!!data.initialData && data.initialData.data.length > 0}
>
	{#each animals as animal (animal.id)}
		<tr
			class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors {selectedIds.has(animal.id) ? 'bg-blue-50/50 dark:bg-blue-900/10' : ''}"
		>
			<td class="px-4 py-3">
				<input
					type="checkbox"
					checked={selectedIds.has(animal.id)}
					onchange={() => toggleSelect(animal.id)}
					class="rounded border-slate-300 dark:border-slate-600 text-blue-600 focus:ring-blue-500 cursor-pointer"
				/>
			</td>
			<td class="px-4 py-3 font-mono text-slate-600 dark:text-slate-400"
				>{animal.life_number || animal.user_number || '—'}</td
			>
			<td class="px-4 py-3">
				<a
					href="/animals/{animal.id}"
					class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 font-medium"
					>{animal.name || 'Без имени'}</a
				>
			</td>
			<td class="px-4 py-3">
				<span
					class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {animal.gender ===
					'female'
						? 'bg-pink-100 dark:bg-pink-900/50 text-pink-700'
						: 'bg-blue-100 dark:bg-blue-900/50 text-blue-700'}"
				>
					{animal.gender === 'female' ? 'Корова' : 'Бык'}
				</span>
			</td>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{animal.birth_date}</td>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{animal.location || '—'}</td>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{animal.group_number ?? '—'}</td>
			<td class="px-4 py-3">
				<span
					class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {animal.active
						? 'bg-green-100 dark:bg-green-900/50 text-green-700'
						: 'bg-slate-100 dark:bg-slate-900 text-slate-500 dark:text-slate-400'}"
				>
					{animal.active ? 'Активно' : 'Неактивно'}
				</span>
			</td>
			<td class="px-4 py-3 text-right">
				<div class="flex gap-2 justify-end">
					<a
						href="/animals/{animal.id}/edit"
						aria-label="Редактировать"
						class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/50 rounded transition-colors"
						><Pencil size={14} /></a
					>
					<button
						onclick={() => (deleteId = animal.id)}
						aria-label="Удалить"
						class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-red-600 hover:bg-red-50 dark:bg-red-900/50 rounded transition-colors cursor-pointer"
						><Trash2 size={14} /></button
					>
				</div>
			</td>
		</tr>
	{/each}
</DataTable>

<Pagination bind:page {total} {perPage} />

<ConfirmDialog
	open={deleteId !== null}
	title="Удалить животное?"
	message="Это действие нельзя отменить."
	loading={deleteLoading}
	onconfirm={handleDelete}
	oncancel={() => (deleteId = null)}
/>

<ConfirmDialog
	open={showBatchDelete}
	title="Деактивировать выбранных?"
	message="Выбрано {selectedIds.size} животных. Они будут помечены как неактивные."
	confirmText="Деактивировать"
	loading={batchDeleteLoading}
	onconfirm={handleBatchDeactivate}
	oncancel={() => (showBatchDelete = false)}
/>

<Modal
	open={showImport}
	title="Импорт животных из CSV"
	onclose={() => (showImport = false)}
>
	<p class="text-sm text-slate-600 dark:text-slate-400 mb-2">
		Формат: <code class="text-xs bg-slate-100 dark:bg-slate-700 px-1 rounded">пол,дата_рождения,имя,номер_жизни</code>
	</p>
	<p class="text-xs text-slate-500 dark:text-slate-400 mb-4">
		Пол: male/female (или м/ж). Дата: YYYY-MM-DD. Имя и номер жизни — необязательны.
	</p>
	<textarea
		bind:value={importCsv}
		rows={8}
		placeholder="female,2020-03-15,Бурёнка,12345&#10;male,2021-06-01,Бычок,"
		class="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-white dark:bg-slate-900 text-slate-800 dark:text-slate-200 font-mono focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
	></textarea>
	{#if importResult}
		<div class="mt-3 p-3 rounded-lg {importResult.errors.length > 0 ? 'bg-yellow-50 dark:bg-yellow-900/20' : 'bg-green-50 dark:bg-green-900/20'}">
			<p class="text-sm font-medium">Создано: {importResult.created}</p>
			{#if importResult.errors.length > 0}
				<p class="text-sm text-red-600 dark:text-red-400 mt-1">Ошибки ({importResult.errors.length}):</p>
				<ul class="text-xs text-red-600 dark:text-red-400 list-disc ml-4 max-h-32 overflow-y-auto">
					{#each importResult.errors as err}
						<li>{err}</li>
					{/each}
				</ul>
			{/if}
		</div>
	{/if}
	<div class="flex gap-3 justify-end pt-4">
		<button
			type="button"
			onclick={() => (showImport = false)}
			class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 cursor-pointer"
		>Закрыть</button>
		<button
			onclick={handleImport}
			disabled={importLoading || !importCsv.trim()}
			class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50 cursor-pointer"
		>
			{importLoading ? 'Импорт...' : 'Импортировать'}
		</button>
	</div>
</Modal>
