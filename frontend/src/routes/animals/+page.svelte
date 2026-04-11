<script lang="ts">
	import { listAnimals, deleteAnimal, type Animal } from '$lib/api/animals';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { toasts } from '$lib/stores/toast';
	import { Pencil, Trash2 } from 'lucide-svelte';

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

	let dataTable: DataTable | undefined = $state();

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
		load();
	});
</script>

<svelte:head>
	<title>Животные — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Животные</h1>
	<a
		href="/animals/new"
		class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors"
	>
		+ Добавить
	</a>
</div>

<div
	class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-6"
>
	<div class="flex flex-wrap gap-3 items-end">
		<div class="flex-1 min-w-[200px]">
			<label for="animal-search" class="block text-xs text-slate-500 dark:text-slate-400 mb-1">Поиск</label>
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
			<label for="animal-gender" class="block text-xs text-slate-500 dark:text-slate-400 mb-1">Пол</label>
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
			<label for="animal-status" class="block text-xs text-slate-500 dark:text-slate-400 mb-1">Статус</label>
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
>
	{#each animals as animal (animal.id)}
		<tr
			class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
		>
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
						class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/50 rounded transition-colors"
						><Pencil size={14} /></a
					>
					<button
						onclick={() => (deleteId = animal.id)}
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
