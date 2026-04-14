<script lang="ts">
	import {
		listSires,
		createSire,
		updateSire,
		deleteSire,
		type Sire,
		type CreateSire,
		type UpdateSire,
	} from '$lib/api/sires';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { useCrudModal } from '$lib/utils/useCrudModal.svelte';
	import { usePaginatedList } from '$lib/utils/usePaginatedList.svelte';
	import { Pencil, Trash2 } from 'lucide-svelte';

	let { data } = $props();

	let dataTable: DataTable;
	let items = $state<Sire[]>([]);

	const list = usePaginatedList();
	const crud = useCrudModal();

	let _skipLoad = !!data.initialData;
	let _hasInitial = $state(!!data.initialData);

	if (data.initialData) {
		items = data.initialData.data;
	}

	let search = $state('');

	let createForm = $state<CreateSire>({});
	let editForm = $state<UpdateSire>({});

	async function load() {
		await list.load(
			(signal) =>
				listSires(
					{
						search: search || undefined,
						page: list.page,
						per_page: list.perPage,
					},
					signal,
				),
			(data) => {
				items = data;
			},
			dataTable,
		);
	}

	function openCreate() {
		createForm = {};
		crud.openCreate();
	}

	function openEdit(s: Sire) {
		editForm = {
			sire_code: s.sire_code ?? undefined,
			life_number: s.life_number ?? undefined,
			name: s.name ?? undefined,
			active: s.active ?? undefined,
		};
		crud.openEdit(s.id);
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		await crud.submit(
			() =>
				crud.modalMode === 'create'
					? createSire(createForm)
					: updateSire(crud.editingId, editForm),
			crud.modalMode === 'create' ? 'Запись создана' : 'Запись обновлена',
			load,
		);
	}

	async function handleDelete() {
		await crud.remove(
			() => deleteSire(crud.deleteId),
			load,
			(msg) => {
				list.error = msg;
			},
		);
	}

	$effect(() => {
		list.page;
		if (_skipLoad) {
			_skipLoad = false;
			return;
		}
		_hasInitial = false;
		load();
	});
</script>

<svelte:head>
	<title>Быки — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Быки</h1>
	<button
		onclick={openCreate}
		class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
	>
		+ Добавить
	</button>
</div>

<div class="flex gap-3 mb-4">
	<input
		type="text"
		placeholder="Поиск по коду, номеру, имени..."
		bind:value={search}
		class="flex-1 px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-white dark:bg-slate-800 dark:text-slate-200"
	/>
	<button
		onclick={() => {
			list.page = 1;
			load();
		}}
		class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-lg cursor-pointer"
	>
		Найти
	</button>
</div>

<ErrorAlert message={data.error} />
<ErrorAlert message={list.error} />

<DataTable
	columns={[
		{ key: 'id', label: 'ID' },
		{ key: 'sire_code', label: 'Код быка' },
		{ key: 'life_number', label: 'Жизненный номер' },
		{ key: 'name', label: 'Имя' },
		{ key: 'active', label: 'Активен' },
		{ key: 'actions', label: '', align: 'right' },
	]}
	loading={list.loading && !_hasInitial}
	initialRows={!!data.initialData && data.initialData.data.length > 0}
	bind:this={dataTable}
	emptyText="Нет данных"
>
	{#each items as s (s.id)}
		<tr
			class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
		>
			<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{s.id}</td>
			<td class="px-4 py-3 font-medium">{s.sire_code ?? '—'}</td>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{s.life_number ?? '—'}</td>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{s.name ?? '—'}</td>
			<td class="px-4 py-3">
				<span
					class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {s.active
						? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
						: 'bg-slate-100 text-slate-600 dark:bg-slate-700 dark:text-slate-400'}"
				>
					{s.active ? 'Да' : 'Нет'}
				</span>
			</td>
			<td class="px-4 py-3 text-right">
				<button
					onclick={() => openEdit(s)}
					aria-label="Редактировать"
					class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
					><Pencil size={14} /></button
				>
				<button
					onclick={() => crud.confirmDelete(s.id)}
					aria-label="Удалить"
					class="text-red-500 hover:text-red-700 text-sm cursor-pointer"
					><Trash2 size={14} /></button
				>
			</td>
		</tr>
	{/each}
</DataTable>

<Modal
	open={crud.showModal}
	title={crud.modalMode === 'create' ? 'Новый бык' : 'Редактировать быка'}
	onclose={crud.close}
>
	<ErrorAlert message={crud.modalError} />
	<form onsubmit={handleSubmit} class="space-y-4">
		{#if crud.modalMode === 'create'}
			<FormField id="c-sire-code" label="Код быка" type="text" bind:value={createForm.sire_code} />
			<FormField
				id="c-life-number"
				label="Жизненный номер"
				type="text"
				bind:value={createForm.life_number}
			/>
			<FormField id="c-name" label="Имя" type="text" bind:value={createForm.name} />
			<label class="flex items-center gap-2 text-sm">
				<input type="checkbox" bind:checked={createForm.active} class="rounded" />
				Активен
			</label>
		{:else}
			<FormField id="e-sire-code" label="Код быка" type="text" bind:value={editForm.sire_code} />
			<FormField
				id="e-life-number"
				label="Жизненный номер"
				type="text"
				bind:value={editForm.life_number}
			/>
			<FormField id="e-name" label="Имя" type="text" bind:value={editForm.name} />
			<label class="flex items-center gap-2 text-sm">
				<input type="checkbox" bind:checked={editForm.active} class="rounded" />
				Активен
			</label>
		{/if}
		<div class="flex gap-3 justify-end pt-2">
			<button
				type="button"
				onclick={crud.close}
				class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 cursor-pointer"
				>Отмена</button
			>
			<button
				type="submit"
				disabled={crud.modalLoading}
				class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50 cursor-pointer"
			>
				{crud.modalLoading
					? 'Сохранение...'
					: crud.modalMode === 'create'
						? 'Создать'
						: 'Сохранить'}
			</button>
		</div>
	</form>
</Modal>

<ConfirmDialog
	open={crud.showDelete}
	title="Удалить быка?"
	message="Это действие нельзя отменить."
	loading={crud.deleteLoading}
	onconfirm={handleDelete}
	oncancel={crud.closeDelete}
/>

<Pagination
	bind:page={list.page}
	total={_hasInitial && data.initialData ? data.initialData.total : list.total}
	perPage={list.perPage}
/>
