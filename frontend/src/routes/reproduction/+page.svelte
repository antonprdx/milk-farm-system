<script lang="ts">
	import {
		listCalvings,
		createCalving,
		updateCalving,
		deleteCalving,
		listInseminations,
		createInsemination,
		updateInsemination,
		deleteInsemination,
		listPregnancies,
		createPregnancy,
		updatePregnancy,
		deletePregnancy,
		listHeats,
		createHeat,
		updateHeat,
		deleteHeat,
		listDryOffs,
		createDryOff,
		updateDryOff,
		deleteDryOff,
		type Calving,
		type Insemination,
		type Pregnancy,
		type Heat,
		type DryOff,
	} from '$lib/api/reproduction';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import FilterBar from '$lib/components/ui/FilterBar.svelte';
	import TabBar from '$lib/components/ui/TabBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import AnimalSelect from '$lib/components/ui/AnimalSelect.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { useCrudModal } from '$lib/utils/useCrudModal.svelte';
	import { usePaginatedList } from '$lib/utils/usePaginatedList.svelte';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';
	import { Pencil, Trash2 } from 'lucide-svelte';

	type Tab = 'calvings' | 'inseminations' | 'pregnancies' | 'heats' | 'dryoffs';

	let { data } = $props();

	let tab = $state<Tab>('calvings');

	let dtCalvings: DataTable | undefined = $state();
	let dtInseminations: DataTable | undefined = $state();
	let dtPregnancies: DataTable | undefined = $state();
	let dtHeats: DataTable | undefined = $state();
	let dtDryoffs: DataTable | undefined = $state();
	let calvings = $state<Calving[]>([]);
	let inseminations = $state<Insemination[]>([]);
	let pregnancies = $state<Pregnancy[]>([]);
	let heats = $state<Heat[]>([]);
	let dryoffs = $state<DryOff[]>([]);

	const list = usePaginatedList();
	const crud = useCrudModal();

	let _skipLoad = !!data.initialData;
	let _hasInitial = $state(!!data.initialData);

	if (data.initialData) {
		calvings = data.initialData.data;
	}

	let today = new Date().toISOString().slice(0, 10);

	let calvingForm = $state({
		animal_id: undefined as number | undefined,
		calving_date: today,
		remarks: '',
		lac_number: '',
	});
	let inseminationForm = $state({
		animal_id: undefined as number | undefined,
		insemination_date: today,
		sire_code: '',
		insemination_type: '',
		charge_number: '',
	});
	let pregnancyForm = $state({
		animal_id: undefined as number | undefined,
		pregnancy_date: today,
		pregnancy_type: '',
		insemination_date: '',
	});
	let heatForm = $state({ animal_id: undefined as number | undefined, heat_date: today });
	let dryoffForm = $state({ animal_id: undefined as number | undefined, dry_off_date: today });

	let editCalvingForm = $state({ calving_date: '', remarks: '', lac_number: '' });
	let editInseminationForm = $state({
		insemination_date: '',
		sire_code: '',
		insemination_type: '',
		charge_number: '',
	});
	let editPregnancyForm = $state({ pregnancy_date: '', pregnancy_type: '', insemination_date: '' });
	let editHeatForm = $state({ heat_date: '' });
	let editDryoffForm = $state({ dry_off_date: '' });

	const v = useFormValidation();

	const tabConfig: { key: Tab; label: string }[] = [
		{ key: 'calvings', label: 'Отёлы' },
		{ key: 'inseminations', label: 'Осеменения' },
		{ key: 'pregnancies', label: 'Стельности' },
		{ key: 'heats', label: 'Охота' },
		{ key: 'dryoffs', label: 'Запуски' },
	];

	const modalTitle = $derived(
		tab === 'calvings'
			? crud.modalMode === 'create'
				? 'Новый отёл'
				: 'Редактировать отёл'
			: tab === 'inseminations'
				? crud.modalMode === 'create'
					? 'Новое осеменение'
					: 'Редактировать осеменение'
				: tab === 'pregnancies'
					? crud.modalMode === 'create'
						? 'Новая стельность'
						: 'Редактировать стельность'
					: tab === 'heats'
						? crud.modalMode === 'create'
							? 'Новая охота'
							: 'Редактировать охоту'
						: crud.modalMode === 'create'
							? 'Новый запуск'
							: 'Редактировать запуск',
	);

	function getFilter() {
		return {
			animal_id: list.animalId || undefined,
			from_date: list.fromDate || undefined,
			till_date: list.tillDate || undefined,
		};
	}

	async function load() {
		const params = { ...getFilter(), page: list.page, per_page: list.perPage };
		switch (tab) {
			case 'calvings':
				await list.load(
					(signal) => listCalvings(params, signal),
					(d) => {
						calvings = d;
					},
					dtCalvings,
				);
				break;
			case 'inseminations':
				await list.load(
					(signal) => listInseminations(params, signal),
					(d) => {
						inseminations = d;
					},
					dtInseminations,
				);
				break;
			case 'pregnancies':
				await list.load(
					(signal) => listPregnancies(params, signal),
					(d) => {
						pregnancies = d;
					},
					dtPregnancies,
				);
				break;
			case 'heats':
				await list.load(
					(signal) => listHeats(params, signal),
					(d) => {
						heats = d;
					},
					dtHeats,
				);
				break;
			case 'dryoffs':
				await list.load(
					(signal) => listDryOffs(params, signal),
					(d) => {
						dryoffs = d;
					},
					dtDryoffs,
				);
				break;
		}
	}

	function switchTab(t: Tab) {
		tab = t;
		list.resetPage();
		load();
	}

	function resetCreateForms() {
		calvingForm = { animal_id: undefined, calving_date: today, remarks: '', lac_number: '' };
		inseminationForm = {
			animal_id: undefined,
			insemination_date: today,
			sire_code: '',
			insemination_type: '',
			charge_number: '',
		};
		pregnancyForm = {
			animal_id: undefined,
			pregnancy_date: today,
			pregnancy_type: '',
			insemination_date: '',
		};
		heatForm = { animal_id: undefined, heat_date: today };
		dryoffForm = { animal_id: undefined, dry_off_date: today };
	}

	function openCreate() {
		v.clear();
		resetCreateForms();
		crud.openCreate();
	}

	function openEditCalving(c: Calving) {
		editCalvingForm = {
			calving_date: c.calving_date,
			remarks: c.remarks ?? '',
			lac_number: c.lac_number != null ? String(c.lac_number) : '',
		};
		v.clear();
		crud.openEdit(c.id);
	}
	function openEditInsemination(i: Insemination) {
		editInseminationForm = {
			insemination_date: i.insemination_date,
			sire_code: i.sire_code ?? '',
			insemination_type: i.insemination_type ?? '',
			charge_number: i.charge_number ?? '',
		};
		v.clear();
		crud.openEdit(i.id);
	}
	function openEditPregnancy(p: Pregnancy) {
		editPregnancyForm = {
			pregnancy_date: p.pregnancy_date,
			pregnancy_type: p.pregnancy_type ?? '',
			insemination_date: p.insemination_date ?? '',
		};
		v.clear();
		crud.openEdit(p.id);
	}
	function openEditHeat(h: Heat) {
		editHeatForm = { heat_date: h.heat_date };
		v.clear();
		crud.openEdit(h.id);
	}
	function openEditDryoff(d: DryOff) {
		editDryoffForm = { dry_off_date: d.dry_off_date };
		v.clear();
		crud.openEdit(d.id);
	}

	async function handleCreateSubmit() {
		switch (tab) {
			case 'calvings':
				await createCalving({
					animal_id: calvingForm.animal_id!,
					calving_date: calvingForm.calving_date,
					remarks: calvingForm.remarks || undefined,
					lac_number: calvingForm.lac_number ? Number(calvingForm.lac_number) : undefined,
				});
				break;
			case 'inseminations':
				await createInsemination({
					animal_id: inseminationForm.animal_id!,
					insemination_date: inseminationForm.insemination_date,
					sire_code: inseminationForm.sire_code || undefined,
					insemination_type: inseminationForm.insemination_type || undefined,
					charge_number: inseminationForm.charge_number || undefined,
				});
				break;
			case 'pregnancies':
				await createPregnancy({
					animal_id: pregnancyForm.animal_id!,
					pregnancy_date: pregnancyForm.pregnancy_date,
					pregnancy_type: pregnancyForm.pregnancy_type || undefined,
					insemination_date: pregnancyForm.insemination_date || undefined,
				});
				break;
			case 'heats':
				await createHeat({ animal_id: heatForm.animal_id!, heat_date: heatForm.heat_date });
				break;
			case 'dryoffs':
				await createDryOff({
					animal_id: dryoffForm.animal_id!,
					dry_off_date: dryoffForm.dry_off_date,
				});
				break;
		}
	}

	async function handleEditSubmit() {
		switch (tab) {
			case 'calvings':
				await updateCalving(crud.editingId, {
					calving_date: editCalvingForm.calving_date || undefined,
					remarks: editCalvingForm.remarks || undefined,
					lac_number: editCalvingForm.lac_number ? Number(editCalvingForm.lac_number) : undefined,
				});
				break;
			case 'inseminations':
				await updateInsemination(crud.editingId, {
					insemination_date: editInseminationForm.insemination_date || undefined,
					sire_code: editInseminationForm.sire_code || undefined,
					insemination_type: editInseminationForm.insemination_type || undefined,
					charge_number: editInseminationForm.charge_number || undefined,
				});
				break;
			case 'pregnancies':
				await updatePregnancy(crud.editingId, {
					pregnancy_date: editPregnancyForm.pregnancy_date || undefined,
					pregnancy_type: editPregnancyForm.pregnancy_type || undefined,
					insemination_date: editPregnancyForm.insemination_date || undefined,
				});
				break;
			case 'heats':
				await updateHeat(crud.editingId, { heat_date: editHeatForm.heat_date || undefined });
				break;
			case 'dryoffs':
				await updateDryOff(crud.editingId, {
					dry_off_date: editDryoffForm.dry_off_date || undefined,
				});
				break;
		}
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		const req = [rules.required()];
		let valid = true;
		if (crud.modalMode === 'create') {
			switch (tab) {
				case 'calvings':
					valid = v.validateAll({
						animal_id: { value: calvingForm.animal_id, rules: req },
						calving_date: { value: calvingForm.calving_date, rules: req },
					});
					break;
				case 'inseminations':
					valid = v.validateAll({
						animal_id: { value: inseminationForm.animal_id, rules: req },
						insemination_date: { value: inseminationForm.insemination_date, rules: req },
					});
					break;
				case 'pregnancies':
					valid = v.validateAll({
						animal_id: { value: pregnancyForm.animal_id, rules: req },
						pregnancy_date: { value: pregnancyForm.pregnancy_date, rules: req },
					});
					break;
				case 'heats':
					valid = v.validateAll({
						animal_id: { value: heatForm.animal_id, rules: req },
						heat_date: { value: heatForm.heat_date, rules: req },
					});
					break;
				case 'dryoffs':
					valid = v.validateAll({
						animal_id: { value: dryoffForm.animal_id, rules: req },
						dry_off_date: { value: dryoffForm.dry_off_date, rules: req },
					});
					break;
			}
		} else {
			switch (tab) {
				case 'calvings':
					valid = v.validateAll({
						calving_date: { value: editCalvingForm.calving_date, rules: req },
					});
					break;
				case 'inseminations':
					valid = v.validateAll({
						insemination_date: { value: editInseminationForm.insemination_date, rules: req },
					});
					break;
				case 'pregnancies':
					valid = v.validateAll({
						pregnancy_date: { value: editPregnancyForm.pregnancy_date, rules: req },
					});
					break;
				case 'heats':
					valid = v.validateAll({ heat_date: { value: editHeatForm.heat_date, rules: req } });
					break;
				case 'dryoffs':
					valid = v.validateAll({
						dry_off_date: { value: editDryoffForm.dry_off_date, rules: req },
					});
					break;
			}
		}
		if (!valid) return;
		await crud.submit(
			() => (crud.modalMode === 'create' ? handleCreateSubmit() : handleEditSubmit()),
			crud.modalMode === 'create' ? 'Запись создана' : 'Запись обновлена',
			load,
		);
	}

	async function handleDelete() {
		const deleteFn = () => {
			switch (tab) {
				case 'calvings':
					return deleteCalving(crud.deleteId);
				case 'inseminations':
					return deleteInsemination(crud.deleteId);
				case 'pregnancies':
					return deletePregnancy(crud.deleteId);
				case 'heats':
					return deleteHeat(crud.deleteId);
				case 'dryoffs':
					return deleteDryOff(crud.deleteId);
			}
		};
		await crud.remove(deleteFn, load, (msg) => {
			list.error = msg;
		});
	}

	$effect(() => {
		if (_skipLoad) {
			_skipLoad = false;
			return;
		}
		_hasInitial = false;
		list.page;
		tab;
		load();
	});
</script>

<svelte:head>
	<title>Воспроизводство — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Воспроизводство</h1>
	<button
		onclick={openCreate}
		class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
	>
		+ Добавить
	</button>
</div>

<TabBar tabs={tabConfig} bind:active={tab} onchange={(t: string) => switchTab(t as Tab)} />
<FilterBar
	bind:fromDate={list.fromDate}
	bind:tillDate={list.tillDate}
	bind:animalId={list.animalId}
	onsearch={load}
/>
<ErrorAlert message={data.error} />
<ErrorAlert message={list.error} />

{#if tab === 'calvings'}
	<DataTable
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'calving_date', label: 'Дата отёла' },
			{ key: 'remarks', label: 'Примечания' },
			{ key: 'lac_number', label: 'Лактация', align: 'right' },
			{ key: 'actions', label: '', align: 'right' },
		]}
		loading={list.loading && !_hasInitial}
		initialRows={!!data.initialData && data.initialData.data.length > 0}
		bind:this={dtCalvings}
		emptyText="Нет данных"
	>
		{#each calvings as c (`calving-${c.id}`)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{c.id}</td>
				<td class="px-4 py-3"
					><a
						href="/animals/{c.animal_id}"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
						>#{c.animal_id}</a
					></td
				>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{c.calving_date}</td>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{c.remarks || '—'}</td>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{c.lac_number ?? '—'}</td
				>
				<td class="px-4 py-3 text-right">
					<button
						onclick={() => openEditCalving(c)}
						aria-label="Редактировать"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
						><Pencil size={14} /></button
					>
					<button
						onclick={() => crud.confirmDelete(c.id)}
						aria-label="Удалить"
						class="text-red-500 hover:text-red-700 text-sm cursor-pointer"
						><Trash2 size={14} /></button
					>
				</td>
			</tr>
		{/each}
	</DataTable>
{:else if tab === 'inseminations'}
	<DataTable
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'insemination_date', label: 'Дата осеменения' },
			{ key: 'sire_code', label: 'Код быка' },
			{ key: 'insemination_type', label: 'Тип' },
			{ key: 'charge_number', label: 'Партия' },
			{ key: 'actions', label: '', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtInseminations}
		emptyText="Нет данных"
	>
		{#each inseminations as i (`insem-${i.id}`)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{i.id}</td>
				<td class="px-4 py-3"
					><a
						href="/animals/{i.animal_id}"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
						>#{i.animal_id}</a
					></td
				>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{i.insemination_date}</td>
				<td class="px-4 py-3 font-mono text-slate-600 dark:text-slate-400">{i.sire_code || '—'}</td>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{i.insemination_type || '—'}</td>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{i.charge_number || '—'}</td>
				<td class="px-4 py-3 text-right">
					<button
						onclick={() => openEditInsemination(i)}
						aria-label="Редактировать"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
						><Pencil size={14} /></button
					>
					<button
						onclick={() => crud.confirmDelete(i.id)}
						aria-label="Удалить"
						class="text-red-500 hover:text-red-700 text-sm cursor-pointer"
						><Trash2 size={14} /></button
					>
				</td>
			</tr>
		{/each}
	</DataTable>
{:else if tab === 'pregnancies'}
	<DataTable
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'pregnancy_date', label: 'Дата проверки' },
			{ key: 'pregnancy_type', label: 'Тип' },
			{ key: 'insemination_date', label: 'Дата осеменения' },
			{ key: 'actions', label: '', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtPregnancies}
		emptyText="Нет данных"
	>
		{#each pregnancies as p (`preg-${p.id}`)}
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
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{p.pregnancy_date}</td>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{p.pregnancy_type || '—'}</td>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{p.insemination_date || '—'}</td>
				<td class="px-4 py-3 text-right">
					<button
						onclick={() => openEditPregnancy(p)}
						aria-label="Редактировать"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
						><Pencil size={14} /></button
					>
					<button
						onclick={() => crud.confirmDelete(p.id)}
						aria-label="Удалить"
						class="text-red-500 hover:text-red-700 text-sm cursor-pointer"
						><Trash2 size={14} /></button
					>
				</td>
			</tr>
		{/each}
	</DataTable>
{:else if tab === 'heats'}
	<DataTable
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'heat_date', label: 'Дата охоты' },
			{ key: 'actions', label: '', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtHeats}
		emptyText="Нет данных"
	>
		{#each heats as h (`heat-${h.id}`)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{h.id}</td>
				<td class="px-4 py-3"
					><a
						href="/animals/{h.animal_id}"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
						>#{h.animal_id}</a
					></td
				>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{h.heat_date}</td>
				<td class="px-4 py-3 text-right">
					<button
						onclick={() => openEditHeat(h)}
						aria-label="Редактировать"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
						><Pencil size={14} /></button
					>
					<button
						onclick={() => crud.confirmDelete(h.id)}
						aria-label="Удалить"
						class="text-red-500 hover:text-red-700 text-sm cursor-pointer"
						><Trash2 size={14} /></button
					>
				</td>
			</tr>
		{/each}
	</DataTable>
{:else}
	<DataTable
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'dry_off_date', label: 'Дата запуска' },
			{ key: 'actions', label: '', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtDryoffs}
		emptyText="Нет данных"
	>
		{#each dryoffs as d (`dryoff-${d.id}`)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{d.id}</td>
				<td class="px-4 py-3"
					><a
						href="/animals/{d.animal_id}"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
						>#{d.animal_id}</a
					></td
				>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{d.dry_off_date}</td>
				<td class="px-4 py-3 text-right">
					<button
						onclick={() => openEditDryoff(d)}
						aria-label="Редактировать"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
						><Pencil size={14} /></button
					>
					<button
						onclick={() => crud.confirmDelete(d.id)}
						aria-label="Удалить"
						class="text-red-500 hover:text-red-700 text-sm cursor-pointer"
						><Trash2 size={14} /></button
					>
				</td>
			</tr>
		{/each}
	</DataTable>
{/if}

<Modal open={crud.showModal} title={modalTitle} onclose={crud.close}>
	<ErrorAlert message={crud.modalError} />
	<form onsubmit={handleSubmit} class="space-y-4">
		{#if crud.modalMode === 'create'}
			{#if tab === 'calvings'}
				<AnimalSelect
					id="m-animal"
					label="Животное"
					bind:value={calvingForm.animal_id}
					required
					error={v.getError('animal_id')}
				/>
				<FormField
					id="m-date"
					label="Дата отёла"
					type="date"
					bind:value={calvingForm.calving_date}
					required
					error={v.getError('calving_date')}
				/>
				<FormField id="m-remarks" label="Примечания" bind:value={calvingForm.remarks} />
				<FormField
					id="m-lac"
					label="Номер лактации"
					type="number"
					bind:value={calvingForm.lac_number}
				/>
			{:else if tab === 'inseminations'}
				<AnimalSelect
					id="m-animal"
					label="Животное"
					bind:value={inseminationForm.animal_id}
					required
					error={v.getError('animal_id')}
				/>
				<FormField
					id="m-date"
					label="Дата осеменения"
					type="date"
					bind:value={inseminationForm.insemination_date}
					required
					error={v.getError('insemination_date')}
				/>
				<FormField id="m-sire" label="Код быка" bind:value={inseminationForm.sire_code} />
				<FormField
					id="m-type"
					label="Тип осеменения"
					bind:value={inseminationForm.insemination_type}
					placeholder="Искусственное / Естественное"
				/>
				<FormField id="m-charge" label="Номер партии" bind:value={inseminationForm.charge_number} />
			{:else if tab === 'pregnancies'}
				<AnimalSelect
					id="m-animal"
					label="Животное"
					bind:value={pregnancyForm.animal_id}
					required
					error={v.getError('animal_id')}
				/>
				<FormField
					id="m-date"
					label="Дата проверки"
					type="date"
					bind:value={pregnancyForm.pregnancy_date}
					required
					error={v.getError('pregnancy_date')}
				/>
				<FormField id="m-type" label="Тип" bind:value={pregnancyForm.pregnancy_type} />
				<FormField
					id="m-insdate"
					label="Дата осеменения"
					type="date"
					bind:value={pregnancyForm.insemination_date}
				/>
			{:else if tab === 'heats'}
				<AnimalSelect
					id="m-animal"
					label="Животное"
					bind:value={heatForm.animal_id}
					required
					error={v.getError('animal_id')}
				/>
				<FormField
					id="m-date"
					label="Дата охоты"
					type="date"
					bind:value={heatForm.heat_date}
					required
					error={v.getError('heat_date')}
				/>
			{:else}
				<AnimalSelect
					id="m-animal"
					label="Животное"
					bind:value={dryoffForm.animal_id}
					required
					error={v.getError('animal_id')}
				/>
				<FormField
					id="m-date"
					label="Дата запуска"
					type="date"
					bind:value={dryoffForm.dry_off_date}
					required
					error={v.getError('dry_off_date')}
				/>
			{/if}
		{:else if tab === 'calvings'}
			<FormField
				id="e-date"
				label="Дата отёла"
				type="date"
				bind:value={editCalvingForm.calving_date}
				required
				error={v.getError('calving_date')}
			/>
			<FormField id="e-remarks" label="Примечания" bind:value={editCalvingForm.remarks} />
			<FormField
				id="e-lac"
				label="Номер лактации"
				type="number"
				bind:value={editCalvingForm.lac_number}
			/>
		{:else if tab === 'inseminations'}
			<FormField
				id="e-date"
				label="Дата осеменения"
				type="date"
				bind:value={editInseminationForm.insemination_date}
				required
				error={v.getError('insemination_date')}
			/>
			<FormField id="e-sire" label="Код быка" bind:value={editInseminationForm.sire_code} />
			<FormField
				id="e-type"
				label="Тип осеменения"
				bind:value={editInseminationForm.insemination_type}
			/>
			<FormField
				id="e-charge"
				label="Номер партии"
				bind:value={editInseminationForm.charge_number}
			/>
		{:else if tab === 'pregnancies'}
			<FormField
				id="e-date"
				label="Дата проверки"
				type="date"
				bind:value={editPregnancyForm.pregnancy_date}
				required
				error={v.getError('pregnancy_date')}
			/>
			<FormField id="e-type" label="Тип" bind:value={editPregnancyForm.pregnancy_type} />
			<FormField
				id="e-insdate"
				label="Дата осеменения"
				type="date"
				bind:value={editPregnancyForm.insemination_date}
			/>
		{:else if tab === 'heats'}
			<FormField
				id="e-date"
				label="Дата охоты"
				type="date"
				bind:value={editHeatForm.heat_date}
				required
				error={v.getError('heat_date')}
			/>
		{:else}
			<FormField
				id="e-date"
				label="Дата запуска"
				type="date"
				bind:value={editDryoffForm.dry_off_date}
				required
				error={v.getError('dry_off_date')}
			/>
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
