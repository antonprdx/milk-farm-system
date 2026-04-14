<script lang="ts">
	import {
		listTasks,
		createTask,
		updateTask,
		deleteTask,
		type Task,
		type CreateTask,
		type UpdateTask,
		type TaskStatus,
		type TaskPriority,
		type TaskCategory,
		TASK_STATUS_LABELS,
		TASK_PRIORITY_LABELS,
		TASK_CATEGORY_LABELS,
	} from '$lib/api/tasks';
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
	import { Pencil, Trash2 } from 'lucide-svelte';

	let dataTable: DataTable;
	let tasks = $state<Task[]>([]);

	const list = usePaginatedList();
	const crud = useCrudModal();
	const v = useFormValidation();

	let _skipLoad = true;
	let today = new Date().toISOString().slice(0, 10);

	let filterStatus = $state<TaskStatus | ''>('');
	let filterPriority = $state<TaskPriority | ''>('');
	let filterCategory = $state<TaskCategory | ''>('');
	let filterOverdue = $state(false);

	let createForm = $state<CreateTask>({ title: '', priority: 'medium', category: 'other' });
	let editForm = $state<UpdateTask>({});

	const titleRules = [rules.required()];

	async function load() {
		await list.load(
			(signal) =>
				listTasks(
					{
						status: filterStatus || undefined,
						priority: filterPriority || undefined,
						category: filterCategory || undefined,
						overdue: filterOverdue || undefined,
						from_date: list.fromDate || undefined,
						till_date: list.tillDate || undefined,
						page: list.page,
						per_page: list.perPage,
					},
					signal,
				),
			(data) => {
				tasks = data;
			},
			dataTable,
		);
	}

	function openCreate() {
		createForm = { title: '', priority: 'medium', category: 'other', due_date: today };
		v.clear();
		crud.openCreate();
	}

	function openEdit(t: Task) {
		editForm = {
			title: t.title,
			description: t.description ?? undefined,
			category: t.category,
			priority: t.priority,
			status: t.status,
			animal_id: t.animal_id ?? undefined,
			due_date: t.due_date ?? undefined,
			assigned_to: t.assigned_to ?? undefined,
		};
		v.clear();
		crud.openEdit(t.id);
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		let valid: boolean;
		if (crud.modalMode === 'create') {
			valid = v.validateAll({
				title: { value: createForm.title, rules: titleRules },
			});
		} else {
			valid = v.validateAll({
				title: { value: editForm.title, rules: titleRules },
			});
		}
		if (!valid) return;
		await crud.submit(
			() =>
				crud.modalMode === 'create'
					? createTask(createForm)
					: updateTask(crud.editingId, editForm),
			crud.modalMode === 'create' ? 'Задача создана' : 'Задача обновлена',
			load,
		);
	}

	async function handleDelete() {
		await crud.remove(
			() => deleteTask(crud.deleteId),
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
		load();
	});

	function statusBadge(s: TaskStatus): string {
		const m: Record<string, string> = {
			pending: 'bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-400',
			in_progress: 'bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-400',
			done: 'bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-400',
			cancelled: 'bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-400',
		};
		return m[s] ?? m['pending'];
	}

	function priorityBadge(p: TaskPriority): string {
		const m: Record<string, string> = {
			low: 'bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-400',
			medium: 'bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-400',
			high: 'bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-400',
			urgent: 'bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-400',
		};
		return m[p] ?? m['medium'];
	}

	function isOverdue(t: Task): boolean {
		if (t.status === 'done' || t.status === 'cancelled') return false;
		if (!t.due_date) return false;
		return t.due_date < today;
	}

	function selectCls(): string {
		return 'px-2 py-1 text-sm border border-slate-300 dark:border-slate-600 rounded bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200';
	}
</script>

<svelte:head>
	<title>Задачи — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-4">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Задачи</h1>
	<button
		onclick={openCreate}
		class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
	>
		+ Добавить
	</button>
