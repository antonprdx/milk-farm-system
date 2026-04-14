<script lang="ts">
	import {
		listTransfers,
		createTransfer,
		updateTransfer,
		deleteTransfer,
		type Transfer,
		type CreateTransfer,
		type UpdateTransfer,
		TRANSFER_TYPE_LABELS,
	} from '$lib/api/transfers';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { useCrudModal } from '$lib/utils/useCrudModal.svelte';
	import { usePaginatedList } from '$lib/utils/usePaginatedList.svelte';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';
	import { formatDatetime } from '$lib/utils/format';
	import { Pencil, Trash2 } from 'lucide-svelte';

	let { data } = $props();

	let dataTable: DataTable;
	let items = $state<Transfer[]>([]);

	const list = usePaginatedList();
	const crud = useCrudModal();
	const v = useFormValidation();

	let _skipLoad = !!data.initialData;
	let _hasInitial = $state(!!data.initialData);

	if (data.initialData) {
		items = data.initialData.data;
	}

	let now = new Date().toISOString().slice(0, 16);

	let createForm = $state<CreateTransfer>({
		animal_id: 0,
		transfer_date: now,
		transfer_type: 'arrival',
	});
	let editForm = $state<UpdateTransfer>({});

	const animalIdRules = [rules.required(), rules.min(1, 'ID животного должен быть положительным')];
	const transferTypeRules = [rules.required()];
	const transferDateRules = [rules.required()];

	async function load() {
		await list.load(
			(signal) =>
				listTransfers(
					{
						animal_id: list.animalId || undefined,
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
		createForm = { animal_id: 0, transfer_date: now, transfer_type: 'arrival' };
		v.clear();
		crud.openCreate();
	}

	function openEdit(t: Transfer) {
		editForm = {
			transfer_date: t.transfer_date,
			transfer_type: t.transfer_type,
			reason_id: t.reason_id ?? undefined,
			from_location: t.from_location ?? undefined,
			to_location: t.to_location ?? undefined,
		};
		v.clear();
		crud.openEdit(t.id);
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		let valid: boolean;
		if (crud.modalMode === 'create') {
			valid = v.validateAll({
				animal_id: { value: createForm.animal_id, rules: animalIdRules },
				transfer_type: { value: createForm.transfer_type, rules: transferTypeRules },
				transfer_date: { value: createForm.transfer_date, rules: transferDateRules },
			});
		} else {
			valid = v.validateAll({
				transfer_type: { value: editForm.transfer_type, rules: transferTypeRules },
				transfer_date: { value: editForm.transfer_date, rules: transferDateRules },
			});
		}
		if (!valid) return;
		await crud.submit(
			() =>
				crud.modalMode === 'create'
					? createTransfer(createForm)
					: updateTransfer(crud.editingId, editForm),
			crud.modalMode === 'create' ? 'Запись создана' : 'Запись обновлена',
			load,
		);
	}

	async function handleDelete() {
		await crud.remove(
			() => deleteTransfer(crud.deleteId),
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
	<title>Перемещения — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Перемещения</h1>
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
		placeholder="ID животного..."
		bind:value={list.animalId}
		class="px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm bg-white dark:bg-slate-800 dark:text-slate-200 w-48"
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
		{ key: 'animal_id', label: 'ID животного' },
		{ key: 'transfer_date', label: 'Дата' },
		{ key: 'transfer_type', label: 'Тип' },
		{ key: 'from_location', label: 'Откуда' },
		{ key: 'to_location', label: 'Куда' },
		{ key: 'actions', label: '', align: 'right' },
	]}
	loading={list.loading && !_hasInitial}
	initialRows={!!data.initialData && data.initialData.data.length > 0}
	bind:this={dataTable}
	emptyText="Нет данных"
>
	{#each items as t (t.id)}
		<tr
			class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
		>
			<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{t.id}</td>
			<td class="px-4 py-3 font-medium">{t.animal_id}</td>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400"
				>{formatDatetime(t.transfer_date)}</td
			>
			<td class="px-4 py-3">
				<span
					class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
				>
					{TRANSFER_TYPE_LABELS[t.transfer_type] || t.transfer_type}
				</span>
			</td>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400"
				>{t.from_location ?? '—'}</td
			>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{t.to_location ?? '—'}</td>
			<td class="px-4 py-3 text-right">
				<button
					onclick={() => openEdit(t)}
					aria-label="Редактировать"
					class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
					><Pencil size={14} /></button
				>
				<button
					onclick={() => crud.confirmDelete(t.id)}
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
	title={crud.modalMode === 'create' ? 'Новое перемещение' : 'Редактировать перемещение'}
	onclose={crud.close}
>
	<ErrorAlert message={crud.modalError} />
	<form onsubmit={handleSubmit} class="space-y-4">
		{#if crud.modalMode === 'create'}
			<FormField
				id="c-animal-id"
				label="ID животного"
				type="number"
				bind:value={createForm.animal_id}
				required
				error={v.getError('animal_id')}
				onblur={() => v.validateField('animal_id', createForm.animal_id, animalIdRules)}
			/>
			<FormField
				id="c-transfer-date"
				label="Дата перемещения"
				type="datetime-local"
				bind:value={createForm.transfer_date}
				required
				error={v.getError('transfer_date')}
				onblur={() =>
					v.validateField('transfer_date', createForm.transfer_date, transferDateRules)}
			/>
			<FormField
				id="c-transfer-type"
				label="Тип"
				type="text"
				bind:value={createForm.transfer_type}
				required
				error={v.getError('transfer_type')}
				onblur={() =>
					v.validateField('transfer_type', createForm.transfer_type, transferTypeRules)}
			/>
			<FormField
				id="c-from"
				label="Откуда"
				type="text"
				bind:value={createForm.from_location}
			/>
			<FormField id="c-to" label="Куда" type="text" bind:value={createForm.to_location} />
		{:else}
			<FormField
				id="e-transfer-date"
				label="Дата перемещения"
				type="datetime-local"
				bind:value={editForm.transfer_date}
				required
				error={v.getError('transfer_date')}
				onblur={() =>
					v.validateField('transfer_date', editForm.transfer_date, transferDateRules)}
			/>
			<FormField
				id="e-transfer-type"
				label="Тип"
				type="text"
				bind:value={editForm.transfer_type}
				required
				error={v.getError('transfer_type')}
				onblur={() =>
					v.validateField('transfer_type', editForm.transfer_type, transferTypeRules)}
			/>
			<FormField
				id="e-from"
				label="Откуда"
				type="text"
				bind:value={editForm.from_location}
			/>
			<FormField id="e-to" label="Куда" type="text" bind:value={editForm.to_location} />
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
	title="Удалить перемещение?"
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
