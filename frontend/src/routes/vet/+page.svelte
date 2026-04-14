<script lang="ts">
	import {
		listVetRecords,
		createVetRecord,
		updateVetRecord,
		deleteVetRecord,
		listWeightRecords,
		createWeightRecord,
		deleteWeightRecord,
		type VetRecord,
		type CreateVetRecord,
		type UpdateVetRecord,
		type VetRecordType,
		type VetRecordStatus,
		type WeightRecord,
		type CreateWeightRecord,
		VET_RECORD_TYPE_LABELS,
		VET_STATUS_LABELS,
	} from '$lib/api/vet';
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

	type Tab = 'records' | 'weights';
	let activeTab: Tab = $state('records');

	let dataTable: DataTable;
	let vetRecords = $state<VetRecord[]>([]);
	let weightRecords = $state<WeightRecord[]>([]);

	const list = usePaginatedList();
	const crud = useCrudModal();
	const v = useFormValidation();

	let _skipLoad = true;
	let today = new Date().toISOString().slice(0, 10);

	let createVetForm = $state<CreateVetRecord>({
		animal_id: 0,
		record_type: 'examination',
		event_date: today,
	});
	let editVetForm = $state<UpdateVetRecord>({});

	let createWeightForm = $state<CreateWeightRecord>({
		animal_id: 0,
		weight_kg: 0,
		measure_date: today,
	});

	let filterAnimalId = $state<number | ''>('');
	let filterType = $state<VetRecordType | ''>('');

	const animalIdRules = [rules.required()];
	const dateRules = [rules.required()];

	async function load() {
		if (activeTab === 'records') {
			await list.load(
				(signal) =>
					listVetRecords(
						{
							animal_id: filterAnimalId || undefined,
							record_type: filterType || undefined,
							from_date: list.fromDate || undefined,
							till_date: list.tillDate || undefined,
							page: list.page,
							per_page: list.perPage,
						},
						signal,
					),
				(data) => {
					vetRecords = data;
				},
				dataTable,
			);
		} else {
			await list.load(
				(signal) =>
					listWeightRecords(
						{
							animal_id: filterAnimalId || undefined,
							from_date: list.fromDate || undefined,
							till_date: list.tillDate || undefined,
							page: list.page,
							per_page: list.perPage,
						},
						signal,
					),
				(data) => {
					weightRecords = data;
				},
				dataTable,
			);
		}
	}

	function switchTab(tab: Tab) {
		activeTab = tab;
		list.page = 1;
		_skipLoad = false;
		load();
	}

	function openCreateVet() {
		createVetForm = {
			animal_id: filterAnimalId || 0,
			record_type: 'examination',
			event_date: today,
		};
		v.clear();
		crud.openCreate();
	}

	function openEditVet(r: VetRecord) {
		editVetForm = {
			record_type: r.record_type,
			status: r.status,
			event_date: r.event_date,
			diagnosis: r.diagnosis ?? undefined,
			treatment: r.treatment ?? undefined,
			medication: r.medication ?? undefined,
			dosage: r.dosage ?? undefined,
			withdrawal_days: r.withdrawal_days ?? undefined,
			veterinarian: r.veterinarian ?? undefined,
			notes: r.notes ?? undefined,
			follow_up_date: r.follow_up_date ?? undefined,
		};
		v.clear();
		crud.openEdit(r.id);
	}

	function openCreateWeight() {
		createWeightForm = {
			animal_id: filterAnimalId || 0,
			weight_kg: 0,
			measure_date: today,
		};
		v.clear();
		crud.openCreate();
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (activeTab === 'records') {
			if (crud.modalMode === 'create') {
				const valid = v.validateAll({
					animal_id: { value: createVetForm.animal_id, rules: animalIdRules },
					event_date: { value: createVetForm.event_date, rules: dateRules },
				});
				if (!valid) return;
				await crud.submit(() => createVetRecord(createVetForm), 'Запись создана', load);
			} else {
				await crud.submit(
					() => updateVetRecord(crud.editingId, editVetForm),
					'Запись обновлена',
					load,
				);
			}
		} else {
			const valid = v.validateAll({
				animal_id: { value: createWeightForm.animal_id, rules: animalIdRules },
				weight_kg: { value: createWeightForm.weight_kg, rules: [rules.required()] },
				measure_date: { value: createWeightForm.measure_date, rules: dateRules },
			});
			if (!valid) return;
			await crud.submit(() => createWeightRecord(createWeightForm), 'Запись создана', load);
		}
	}

	async function handleDelete() {
		if (activeTab === 'records') {
			await crud.remove(() => deleteVetRecord(crud.deleteId), load, (msg) => (list.error = msg));
		} else {
			await crud.remove(
				() => deleteWeightRecord(crud.deleteId),
				load,
				(msg) => (list.error = msg),
			);
		}
	}

	$effect(() => {
		list.page;
		if (_skipLoad) {
			_skipLoad = false;
			return;
		}
		load();
	});

	function typeBadge(t: VetRecordType): string {
		const m: Record<string, string> = {
			vaccination: 'bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-400',
			treatment: 'bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-400',
			disease: 'bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-400',
			surgery: 'bg-purple-100 dark:bg-purple-900/40 text-purple-700 dark:text-purple-400',
			deworming: 'bg-teal-100 dark:bg-teal-900/40 text-teal-700 dark:text-teal-400',
			hoof_care: 'bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-400',
			examination: 'bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-400',
			other: 'bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-400',
		};
		return m[t] ?? m['other'];
	}

	function statusBadge(s: VetRecordStatus): string {
		const m: Record<string, string> = {
			planned: 'bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-400',
			in_progress: 'bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-400',
			completed: 'bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-400',
			cancelled: 'bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-500',
		};
		return m[s] ?? m['planned'];
	}