</div>

<div
	class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-6"
>
	<div class="flex flex-wrap gap-3 items-end">
		<div>
			<label class="block text-xs text-slate-500 dark:text-slate-400 mb-1">Статус</label>
			<select bind:value={filterStatus} class={selectCls()}>
				<option value="">Все</option>
				{#each Object.entries(TASK_STATUS_LABELS) as [key, label] (key)}
					<option value={key}>{label}</option>
				{/each}
			</select>
		</div>
		<div>
			<label class="block text-xs text-slate-500 dark:text-slate-400 mb-1">Приоритет</label>
			<select bind:value={filterPriority} class={selectCls()}>
				<option value="">Все</option>
				{#each Object.entries(TASK_PRIORITY_LABELS) as [key, label] (key)}
					<option value={key}>{label}</option>
				{/each}
			</select>
		</div>
		<div>
			<label class="block text-xs text-slate-500 dark:text-slate-400 mb-1">Категория</label>
			<select bind:value={filterCategory} class={selectCls()}>
				<option value="">Все</option>
				{#each Object.entries(TASK_CATEGORY_LABELS) as [key, label] (key)}
					<option value={key}>{label}</option>
				{/each}
			</select>
		</div>
		<div>
			<label class="flex items-center gap-2 text-xs text-slate-500 dark:text-slate-400 mt-4 cursor-pointer">
				<input type="checkbox" bind:checked={filterOverdue} class="rounded" />
				Просроченные
			</label>
		</div>
		<div>
			<label class="block text-xs text-slate-500 dark:text-slate-400 mb-1">С</label>
			<input
				type="date"
				bind:value={list.fromDate}
				class="px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
			/>
		</div>
		<div>
			<label class="block text-xs text-slate-500 dark:text-slate-400 mb-1">По</label>
			<input
				type="date"
				bind:value={list.tillDate}
				class="px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
			/>
		</div>
		<button
			onclick={load}
			class="px-4 py-2 bg-slate-100 dark:bg-slate-900 hover:bg-slate-200 dark:bg-slate-700 text-slate-700 dark:text-slate-300 text-sm rounded-lg transition-colors cursor-pointer"
			>Найти</button
		>
	</div>
</div>

<ErrorAlert message={list.error} />

<DataTable
	columns={[
		{ key: 'id', label: 'ID' },
		{ key: 'title', label: 'Задача' },
		{ key: 'category', label: 'Категория' },
		{ key: 'priority', label: 'Приоритет' },
		{ key: 'status', label: 'Статус' },
		{ key: 'animal_id', label: 'Животное' },
		{ key: 'due_date', label: 'Срок' },
		{ key: 'assigned_to', label: 'Ответственный' },
		{ key: 'actions', label: '', align: 'right' },
	]}
	loading={list.loading}
	bind:this={dataTable}
	emptyText="Нет задач"
	initialRows={false}
>
	{#each tasks as t (t.id)}
		<tr
			class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors {isOverdue(t)
				? 'bg-red-50 dark:bg-red-900/10'
				: ''}"
		>
			<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{t.id}</td>
			<td class="px-4 py-3 font-medium">{t.title}</td>
			<td class="px-4 py-3 text-sm text-slate-600 dark:text-slate-400"
				>{TASK_CATEGORY_LABELS[t.category]}</td
			>
			<td class="px-4 py-3">
				<span
					class="px-2 py-0.5 rounded-full text-xs font-medium {priorityBadge(t.priority)}"
					>{TASK_PRIORITY_LABELS[t.priority]}</span
				>
			</td>
			<td class="px-4 py-3">
				<span
					class="px-2 py-0.5 rounded-full text-xs font-medium {statusBadge(t.status)}"
					>{TASK_STATUS_LABELS[t.status]}</span
				>
			</td>
			<td class="px-4 py-3 text-sm">{t.animal_id ? `#${t.animal_id}` : '—'}</td>
			<td class="px-4 py-3 text-sm {isOverdue(t) ? 'text-red-600 dark:text-red-400 font-semibold' : ''}"
				>{t.due_date ?? '—'}</td
			>
			<td class="px-4 py-3 text-sm text-slate-600 dark:text-slate-400"
				>{t.assigned_to ?? '—'}</td
			>
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
	title={crud.modalMode === 'create' ? 'Новая задача' : 'Редактировать задачу'}
	onclose={crud.close}
>
	<ErrorAlert message={crud.modalError} />
	<form onsubmit={handleSubmit} class="space-y-4">
		{#if crud.modalMode === 'create'}
			<FormField
				id="c-title"
				label="Заголовок"
				bind:value={createForm.title}
				required
				error={v.getError('title')}
				onblur={() => v.validateField('title', createForm.title, titleRules)}
			/>
			<FormField id="c-desc" label="Описание" type="textarea" bind:value={createForm.description} />
			<div class="grid grid-cols-2 gap-3">
				<div>
					<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
						>Категория</label
					>
					<select
						bind:value={createForm.category}
						class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
					>
						{#each Object.entries(TASK_CATEGORY_LABELS) as [key, label] (key)}
							<option value={key}>{label}</option>
						{/each}
					</select>
				</div>
				<div>
					<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
						>Приоритет</label
					>
					<select
						bind:value={createForm.priority}
						class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
					>
						{#each Object.entries(TASK_PRIORITY_LABELS) as [key, label] (key)}
							<option value={key}>{label}</option>
						{/each}
					</select>
				</div>
			</div>
			<div class="grid grid-cols-2 gap-3">
				<FormField id="c-animal" label="ID животного" type="number" bind:value={createForm.animal_id} />
				<FormField id="c-due" label="Срок" type="date" bind:value={createForm.due_date} />
			</div>
			<FormField id="c-assigned" label="Ответственный" bind:value={createForm.assigned_to} />
		{:else}
			<FormField
				id="e-title"
				label="Заголовок"
				bind:value={editForm.title}
				required
				error={v.getError('title')}
				onblur={() => v.validateField('title', editForm.title, titleRules)}
			/>
			<FormField id="e-desc" label="Описание" type="textarea" bind:value={editForm.description} />
			<div class="grid grid-cols-3 gap-3">
				<div>
					<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
						>Категория</label
					>
					<select
						bind:value={editForm.category}
						class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
					>
						{#each Object.entries(TASK_CATEGORY_LABELS) as [key, label] (key)}
							<option value={key}>{label}</option>
						{/each}
					</select>
				</div>
				<div>
					<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
						>Приоритет</label
					>
					<select
						bind:value={editForm.priority}
						class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
					>
						{#each Object.entries(TASK_PRIORITY_LABELS) as [key, label] (key)}
							<option value={key}>{label}</option>
						{/each}
					</select>
				</div>
				<div>
					<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
						>Статус</label
					>
					<select
						bind:value={editForm.status}
						class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
					>
						{#each Object.entries(TASK_STATUS_LABELS) as [key, label] (key)}
							<option value={key}>{label}</option>
						{/each}
					</select>
				</div>
			</div>
			<div class="grid grid-cols-2 gap-3">
				<FormField id="e-animal" label="ID животного" type="number" bind:value={editForm.animal_id} />
				<FormField id="e-due" label="Срок" type="date" bind:value={editForm.due_date} />
			</div>
			<FormField id="e-assigned" label="Ответственный" bind:value={editForm.assigned_to} />
		{/if}
		<div class="flex gap-3 justify-end pt-2">
			<button
				type="button"
				onclick={crud.close}
				class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-800/50 cursor-pointer"
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
	title="Удалить задачу?"
	message="Это действие нельзя отменить."
	loading={crud.deleteLoading}
	onconfirm={handleDelete}
	oncancel={crud.closeDelete}
/>

<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />
