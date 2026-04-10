<script lang="ts">
	import {
		listProductions,
		listVisits,
		listQuality,
		createProduction,
		updateProduction,
		deleteProduction,
		type MilkDayProduction,
		type MilkVisit,
		type MilkQuality,
		type CreateMilkDayProduction,
		type UpdateMilkDayProduction,
	} from '$lib/api/milk';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import FilterBar from '$lib/components/ui/FilterBar.svelte';
	import TabBar from '$lib/components/ui/TabBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import MilkChart from '$lib/components/MilkChart.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { fmtNum, formatDatetime } from '$lib/utils/format';
	import { toasts } from '$lib/stores/toast';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';
	import { Pencil, Trash2 } from 'lucide-svelte';

	type Tab = 'productions' | 'visits' | 'quality';
	type ModalMode = 'create' | 'edit';

	let tab = $state<Tab>('productions');
	let loading = $state(true);
	let error = $state('');

	let productions = $state<MilkDayProduction[]>([]);
	let visits = $state<MilkVisit[]>([]);
	let quality = $state<MilkQuality[]>([]);
	let total = $state(0);
	let page = $state(1);
	let perPage = 20;

	let fromDate = $state('');
	let tillDate = $state('');
	let animalId = $state('');

	let modalMode = $state<ModalMode>('create');
	let showModal = $state(false);
	let modalLoading = $state(false);
	let modalError = $state('');
	let editingId = $state(0);

	let showDelete = $state(false);
	let deleteLoading = $state(false);
	let deleteId = $state(0);

	let today = new Date().toISOString().slice(0, 10);

	let createForm = $state<CreateMilkDayProduction>({ animal_id: 0, date: today });
	let editForm = $state({ date: '', milk_amount: '', avg_amount: '', avg_weight: '', isk: '' });

	const v = useFormValidation();

	async function load() {
		try {
			loading = true;
			error = '';
			const filter = {
				animal_id: animalId ? Number(animalId) : undefined,
				from_date: fromDate || undefined,
				till_date: tillDate || undefined,
				page,
				per_page: perPage,
			};
			if (tab === 'productions') {
				const res = await listProductions(filter);
				productions = res.data;
				total = res.total;
			} else if (tab === 'visits') {
				const res = await listVisits(filter);
				visits = res.data;
				total = res.total;
			} else {
				const res = await listQuality(filter);
				quality = res.data;
				total = res.total;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			loading = false;
		}
	}

	function switchTab(t: Tab) {
		tab = t;
		page = 1;
		load();
	}

	function openCreate() {
		modalMode = 'create';
		modalError = '';
		createForm = { animal_id: 0, date: today };
		v.clear();
		showModal = true;
	}

	function openEdit(p: MilkDayProduction) {
		modalMode = 'edit';
		editingId = p.id;
		modalError = '';
		v.clear();
		editForm = {
			date: p.date,
			milk_amount: p.milk_amount != null ? String(p.milk_amount) : '',
			avg_amount: p.avg_amount != null ? String(p.avg_amount) : '',
			avg_weight: p.avg_weight != null ? String(p.avg_weight) : '',
			isk: p.isk != null ? String(p.isk) : '',
		};
		showModal = true;
	}

	function confirmDelete(id: number) {
		deleteId = id;
		showDelete = true;
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		modalError = '';
		if (modalMode === 'create') {
			if (
				!v.validateAll({
					animal_id: {
						value: createForm.animal_id,
						rules: [rules.required(), rules.positive('Укажите ID животного')],
					},
					date: { value: createForm.date, rules: [rules.required()] },
				})
			)
				return;
		} else {
			if (
				!v.validateAll({
					date: { value: editForm.date, rules: [rules.required()] },
				})
			)
				return;
		}
		try {
			modalLoading = true;
			if (modalMode === 'create') {
				await createProduction(createForm);
				toasts.success('Надой создан');
			} else {
				const data: UpdateMilkDayProduction = {};
				if (editForm.date) data.date = editForm.date;
				if (editForm.milk_amount) data.milk_amount = Number(editForm.milk_amount);
				if (editForm.avg_amount) data.avg_amount = Number(editForm.avg_amount);
				if (editForm.avg_weight) data.avg_weight = Number(editForm.avg_weight);
				if (editForm.isk) data.isk = Number(editForm.isk);
				await updateProduction(editingId, data);
				toasts.success('Надой обновлён');
			}
			showModal = false;
			load();
		} catch (e) {
			modalError = e instanceof Error ? e.message : 'Ошибка';
		} finally {
			modalLoading = false;
		}
	}

	async function handleDelete() {
		try {
			deleteLoading = true;
			await deleteProduction(deleteId);
			showDelete = false;
			toasts.success('Надой удалён');
			load();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка удаления';
			showDelete = false;
		} finally {
			deleteLoading = false;
		}
	}

	$effect(() => {
		page;
		tab;
		load();
	});
</script>

<svelte:head>
	<title>Удои — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Удои</h1>
	{#if tab === 'productions'}
		<button
			onclick={openCreate}
			class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
		>
			+ Добавить надой
		</button>
	{/if}
</div>

<TabBar
	tabs={[
		{ key: 'productions', label: 'Удои за день' },
		{ key: 'visits', label: 'Визиты' },
		{ key: 'quality', label: 'Качество' },
	]}
	bind:active={tab}
	onchange={(t: string) => switchTab(t as Tab)}
/>

<FilterBar bind:fromDate bind:tillDate bind:animalId onsearch={load} />

{#if tab === 'productions' && productions.length > 0}
	<MilkChart {productions} />
{/if}

<ErrorAlert message={error} />

{#if tab === 'productions'}
	<DataTable
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'date', label: 'Дата' },
			{ key: 'milk_amount', label: 'Надой, л', align: 'right' },
			{ key: 'avg_amount', label: 'Средний надой', align: 'right' },
			{ key: 'avg_weight', label: 'Средний вес', align: 'right' },
			{ key: 'isk', label: 'ИСК', align: 'right' },
			{ key: 'actions', label: '', align: 'right' },
		]}
		{loading}
	>
		{#if productions.length === 0}
			<tr
				><td colspan="8" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
					>Нет данных</td
				></tr
			>
		{:else}
			{#each productions as p (p.id)}
				<tr
					class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
				>
					<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{p.id}</td>
					<td class="px-4 py-3"
						><a
							href="/animals/{p.animal_id}"
							class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
							>#{p.animal_id}</a
						></td
					>
					<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{p.date}</td>
					<td class="px-4 py-3 text-right font-medium">{fmtNum(p.milk_amount)}</td>
					<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
						>{fmtNum(p.avg_amount)}</td
					>
					<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
						>{fmtNum(p.avg_weight)}</td
					>
					<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400">{fmtNum(p.isk)}</td>
					<td class="px-4 py-3 text-right">
						<button
							onclick={() => openEdit(p)}
							class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
							><Pencil size={14} /></button
						>
						<button
							onclick={() => confirmDelete(p.id)}
							class="text-red-500 hover:text-red-700 text-sm cursor-pointer"
							><Trash2 size={14} /></button
						>
					</td>
				</tr>
			{/each}
		{/if}
	</DataTable>
{:else if tab === 'visits'}
	<DataTable
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'visit_datetime', label: 'Время визита' },
			{ key: 'milk_amount', label: 'Надой, л', align: 'right' },
			{ key: 'duration_seconds', label: 'Длительность, с', align: 'right' },
			{ key: 'milk_destination', label: 'Назначение', align: 'right' },
		]}
		{loading}
	>
		{#if visits.length === 0}
			<tr
				><td colspan="6" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
					>Нет данных</td
				></tr
			>
		{:else}
			{#each visits as v (v.id)}
				<tr
					class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
				>
					<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{v.id}</td>
					<td class="px-4 py-3"
						><a
							href="/animals/{v.animal_id}"
							class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
							>#{v.animal_id}</a
						></td
					>
					<td class="px-4 py-3 text-slate-600 dark:text-slate-400"
						>{formatDatetime(v.visit_datetime)}</td
					>
					<td class="px-4 py-3 text-right font-medium">{fmtNum(v.milk_amount)}</td>
					<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
						>{v.duration_seconds ?? '—'}</td
					>
					<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
						>{v.milk_destination ?? '—'}</td
					>
				</tr>
			{/each}
		{/if}
	</DataTable>
{:else}
	<DataTable
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'date', label: 'Дата' },
			{ key: 'milk_amount', label: 'Надой, л', align: 'right' },
			{ key: 'fat_percentage', label: 'Жир, %', align: 'right' },
			{ key: 'protein_percentage', label: 'Белок, %', align: 'right' },
			{ key: 'lactose_percentage', label: 'Лактоза, %', align: 'right' },
			{ key: 'scc', label: 'СОК', align: 'right' },
			{ key: 'milkings', label: 'Доек', align: 'right' },
			{ key: 'refusals', label: 'Отказов', align: 'right' },
		]}
		{loading}
	>
		{#if quality.length === 0}
			<tr
				><td colspan="10" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
					>Нет данных</td
				></tr
			>
		{:else}
			{#each quality as q (q.id)}
				<tr
					class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
				>
					<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{q.id}</td>
					<td class="px-4 py-3"
						><a
							href="/animals/{q.animal_id}"
							class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
							>#{q.animal_id}</a
						></td
					>
					<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{q.date}</td>
					<td class="px-4 py-3 text-right font-medium">{fmtNum(q.milk_amount)}</td>
					<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
						>{fmtNum(q.fat_percentage)}</td
					>
					<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
						>{fmtNum(q.protein_percentage)}</td
					>
					<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
						>{fmtNum(q.lactose_percentage)}</td
					>
					<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400">{q.scc ?? '—'}</td>
					<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
						>{q.milkings ?? '—'}</td
					>
					<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
						>{q.refusals ?? '—'}</td
					>
				</tr>
			{/each}
		{/if}
	</DataTable>
{/if}

<Pagination bind:page {total} {perPage} />

<Modal
	open={showModal}
	title={modalMode === 'create' ? 'Новый дневной надой' : 'Редактировать надой'}
	onclose={() => (showModal = false)}
>
	<ErrorAlert message={modalError} />
	<form onsubmit={handleSubmit} class="space-y-4">
		{#if modalMode === 'create'}
			<FormField
				id="c-animal"
				label="ID животного"
				type="number"
				bind:value={createForm.animal_id}
				required
				error={v.getError('animal_id')}
			/>
			<FormField
				id="c-date"
				label="Дата"
				type="date"
				bind:value={createForm.date}
				required
				error={v.getError('date')}
			/>
			<FormField
				id="c-amount"
				label="Надой, л"
				type="number"
				bind:value={createForm.milk_amount}
				step="0.1"
			/>
			<FormField
				id="c-avg"
				label="Средний надой"
				type="number"
				bind:value={createForm.avg_amount}
				step="0.1"
			/>
			<FormField
				id="c-weight"
				label="Средний вес"
				type="number"
				bind:value={createForm.avg_weight}
				step="0.1"
			/>
			<FormField id="c-isk" label="ИСК" type="number" bind:value={createForm.isk} step="0.01" />
		{:else}
			<FormField
				id="e-date"
				label="Дата"
				type="date"
				bind:value={editForm.date}
				required
				error={v.getError('date')}
			/>
			<FormField
				id="e-amount"
				label="Надой, л"
				type="number"
				bind:value={editForm.milk_amount}
				step="0.1"
			/>
			<FormField
				id="e-avg"
				label="Средний надой"
				type="number"
				bind:value={editForm.avg_amount}
				step="0.1"
			/>
			<FormField
				id="e-weight"
				label="Средний вес"
				type="number"
				bind:value={editForm.avg_weight}
				step="0.1"
			/>
			<FormField id="e-isk" label="ИСК" type="number" bind:value={editForm.isk} step="0.01" />
		{/if}
		<div class="flex gap-3 justify-end pt-2">
			<button
				type="button"
				onclick={() => (showModal = false)}
				class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 cursor-pointer"
				>Отмена</button
			>
			<button
				type="submit"
				disabled={modalLoading}
				class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50 cursor-pointer"
			>
				{modalLoading ? 'Сохранение...' : modalMode === 'create' ? 'Создать' : 'Сохранить'}
			</button>
		</div>
	</form>
</Modal>

<ConfirmDialog
	open={showDelete}
	title="Удалить надой?"
	message="Это действие нельзя отменить."
	loading={deleteLoading}
	onconfirm={handleDelete}
	oncancel={() => (showDelete = false)}
/>