</script>

<svelte:head>
	<title>Ветеринарный журнал — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-4">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Ветеринарный журнал</h1>
	<button
		onclick={activeTab === 'records' ? openCreateVet : openCreateWeight}
		class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
	>
		+ {activeTab === 'records' ? 'Запись' : 'Взвешивание'}
	</button>
</div>

<div class="flex gap-1 border-b border-slate-200 dark:border-slate-700 mb-4">
	<button
		onclick={() => switchTab('records')}
		class="px-4 py-2 text-sm font-medium transition-colors cursor-pointer relative {activeTab === 'records'
			? 'text-blue-600 dark:text-blue-400'
			: 'text-slate-500 hover:text-slate-700'}"
	>
		Вет. записи
		{#if activeTab === 'records'}
			<span class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-600 dark:bg-blue-400 rounded-full" />
		{/if}
	</button>
	<button
		onclick={() => switchTab('weights')}
		class="px-4 py-2 text-sm font-medium transition-colors cursor-pointer relative {activeTab === 'weights'
			? 'text-blue-600 dark:text-blue-400'
			: 'text-slate-500 hover:text-slate-700'}"
	>
		Вес / BCS
		{#if activeTab === 'weights'}
			<span class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-600 dark:bg-blue-400 rounded-full" />
		{/if}
	</button>
</div>

<div
	class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-6"
>
	<div class="flex flex-wrap gap-3 items-end">
		<div>
			<label class="block text-xs text-slate-500 dark:text-slate-400 mb-1">ID коровы</label>
			<input
				type="number"
				bind:value={filterAnimalId}
				placeholder="Все"
				class="w-24 px-2 py-1 text-sm border border-slate-300 dark:border-slate-600 rounded bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
			/>
		</div>
		{#if activeTab === 'records'}
			<div>
				<label class="block text-xs text-slate-500 dark:text-slate-400 mb-1">Тип</label>
				<select
					bind:value={filterType}
					class="px-2 py-1 text-sm border border-slate-300 dark:border-slate-600 rounded bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
				>
					<option value="">Все</option>
					{#each Object.entries(VET_RECORD_TYPE_LABELS) as [key, label] (key)}
						<option value={key}>{label}</option>
					{/each}
				</select>
			</div>
		{/if}
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

{#if activeTab === 'records'}
	<DataTable
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Корова' },
			{ key: 'record_type', label: 'Тип' },
			{ key: 'status', label: 'Статус' },
			{ key: 'event_date', label: 'Дата' },
			{ key: 'diagnosis', label: 'Диагноз' },
			{ key: 'medication', label: 'Препарат' },
			{ key: 'withdrawal', label: 'Ожидание' },
			{ key: 'veterinarian', label: 'Ветеринар' },
			{ key: 'actions', label: '', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dataTable}
		emptyText="Нет ветеринарных записей"
		initialRows={false}
	>
		{#each vetRecords as r (r.id)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{r.id}</td>
				<td class="px-4 py-3 font-medium">#{r.animal_id}</td>
				<td class="px-4 py-3">
					<span
						class="px-2 py-0.5 rounded-full text-xs font-medium {typeBadge(r.record_type)}"
						>{VET_RECORD_TYPE_LABELS[r.record_type]}</span
					>
				</td>
				<td class="px-4 py-3">
					<span
						class="px-2 py-0.5 rounded-full text-xs font-medium {statusBadge(r.status)}"
						>{VET_STATUS_LABELS[r.status]}</span
					>
				</td>
				<td class="px-4 py-3">{r.event_date}</td>
				<td class="px-4 py-3 text-sm text-slate-600 dark:text-slate-400 max-w-[200px] truncate"
					>{r.diagnosis ?? '—'}</td
				>
				<td class="px-4 py-3 text-sm text-slate-600 dark:text-slate-400 max-w-[150px] truncate"
					>{r.medication ?? '—'}</td
				>
				<td class="px-4 py-3 text-sm">
					{#if r.withdrawal_end_date}
						<span class="text-orange-600 dark:text-orange-400 font-medium"
							>до {r.withdrawal_end_date}</span
						>
					{:else}
						—
					{/if}
				</td>
				<td class="px-4 py-3 text-sm text-slate-600 dark:text-slate-400"
					>{r.veterinarian ?? '—'}</td
				>
				<td class="px-4 py-3 text-right">
					<button
						onclick={() => openEditVet(r)}
						aria-label="Редактировать"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
						><Pencil size={14} /></button
					>
					<button
						onclick={() => crud.confirmDelete(r.id)}
						aria-label="Удалить"
						class="text-red-500 hover:text-red-700 text-sm cursor-pointer"
						><Trash2 size={14} /></button
					>
				</td>
			</tr>
		{/each}
	</DataTable>

	<Modal
		open={crud.showModal && (activeTab === 'records' || crud.modalMode === 'create')}
		title={crud.modalMode === 'create' ? 'Новая вет. запись' : 'Редактировать запись'}
		onclose={crud.close}
	>
		<ErrorAlert message={crud.modalError} />
		<form onsubmit={handleSubmit} class="space-y-4">
			{#if crud.modalMode === 'create'}
				<FormField id="c-aid" label="ID коровы" type="number" bind:value={createVetForm.animal_id} required
					error={v.getError('animal_id')}
					onblur={() => v.validateField('animal_id', createVetForm.animal_id, animalIdRules)}
				/>
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
							>Тип записи</label
						>
						<select
							bind:value={createVetForm.record_type}
							class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
						>
							{#each Object.entries(VET_RECORD_TYPE_LABELS) as [key, label] (key)}
								<option value={key}>{label}</option>
							{/each}
						</select>
					</div>
					<div>
						<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
							>Статус</label
						>
						<select
							bind:value={createVetForm.status}
							class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
						>
							{#each Object.entries(VET_STATUS_LABELS) as [key, label] (key)}
								<option value={key}>{label}</option>
							{/each}
						</select>
					</div>
				</div>
				<FormField id="c-date" label="Дата" type="date" bind:value={createVetForm.event_date} required
					error={v.getError('event_date')}
					onblur={() => v.validateField('event_date', createVetForm.event_date, dateRules)}
				/>
				<FormField id="c-diag" label="Диагноз" bind:value={createVetForm.diagnosis} />
				<FormField id="c-treat" label="Лечение" bind:value={createVetForm.treatment} />
				<div class="grid grid-cols-2 gap-3">
					<FormField id="c-med" label="Препарат" bind:value={createVetForm.medication} />
					<FormField id="c-dosage" label="Дозировка" bind:value={createVetForm.dosage} />
				</div>
				<div class="grid grid-cols-2 gap-3">
					<FormField id="c-wd" label="Срок ожидания (дни)" type="number" bind:value={createVetForm.withdrawal_days} />
					<FormField id="c-vet" label="Ветеринар" bind:value={createVetForm.veterinarian} />
				</div>
				<FormField id="c-follow" label="Дата повторного осмотра" type="date" bind:value={createVetForm.follow_up_date} />
				<FormField id="c-notes" label="Заметки" type="textarea" bind:value={createVetForm.notes} />
			{:else}
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
							>Тип записи</label
						>
						<select
							bind:value={editVetForm.record_type}
							class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
						>
							{#each Object.entries(VET_RECORD_TYPE_LABELS) as [key, label] (key)}
								<option value={key}>{label}</option>
							{/each}
						</select>
					</div>
					<div>
						<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
							>Статус</label
						>
						<select
							bind:value={editVetForm.status}
							class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
						>
							{#each Object.entries(VET_STATUS_LABELS) as [key, label] (key)}
								<option value={key}>{label}</option>
							{/each}
						</select>
					</div>
				</div>
				<FormField id="e-date" label="Дата" type="date" bind:value={editVetForm.event_date} required
					error={v.getError('event_date')}
					onblur={() => v.validateField('event_date', editVetForm.event_date, dateRules)}
				/>
				<FormField id="e-diag" label="Диагноз" bind:value={editVetForm.diagnosis} />
				<FormField id="e-treat" label="Лечение" bind:value={editVetForm.treatment} />
				<div class="grid grid-cols-2 gap-3">
					<FormField id="e-med" label="Препарат" bind:value={editVetForm.medication} />
					<FormField id="e-dosage" label="Дозировка" bind:value={editVetForm.dosage} />
				</div>
				<div class="grid grid-cols-2 gap-3">
					<FormField id="e-wd" label="Срок ожидания (дни)" type="number" bind:value={editVetForm.withdrawal_days} />
					<FormField id="e-vet" label="Ветеринар" bind:value={editVetForm.veterinarian} />
				</div>
				<FormField id="e-follow" label="Дата повторного осмотра" type="date" bind:value={editVetForm.follow_up_date} />
				<FormField id="e-notes" label="Заметки" type="textarea" bind:value={editVetForm.notes} />
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
					{crud.modalLoading ? 'Сохранение...' : crud.modalMode === 'create' ? 'Создать' : 'Сохранить'}
				</button>
			</div>
		</form>
	</Modal>
{:else}
	<DataTable
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Корова' },
			{ key: 'weight_kg', label: 'Вес, кг', align: 'right' },
			{ key: 'bcs', label: 'BCS', align: 'right' },
			{ key: 'measure_date', label: 'Дата' },
			{ key: 'notes', label: 'Заметки' },
			{ key: 'actions', label: '', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dataTable}
		emptyText="Нет записей о взвешивании"
		initialRows={false}
	>
		{#each weightRecords as w (w.id)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{w.id}</td>
				<td class="px-4 py-3 font-medium">#{w.animal_id}</td>
				<td class="px-4 py-3 text-right font-medium">{w.weight_kg.toFixed(1)}</td>
				<td class="px-4 py-3 text-right">{w.bcs?.toFixed(1) ?? '—'}</td>
				<td class="px-4 py-3">{w.measure_date}</td>
				<td class="px-4 py-3 text-sm text-slate-600 dark:text-slate-400 max-w-[200px] truncate"
					>{w.notes ?? '—'}</td
				>
				<td class="px-4 py-3 text-right">
					<button
						onclick={() => crud.confirmDelete(w.id)}
						aria-label="Удалить"
						class="text-red-500 hover:text-red-700 text-sm cursor-pointer"
						><Trash2 size={14} /></button
					>
				</td>
			</tr>
		{/each}
	</DataTable>

	<Modal
		open={crud.showModal && activeTab === 'weights' && crud.modalMode === 'create'}
		title="Новое взвешивание"
		onclose={crud.close}
	>
		<ErrorAlert message={crud.modalError} />
		<form onsubmit={handleSubmit} class="space-y-4">
			<FormField id="w-aid" label="ID коровы" type="number" bind:value={createWeightForm.animal_id} required
				error={v.getError('animal_id')}
				onblur={() => v.validateField('animal_id', createWeightForm.animal_id, animalIdRules)}
			/>
			<div class="grid grid-cols-2 gap-3">
				<FormField id="w-weight" label="Вес, кг" type="number" step="0.1" bind:value={createWeightForm.weight_kg} required
					error={v.getError('weight_kg')}
					onblur={() => v.validateField('weight_kg', createWeightForm.weight_kg, [rules.required()])}
				/>
				<FormField id="w-bcs" label="BCS (1-5)" type="number" step="0.1" bind:value={createWeightForm.bcs} />
			</div>
			<FormField id="w-date" label="Дата" type="date" bind:value={createWeightForm.measure_date} required
				error={v.getError('measure_date')}
				onblur={() => v.validateField('measure_date', createWeightForm.measure_date, dateRules)}
			/>
			<FormField id="w-notes" label="Заметки" bind:value={createWeightForm.notes} />
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
					{crud.modalLoading ? 'Сохранение...' : 'Создать'}
				</button>
			</div>
		</form>
	</Modal>
{/if}

<ConfirmDialog
	open={crud.showDelete}
	title="Удалить запись?"
	message="Это действие нельзя отменить."
	loading={crud.deleteLoading}
	onconfirm={handleDelete}
	oncancel={crud.closeDelete}
/>

<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />
