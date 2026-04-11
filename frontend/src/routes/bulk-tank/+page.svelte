<script lang="ts">
	import {
		listBulkTankTests,
		createBulkTankTest,
		updateBulkTankTest,
		deleteBulkTankTest,
		type BulkTankTest,
		type CreateBulkTankTest,
		type UpdateBulkTankTest,
	} from '$lib/api/bulk_tank';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import FilterBar from '$lib/components/ui/FilterBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import { fmtNum } from '$lib/utils/format';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { useCrudModal } from '$lib/utils/useCrudModal.svelte';
	import { usePaginatedList } from '$lib/utils/usePaginatedList.svelte';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';
	import { Pencil, Trash2 } from 'lucide-svelte';

	let { data } = $props();

	let dataTable: DataTable;
	let tests = $state<BulkTankTest[]>([]);

	const list = usePaginatedList();
	const crud = useCrudModal();
	const v = useFormValidation();

	let _skipLoad = !!data.initialData;
	let _hasInitial = $state(!!data.initialData);

	if (data.initialData) {
		tests = data.initialData.data;
	}

	const dateRules = [rules.required()];
	const fatRules = [rules.required(), rules.percentage()];
	const proteinRules = [rules.required(), rules.percentage()];

	let today = new Date().toISOString().slice(0, 10);

	let createForm = $state<CreateBulkTankTest>({ date: today, fat: 3.8, protein: 3.2 });
	let editForm = $state<UpdateBulkTankTest>({});

	async function load() {
		await list.load(
			(signal) =>
				listBulkTankTests({
					from_date: list.fromDate || undefined,
					till_date: list.tillDate || undefined,
					page: list.page,
					per_page: list.perPage,
				}, signal),
			(data) => {
				tests = data;
			},
			dataTable,
		);
	}

	function openCreate() {
		createForm = { date: today, fat: 3.8, protein: 3.2 };
		v.clear();
		crud.openCreate();
	}

	function openEdit(t: BulkTankTest) {
		editForm = {
			date: t.date,
			fat: t.fat,
			protein: t.protein,
			lactose: t.lactose ?? undefined,
			scc: t.scc ?? undefined,
			ffa: t.ffa ?? undefined,
		};
		v.clear();
		crud.openEdit(t.id);
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		let valid: boolean;
		if (crud.modalMode === 'create') {
			valid = v.validateAll({
				date: { value: createForm.date, rules: dateRules },
				fat: { value: createForm.fat, rules: fatRules },
				protein: { value: createForm.protein, rules: proteinRules },
			});
		} else {
			valid = v.validateAll({
				date: { value: editForm.date, rules: dateRules },
				fat: { value: editForm.fat, rules: fatRules },
				protein: { value: editForm.protein, rules: proteinRules },
			});
		}
		if (!valid) return;
		await crud.submit(
			() =>
				crud.modalMode === 'create'
					? createBulkTankTest(createForm)
					: updateBulkTankTest(crud.editingId, editForm),
			crud.modalMode === 'create' ? 'Запись создана' : 'Запись обновлена',
			load,
		);
	}

	async function handleDelete() {
		await crud.remove(
			() => deleteBulkTankTest(crud.deleteId),
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
	<title>Танк-охладитель — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Танк-охладитель</h1>
	<button
		onclick={openCreate}
		class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
	>
		+ Добавить
	</button>
</div>

<FilterBar
	bind:fromDate={list.fromDate}
	bind:tillDate={list.tillDate}
	showAnimal={false}
	onsearch={load}
/>
<ErrorAlert message={list.error} />

<DataTable
	columns={[
		{ key: 'id', label: 'ID' },
		{ key: 'date', label: 'Дата' },
		{ key: 'fat', label: 'Жир, %', align: 'right' },
		{ key: 'protein', label: 'Белок, %', align: 'right' },
		{ key: 'lactose', label: 'Лактоза, %', align: 'right' },
		{ key: 'scc', label: 'СОК', align: 'right' },
		{ key: 'ffa', label: 'FFA', align: 'right' },
		{ key: 'actions', label: '', align: 'right' },
	]}
	loading={list.loading && !_hasInitial}
	initialRows={!!data.initialData && data.initialData.data.length > 0}
	bind:this={dataTable}
	emptyText="Нет данных"
>
	{#each tests as t (t.id)}
		<tr
			class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
		>
			<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{t.id}</td>
			<td class="px-4 py-3 font-medium">{t.date}</td>
			<td class="px-4 py-3 text-right font-medium">{fmtNum(t.fat)}</td>
			<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400">{fmtNum(t.protein)}</td>
			<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400">{fmtNum(t.lactose)}</td>
			<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400">{t.scc ?? '—'}</td>
			<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
				>{t.ffa != null ? fmtNum(t.ffa) : '—'}</td
			>
			<td class="px-4 py-3 text-right">
				<button
					onclick={() => openEdit(t)}
					class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
					><Pencil size={14} /></button
				>
				<button
					onclick={() => crud.confirmDelete(t.id)}
					class="text-red-500 hover:text-red-700 text-sm cursor-pointer"
					><Trash2 size={14} /></button
				>
			</td>
		</tr>
	{/each}
</DataTable>

<Modal
	open={crud.showModal}
	title={crud.modalMode === 'create' ? 'Новая запись' : 'Редактировать запись'}
	onclose={crud.close}
>
	<ErrorAlert message={crud.modalError} />
	<form onsubmit={handleSubmit} class="space-y-4">
		{#if crud.modalMode === 'create'}
			<FormField
				id="c-date"
				label="Дата"
				type="date"
				bind:value={createForm.date}
				required
				error={v.getError('date')}
				onblur={() => v.validateField('date', createForm.date, dateRules)}
			/>
			<div class="grid grid-cols-2 gap-3">
				<FormField
					id="c-fat"
					label="Жир, %"
					type="number"
					bind:value={createForm.fat}
					step="0.01"
					required
					error={v.getError('fat')}
					onblur={() => v.validateField('fat', createForm.fat, fatRules)}
				/>
				<FormField
					id="c-protein"
					label="Белок, %"
					type="number"
					bind:value={createForm.protein}
					step="0.01"
					required
					error={v.getError('protein')}
					onblur={() => v.validateField('protein', createForm.protein, proteinRules)}
				/>
			</div>
			<div class="grid grid-cols-3 gap-3">
				<FormField
					id="c-lactose"
					label="Лактоза, %"
					type="number"
					bind:value={createForm.lactose}
					step="0.01"
				/>
				<FormField id="c-scc" label="СОК" type="number" bind:value={createForm.scc} />
				<FormField id="c-ffa" label="FFA" type="number" bind:value={createForm.ffa} step="0.001" />
			</div>
		{:else}
			<FormField
				id="e-date"
				label="Дата"
				type="date"
				bind:value={editForm.date}
				required
				error={v.getError('date')}
				onblur={() => v.validateField('date', editForm.date, dateRules)}
			/>
			<div class="grid grid-cols-2 gap-3">
				<FormField
					id="e-fat"
					label="Жир, %"
					type="number"
					bind:value={editForm.fat}
					step="0.01"
					required
					error={v.getError('fat')}
					onblur={() => v.validateField('fat', editForm.fat, fatRules)}
				/>
				<FormField
					id="e-protein"
					label="Белок, %"
					type="number"
					bind:value={editForm.protein}
					step="0.01"
					required
					error={v.getError('protein')}
					onblur={() => v.validateField('protein', editForm.protein, proteinRules)}
				/>
			</div>
			<div class="grid grid-cols-3 gap-3">
				<FormField
					id="e-lactose"
					label="Лактоза, %"
					type="number"
					bind:value={editForm.lactose}
					step="0.01"
				/>
				<FormField id="e-scc" label="СОК" type="number" bind:value={editForm.scc} />
				<FormField id="e-ffa" label="FFA" type="number" bind:value={editForm.ffa} step="0.001" />
			</div>
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
	title="Удалить запись?"
	message="Это действие нельзя отменить."
	loading={crud.deleteLoading}
	onconfirm={handleDelete}
	oncancel={crud.closeDelete}
/>

<Pagination bind:page={list.page} total={_hasInitial && data.initialData ? data.initialData.total : list.total} perPage={list.perPage} />
