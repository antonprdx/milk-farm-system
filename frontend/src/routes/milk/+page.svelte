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
	import AnimalSelect from '$lib/components/ui/AnimalSelect.svelte';
	import MilkChart from '$lib/components/MilkChart.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { fmtNum, formatDatetime } from '$lib/utils/format';
	import { useCrudModal } from '$lib/utils/useCrudModal.svelte';
	import { usePaginatedList } from '$lib/utils/usePaginatedList.svelte';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';
	import { Pencil, Trash2 } from 'lucide-svelte';

	type Tab = 'productions' | 'visits' | 'quality';

	let tab = $state<Tab>('productions');

	let productions = $state<MilkDayProduction[]>([]);
	let visits = $state<MilkVisit[]>([]);
	let quality = $state<MilkQuality[]>([]);

	const list = usePaginatedList();
	const crud = useCrudModal();

	let today = new Date().toISOString().slice(0, 10);

	let createForm = $state<CreateMilkDayProduction>({ animal_id: 0, date: today });
	let editForm = $state({ date: '', milk_amount: '', avg_amount: '', avg_weight: '', isk: '' });

	let dtProductions: DataTable | undefined = $state();
	let dtVisits: DataTable | undefined = $state();
	let dtQuality: DataTable | undefined = $state();

	const v = useFormValidation();

	async function load() {
		const params = {
			animal_id: list.animalId || undefined,
			from_date: list.fromDate || undefined,
			till_date: list.tillDate || undefined,
			page: list.page,
			per_page: list.perPage,
		};
		if (tab === 'productions') {
			await list.load(
				() => listProductions(params),
				(data) => {
					productions = data;
				},
				dtProductions,
			);
		} else if (tab === 'visits') {
			await list.load(
				() => listVisits(params),
				(data) => {
					visits = data;
				},
				dtVisits,
			);
		} else {
			await list.load(
				() => listQuality(params),
				(data) => {
					quality = data;
				},
				dtQuality,
			);
		}
	}

	function switchTab(t: Tab) {
		tab = t;
		list.resetPage();
		load();
	}

	function openCreate() {
		createForm = { animal_id: 0, date: today };
		v.clear();
		crud.openCreate();
	}

	function openEdit(p: MilkDayProduction) {
		editForm = {
			date: p.date,
			milk_amount: p.milk_amount != null ? String(p.milk_amount) : '',
			avg_amount: p.avg_amount != null ? String(p.avg_amount) : '',
			avg_weight: p.avg_weight != null ? String(p.avg_weight) : '',
			isk: p.isk != null ? String(p.isk) : '',
		};
		v.clear();
		crud.openEdit(p.id);
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (crud.modalMode === 'create') {
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
		await crud.submit(
			() =>
				crud.modalMode === 'create'
					? createProduction(createForm)
					: updateProduction(crud.editingId, {
							date: editForm.date || undefined,
							milk_amount: editForm.milk_amount ? Number(editForm.milk_amount) : undefined,
							avg_amount: editForm.avg_amount ? Number(editForm.avg_amount) : undefined,
							avg_weight: editForm.avg_weight ? Number(editForm.avg_weight) : undefined,
							isk: editForm.isk ? Number(editForm.isk) : undefined,
						} as UpdateMilkDayProduction),
			crud.modalMode === 'create' ? 'Надой создан' : 'Надой обновлён',
			load,
		);
	}

	async function handleDelete() {
		await crud.remove(() => deleteProduction(crud.deleteId), load, (msg) => {
			list.error = msg;
		});
	}

	$effect(() => {
		list.page;
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

<FilterBar bind:fromDate={list.fromDate} bind:tillDate={list.tillDate} bind:animalId={list.animalId} onsearch={load} />

{#if tab === 'productions' && productions.length > 0}
	<MilkChart {productions} />
{/if}

<ErrorAlert message={list.error} />

{#if tab === 'productions'}
	<DataTable
		bind:this={dtProductions}
		emptyText="Нет данных"
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
		loading={list.loading}
	>
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
						onclick={() => crud.confirmDelete(p.id)}
						class="text-red-500 hover:text-red-700 text-sm cursor-pointer"
						><Trash2 size={14} /></button
					>
				</td>
			</tr>
		{/each}
	</DataTable>
{:else if tab === 'visits'}
	<DataTable
		bind:this={dtVisits}
		emptyText="Нет данных"
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'visit_datetime', label: 'Время визита' },
			{ key: 'milk_amount', label: 'Надой, л', align: 'right' },
			{ key: 'duration_seconds', label: 'Длительность, с', align: 'right' },
			{ key: 'milk_destination', label: 'Назначение', align: 'right' },
		]}
		loading={list.loading}
	>
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
	</DataTable>
{:else}
	<DataTable
		bind:this={dtQuality}
		emptyText="Нет данных"
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
		loading={list.loading}
	>
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
	</DataTable>
{/if}

<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />

<Modal
	open={crud.showModal}
	title={crud.modalMode === 'create' ? 'Новый дневной надой' : 'Редактировать надой'}
	onclose={crud.close}
>
	<ErrorAlert message={crud.modalError} />
	<form onsubmit={handleSubmit} class="space-y-4">
		{#if crud.modalMode === 'create'}
			<AnimalSelect
				id="c-animal"
				label="Животное"
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
				onclick={crud.close}
				class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 cursor-pointer"
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

<ConfirmDialog
	open={crud.showDelete}
	title="Удалить надой?"
	message="Это действие нельзя отменить."
	loading={crud.deleteLoading}
	onconfirm={handleDelete}
	oncancel={crud.closeDelete}
/>
