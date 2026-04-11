<script lang="ts">
	import {
		listContacts,
		createContact,
		updateContact,
		deleteContact,
		type Contact,
		type CreateContact,
		type UpdateContact,
	} from '$lib/api/contacts';
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
	import { Pencil, Trash2 } from 'lucide-svelte';

	let { data } = $props();

	let dataTable: DataTable;
	let contacts = $state<Contact[]>([]);

	const list = usePaginatedList();
	const crud = useCrudModal();

	let _skipLoad = !!data.initialData;
	let _hasInitial = $state(!!data.initialData);

	if (data.initialData) {
		contacts = data.initialData.data;
	}

	let form = $state<CreateContact & { active: boolean }>({
		name: '',
		type_id: undefined,
		farm_number: '',
		active: true,
		phone_cell: '',
		phone_home: '',
		phone_work: '',
		email: '',
		company_name: '',
		description: '',
	});

	const v = useFormValidation();

	async function load() {
		await list.load(
			(signal) => listContacts({ page: list.page, per_page: list.perPage }, signal),
			(data) => {
				contacts = data;
			},
			dataTable,
		);
	}

	function openCreate() {
		v.clear();
		form = {
			name: '',
			type_id: undefined,
			farm_number: '',
			active: true,
			phone_cell: '',
			phone_home: '',
			phone_work: '',
			email: '',
			company_name: '',
			description: '',
		};
		crud.openCreate();
	}

	function openEdit(c: Contact) {
		v.clear();
		form = {
			name: c.name,
			type_id: c.contact_type_id ?? undefined,
			farm_number: c.farm_number ?? '',
			active: c.active,
			phone_cell: c.phone_cell ?? '',
			phone_home: c.phone_home ?? '',
			phone_work: c.phone_work ?? '',
			email: c.email ?? '',
			company_name: c.company_name ?? '',
			description: c.description ?? '',
		};
		crud.openEdit(c.id);
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		const fields: Record<
			string,
			{ value: string | number | undefined; rules: ReturnType<typeof rules.required>[] }
		> = {
			name: { value: form.name, rules: [rules.required()] },
		};
		if (form.email) fields.email = { value: form.email, rules: [rules.email()] };
		if (!v.validateAll(fields)) return;

		const data: CreateContact & { active?: boolean } = {
			name: form.name,
			type_id: form.type_id || undefined,
			farm_number: form.farm_number || undefined,
			active: form.active,
			phone_cell: form.phone_cell || undefined,
			phone_home: form.phone_home || undefined,
			phone_work: form.phone_work || undefined,
			email: form.email || undefined,
			company_name: form.company_name || undefined,
			description: form.description || undefined,
		};
		await crud.submit(
			() =>
				crud.modalMode === 'create'
					? createContact(data)
					: updateContact(crud.editingId, { ...data } as UpdateContact),
			crud.modalMode === 'create' ? 'Контакт создан' : 'Контакт обновлён',
			load,
		);
	}

	async function handleDelete() {
		await crud.remove(
			() => deleteContact(crud.deleteId),
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
	<title>Контакты — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Контакты</h1>
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
		{ key: 'name', label: 'Имя' },
		{ key: 'company_name', label: 'Компания' },
		{ key: 'phone_cell', label: 'Телефон' },
		{ key: 'email', label: 'Email' },
		{ key: 'active', label: 'Статус' },
		{ key: 'actions', label: 'Действия', align: 'right' },
	]}
	loading={list.loading && !_hasInitial}
	initialRows={!!data.initialData && data.initialData.data.length > 0}
	bind:this={dataTable}
	emptyText="Нет контактов"
>
	{#each contacts as c (c.id)}
		<tr
			class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
		>
			<td class="px-4 py-3 font-medium text-slate-800 dark:text-slate-100">{c.name}</td>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{c.company_name || '—'}</td>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">
				{#if c.phone_cell}<span class="block">{c.phone_cell}</span>{/if}
				{#if c.phone_home}<span class="block text-xs text-slate-400 dark:text-slate-500"
						>{c.phone_home}</span
					>{/if}
				{#if !c.phone_cell && !c.phone_home}—{/if}
			</td>
			<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{c.email || '—'}</td>
			<td class="px-4 py-3">
				<span
					class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {c.active
						? 'bg-green-100 dark:bg-green-900/50 text-green-700'
						: 'bg-slate-100 dark:bg-slate-900 text-slate-500 dark:text-slate-400'}"
				>
					{c.active ? 'Активен' : 'Неактивен'}
				</span>
			</td>
			<td class="px-4 py-3 text-right">
				<div class="flex gap-2 justify-end">
					<button
						onclick={() => openEdit(c)}
						class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/50 rounded transition-colors cursor-pointer"
						><Pencil size={14} /></button
					>
					<button
						onclick={() => crud.confirmDelete(c.id)}
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
	title={crud.modalMode === 'create' ? 'Новый контакт' : 'Редактирование'}
	maxWidth="max-w-lg"
	onclose={crud.close}
>
	<ErrorAlert message={crud.modalError} />
	<form onsubmit={handleSubmit} class="space-y-4">
		<FormField id="f-name" label="Имя" bind:value={form.name} required error={v.getError('name')} />
		<div class="grid grid-cols-2 gap-4">
			<FormField id="f-company" label="Компания" bind:value={form.company_name} />
			<FormField id="f-farm" label="Номер фермы" bind:value={form.farm_number} />
		</div>
		<div class="grid grid-cols-3 gap-4">
			<FormField id="f-cell" label="Мобильный" type="tel" bind:value={form.phone_cell} />
			<FormField id="f-home" label="Домашний" type="tel" bind:value={form.phone_home} />
			<FormField id="f-work" label="Рабочий" type="tel" bind:value={form.phone_work} />
		</div>
		<FormField
			id="f-email"
			label="Email"
			type="email"
			bind:value={form.email}
			error={v.getError('email')}
		/>
		<FormField id="f-desc" label="Описание" type="textarea" bind:value={form.description} />
		<FormField label="Активен" type="checkbox" bind:checked={form.active} />
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
	title="Удалить контакт?"
	message="Это действие нельзя отменить."
	loading={crud.deleteLoading}
	onconfirm={handleDelete}
	oncancel={crud.closeDelete}
/>

<Pagination bind:page={list.page} total={_hasInitial && data.initialData ? data.initialData.total : list.total} perPage={list.perPage} />
