<script lang="ts">
	import {
		listLocations,
		createLocation,
		updateLocation,
		deleteLocation,
		type Location,
		type CreateLocation,
		type UpdateLocation,
	} from '$lib/api/locations';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';
	import { useCrudModal } from '$lib/utils/useCrudModal.svelte';
	import { usePaginatedList } from '$lib/utils/usePaginatedList.svelte';
	import { MapPin, Pencil, Trash2 } from 'lucide-svelte';

	let { data } = $props();

	let dataTable: DataTable;
	let locations = $state<Location[]>([]);

	const list = usePaginatedList();
	const crud = useCrudModal();

	let _skipLoad = !!data.initialData;
	let _hasInitial = $state(!!data.initialData);

	if (data.initialData) {
		locations = data.initialData.data;
	}

	let form = $state<{ name: string; location_type: string }>({
		name: '',
		location_type: '',
	});

	const v = useFormValidation();

	async function load() {
		await list.load(
			(signal) => listLocations({ page: list.page, per_page: list.perPage }, signal),
			(data) => {
				locations = data;
			},
			dataTable,
		);
	}

	function openCreate() {
		v.clear();
		form = { name: '', location_type: '' };
		crud.openCreate();
	}

	function openEdit(loc: Location) {
		v.clear();
		form = {
			name: loc.name,
			location_type: loc.location_type ?? '',
		};
		crud.openEdit(loc.id);
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!v.validateAll({ name: { value: form.name, rules: [rules.required()] } })) return;

		const payload: CreateLocation & { location_type?: string } = {
			name: form.name,
			location_type: form.location_type || undefined,
		};
		await crud.submit(
			() =>
				crud.modalMode === 'create'
					? createLocation(payload)
					: updateLocation(crud.editingId, { ...payload } as UpdateLocation),
			crud.modalMode === 'create' ? 'Локация создана' : 'Локация обновлена',
			load,
		);
	}

	async function handleDelete() {
		await crud.remove(
			() => deleteLocation(crud.deleteId),
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
	<title>Локации — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Локации</h1>
	<button
		onclick={openCreate}
		class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
	>
		+ Добавить
	</button>
</div>

<ErrorAlert message={list.error} />

<DataTable
	columns={[
		{ key: 'name', label: 'Название' },
		{ key: 'location_type', label: 'Тип' },
		{ key: 'created_at', label: 'Создано' },
		{ key: 'actions', label: 'Действия', align: 'right' },
	]}
	loading={list.loading && !_hasInitial}
	initialRows={!!data.initialData && data.initialData.data.length > 0}
	bind:this={dataTable}
	emptyText="Нет локаций"
>
	{#each locations as loc (loc.id)}
		<tr
			class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
		>
			<td class="px-4 py-3 font-medium text-slate-800 dark:text-slate-100">
				<span class="inline-flex items-center gap-2">
					<MapPin size={14} class="text-slate-400" />
					{loc.name}
				</span>
			</td>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{loc.location_type || '—'}</td>
			<td class="px-4 py-3 text-slate-500 dark:text-slate-400 text-sm">
				{new Date(loc.created_at).toLocaleDateString('ru-RU')}
			</td>
			<td class="px-4 py-3 text-right">
				<div class="flex gap-2 justify-end">
					<button
						onclick={() => openEdit(loc)}
						aria-label="Редактировать {loc.name}"
						class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/50 rounded transition-colors cursor-pointer"
						><Pencil size={14} /></button
					>
					<button
						onclick={() => crud.confirmDelete(loc.id)}
						aria-label="Удалить {loc.name}"
						class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-red-600 hover:bg-red-50 dark:bg-red-900/50 rounded transition-colors cursor-pointer"
						><Trash2 size={14} /></button
					>
				</div>
			</td>
		</tr>
	{/each}
</DataTable>

<Modal
	open={crud.showModal}
	title={crud.modalMode === 'create' ? 'Новая локация' : 'Редактирование'}
	onclose={crud.close}
>
	<ErrorAlert message={crud.modalError} />
	<form onsubmit={handleSubmit} class="space-y-4">
		<FormField id="f-name" label="Название" bind:value={form.name} required error={v.getError('name')} />
		<FormField id="f-type" label="Тип локации" bind:value={form.location_type} />
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
				{crud.modalLoading ? 'Сохранение...' : 'Сохранить'}
			</button>
		</div>
	</form>
</Modal>

<ConfirmDialog
	open={crud.showDelete}
	title="Удалить локацию?"
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
