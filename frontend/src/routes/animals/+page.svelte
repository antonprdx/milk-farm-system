<script lang="ts">
	import { listAnimals, deleteAnimal, batchDeactivateAnimals, batchUpdateAnimals, importAnimalsCsv, type Animal } from '$lib/api/animals';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { toasts } from '$lib/stores/toast';
	import { Pencil, Trash2, LayoutGrid, List } from 'lucide-svelte';

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

	let showBatchUpdate = $state(false);
	let batchUpdateLocation = $state('');
	let batchUpdateGroup = $state('');
	let batchUpdateLoading = $state(false);

	let viewMode = $state<'table' | 'cards'>('table');

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

	async function handleBatchUpdate() {
		try {
			batchUpdateLoading = true;
			const data: { location?: string; group_number?: number } = {};
			if (batchUpdateLocation) data.location = batchUpdateLocation;
			if (batchUpdateGroup) data.group_number = parseInt(batchUpdateGroup);
			await batchUpdateAnimals([...selectedIds], data);
			selectedIds = new Set();
			showBatchUpdate = false;
			batchUpdateLocation = '';
			batchUpdateGroup = '';
			toasts.success('Животные обновлены');
			load();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка';
		} finally {
			batchUpdateLoading = false;
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
				onclick={() => { showBatchUpdate = true; batchUpdateLocation = ''; batchUpdateGroup = ''; }}
				class="px-3 py-2 bg-blue-50 dark:bg-blue-900/30 hover:bg-blue-100 dark:hover:bg-blue-900/50 text-blue-600 dark:text-blue-400 text-sm font-medium rounded-lg transition-colors cursor-pointer"
			>Переместить</button>
			<button
				onclick={() => (showBatchDelete = true)}
				class="px-3 py-2 bg-red-50 dark:bg-red-900/30 hover:bg-red-100 dark:hover:bg-red-900/50 text-red-600 dark:text-red-400 text-sm font-medium rounded-lg transition-colors cursor-pointer"
			>Деактивировать</button>
		{/if}
		<button
			onclick={() => { showImport = true; importCsv = ''; importResult = null; }}
			class="px-4 py-2 bg-slate-100 dark:bg-slate-700 hover:bg-slate-200 dark:hover:bg-slate-600 text-slate-700 dark:text-slate-300 text-sm font-medium rounded-lg transition-colors cursor-pointer"
		>Импорт CSV</button>
		<div class="flex border border-slate-300 dark:border-slate-600 rounded-lg overflow-hidden">
			<button
				onclick={() => (viewMode = 'table')}
				class="px-2.5 py-2 text-sm transition-colors cursor-pointer {viewMode === 'table' ? 'bg-blue-600 text-white' : 'bg-white dark:bg-slate-800 text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700'}"
			><List size={16} /></button>
			<button
				onclick={() => (viewMode = 'cards')}
				class="px-2.5 py-2 text-sm transition-colors cursor-pointer {viewMode === 'cards' ? 'bg-blue-600 text-white' : 'bg-white dark:bg-slate-800 text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700'}"
			><LayoutGrid size={16} /></button>
		</div>
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

{#if viewMode === 'table'}
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
{:else}
	{#if loading}
		<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
			{#each Array(8) as _, i (i)}
				<div class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-5 animate-pulse">
					<div class="h-5 bg-slate-200 dark:bg-slate-700 rounded w-2/3 mb-3"></div>
					<div class="h-3 bg-slate-200 dark:bg-slate-700 rounded w-1/2 mb-2"></div>
					<div class="h-3 bg-slate-200 dark:bg-slate-700 rounded w-1/3 mb-4"></div>
					<div class="flex gap-2">
						<div class="h-6 bg-slate-200 dark:bg-slate-700 rounded w-16"></div>
						<div class="h-6 bg-slate-200 dark:bg-slate-700 rounded w-16"></div>
					</div>
				</div>
			{/each}
		</div>
	{:else if animals.length === 0}
		<div class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-8 text-center text-slate-400">
			Животные не найдены
		</div>
	{:else}
		<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
			{#each animals as animal (animal.id)}
				{@const cardUrl = `/animals/${animal.id}`}
				<div
					class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-5 hover:shadow-md hover:border-blue-200 dark:hover:border-blue-800 transition-all group"
				>
					<div class="flex items-start justify-between mb-3">
						<a
							href={cardUrl}
							class="block"
						>
							<h3 class="font-semibold text-slate-800 dark:text-slate-100 group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors">
								{animal.name || 'Без имени'}
							</h3>
							<p class="text-xs text-slate-400 font-mono mt-0.5">
								#{animal.life_number || animal.user_number || animal.id}
							</p>
						</a>
						<div class="flex items-center gap-1.5">
							<input
								type="checkbox"
								checked={selectedIds.has(animal.id)}
								onclick={(e) => { e.stopPropagation(); toggleSelect(animal.id); }}
								class="rounded border-slate-300 dark:border-slate-600 text-blue-600 focus:ring-blue-500 cursor-pointer"
							/>
						</div>
					</div>

					<div class="flex flex-wrap gap-1.5 mb-3">
						<span
							class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {animal.gender === 'female'
								? 'bg-pink-100 dark:bg-pink-900/50 text-pink-700'
								: 'bg-blue-100 dark:bg-blue-900/50 text-blue-700'}"
						>
							{animal.gender === 'female' ? 'Корова' : 'Бык'}
						</span>
						<span
							class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {animal.active
								? 'bg-green-100 dark:bg-green-900/50 text-green-700'
								: 'bg-slate-100 dark:bg-slate-900 text-slate-500 dark:text-slate-400'}"
						>
							{animal.active ? 'Активно' : 'Неактивно'}
						</span>
					</div>

					<div class="space-y-1.5 text-sm text-slate-500 dark:text-slate-400">
						{#if animal.birth_date}
							<p>Рожд. {animal.birth_date}</p>
						{/if}
						{#if animal.location}
							<p>Локация: {animal.location}</p>
						{/if}
						{#if animal.group_number != null}
							<p>Группа: {animal.group_number}</p>
						{/if}
					</div>

					<div class="flex gap-2 mt-3 pt-3 border-t border-slate-100 dark:border-slate-700">
						<a
							href="/animals/{animal.id}/edit"
							class="px-2 py-1 text-xs text-slate-500 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/50 rounded transition-colors cursor-pointer"
						>
							<Pencil size={13} />
						</a>
						<button
							onclick={() => (deleteId = animal.id)}
							class="px-2 py-1 text-xs text-slate-500 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/50 rounded transition-colors cursor-pointer"
						>
							<Trash2 size={13} />
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
{/if}

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

<Modal
	open={showBatchUpdate}
	title="Переместить выбранных животных"
	onclose={() => (showBatchUpdate = false)}
>
	<p class="text-sm text-slate-600 dark:text-slate-400 mb-4">
		Выбрано {selectedIds.size} животных. Оставьте поле пустым, чтобы не менять.
	</p>
	<div class="space-y-3">
		<div>
			<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">Локация</label>
			<input
				type="text"
				bind:value={batchUpdateLocation}
				placeholder="Новая локация"
				class="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-white dark:bg-slate-900 text-slate-800 dark:text-slate-200 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
			/>
		</div>
		<div>
			<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">Группа</label>
			<input
				type="number"
				bind:value={batchUpdateGroup}
				placeholder="Номер группы"
				class="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-white dark:bg-slate-900 text-slate-800 dark:text-slate-200 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
			/>
		</div>
	</div>
	<div class="flex gap-3 justify-end pt-4">
		<button
			type="button"
			onclick={() => (showBatchUpdate = false)}
			class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-800/50 cursor-pointer"
		>Отмена</button>
		<button
			onclick={handleBatchUpdate}
			disabled={batchUpdateLoading || (!batchUpdateLocation && !batchUpdateGroup)}
			class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50 cursor-pointer"
		>
			{batchUpdateLoading ? 'Обновление...' : 'Обновить'}
		</button>
	</div>
</Modal>
