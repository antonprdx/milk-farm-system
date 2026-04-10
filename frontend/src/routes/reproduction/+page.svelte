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
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { toasts } from '$lib/stores/toast';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';
	import { Pencil, Trash2 } from 'lucide-svelte';

	type Tab = 'calvings' | 'inseminations' | 'pregnancies' | 'heats' | 'dryoffs';
	type ModalMode = 'create' | 'edit';

	let tab = $state<Tab>('calvings');
	let loading = $state(true);
	let error = $state('');

	let calvings = $state<Calving[]>([]);
	let inseminations = $state<Insemination[]>([]);
	let pregnancies = $state<Pregnancy[]>([]);
	let heats = $state<Heat[]>([]);
	let dryoffs = $state<DryOff[]>([]);

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

	let total = $state(0);
	let page = $state(1);
	let perPage = 20;

	let today = new Date().toISOString().slice(0, 10);

	let calvingForm = $state({ animal_id: '', calving_date: today, remarks: '', lac_number: '' });
	let inseminationForm = $state({
		animal_id: '',
		insemination_date: today,
		sire_code: '',
		insemination_type: '',
		charge_number: '',
	});
	let pregnancyForm = $state({
		animal_id: '',
		pregnancy_date: today,
		pregnancy_type: '',
		insemination_date: '',
	});
	let heatForm = $state({ animal_id: '', heat_date: today });
	let dryoffForm = $state({ animal_id: '', dry_off_date: today });

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
			? modalMode === 'create'
				? 'Новый отёл'
				: 'Редактировать отёл'
			: tab === 'inseminations'
				? modalMode === 'create'
					? 'Новое осеменение'
					: 'Редактировать осеменение'
				: tab === 'pregnancies'
					? modalMode === 'create'
						? 'Новая стельность'
						: 'Редактировать стельность'
					: tab === 'heats'
						? modalMode === 'create'
							? 'Новая охота'
							: 'Редактировать охоту'
						: modalMode === 'create'
							? 'Новый запуск'
							: 'Редактировать запуск',
	);

	function getFilter() {
		return {
			animal_id: animalId ? Number(animalId) : undefined,
			from_date: fromDate || undefined,
			till_date: tillDate || undefined,
		};
	}

	async function load() {
		try {
			loading = true;
			error = '';
			const filter = { ...getFilter(), page, per_page: perPage };
			switch (tab) {
				case 'calvings': {
					const r = await listCalvings(filter);
					calvings = r.data;
					total = r.total;
					break;
				}
				case 'inseminations': {
					const r = await listInseminations(filter);
					inseminations = r.data;
					total = r.total;
					break;
				}
				case 'pregnancies': {
					const r = await listPregnancies(filter);
					pregnancies = r.data;
					total = r.total;
					break;
				}
				case 'heats': {
					const r = await listHeats(filter);
					heats = r.data;
					total = r.total;
					break;
				}
				case 'dryoffs': {
					const r = await listDryOffs(filter);
					dryoffs = r.data;
					total = r.total;
					break;
				}
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
		v.clear();
		resetCreateForms();
		showModal = true;
	}

	function resetCreateForms() {
		calvingForm = { animal_id: '', calving_date: today, remarks: '', lac_number: '' };
		inseminationForm = {
			animal_id: '',
			insemination_date: today,
			sire_code: '',
			insemination_type: '',
			charge_number: '',
		};
		pregnancyForm = {
			animal_id: '',
			pregnancy_date: today,
			pregnancy_type: '',
			insemination_date: '',
		};
		heatForm = { animal_id: '', heat_date: today };
		dryoffForm = { animal_id: '', dry_off_date: today };
	}

	function openEditCalving(c: Calving) {
		modalMode = 'edit';
		editingId = c.id;
		modalError = '';
		v.clear();
		editCalvingForm = {
			calving_date: c.calving_date,
			remarks: c.remarks ?? '',
			lac_number: c.lac_number != null ? String(c.lac_number) : '',
		};
		showModal = true;
	}
	function openEditInsemination(i: Insemination) {
		modalMode = 'edit';
		editingId = i.id;
		modalError = '';
		v.clear();
		editInseminationForm = {
			insemination_date: i.insemination_date,
			sire_code: i.sire_code ?? '',
			insemination_type: i.insemination_type ?? '',
			charge_number: i.charge_number ?? '',
		};
		showModal = true;
	}
	function openEditPregnancy(p: Pregnancy) {
		modalMode = 'edit';
		editingId = p.id;
		modalError = '';
		v.clear();
		editPregnancyForm = {
			pregnancy_date: p.pregnancy_date,
			pregnancy_type: p.pregnancy_type ?? '',
			insemination_date: p.insemination_date ?? '',
		};
		showModal = true;
	}
	function openEditHeat(h: Heat) {
		modalMode = 'edit';
		editingId = h.id;
		modalError = '';
		v.clear();
		editHeatForm = { heat_date: h.heat_date };
		showModal = true;
	}
	function openEditDryoff(d: DryOff) {
		modalMode = 'edit';
		editingId = d.id;
		modalError = '';
		v.clear();
		editDryoffForm = { dry_off_date: d.dry_off_date };
		showModal = true;
	}

	function confirmDelete(id: number) {
		deleteId = id;
		showDelete = true;
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		modalError = '';
		const req = [rules.required()];
		const posId = [rules.required(), rules.positive('Укажите ID животного')];
		let valid = true;
		if (modalMode === 'create') {
			switch (tab) {
				case 'calvings':
					valid = v.validateAll({
						animal_id: { value: calvingForm.animal_id, rules: posId },
						calving_date: { value: calvingForm.calving_date, rules: req },
					});
					break;
				case 'inseminations':
					valid = v.validateAll({
						animal_id: { value: inseminationForm.animal_id, rules: posId },
						insemination_date: { value: inseminationForm.insemination_date, rules: req },
					});
					break;
				case 'pregnancies':
					valid = v.validateAll({
						animal_id: { value: pregnancyForm.animal_id, rules: posId },
						pregnancy_date: { value: pregnancyForm.pregnancy_date, rules: req },
					});
					break;
				case 'heats':
					valid = v.validateAll({
						animal_id: { value: heatForm.animal_id, rules: posId },
						heat_date: { value: heatForm.heat_date, rules: req },
					});
					break;
				case 'dryoffs':
					valid = v.validateAll({
						animal_id: { value: dryoffForm.animal_id, rules: posId },
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
		try {
			modalLoading = true;
			if (modalMode === 'create') {
				await handleCreateSubmit();
			} else {
				await handleEditSubmit();
			}
			showModal = false;
			load();
		} catch (e) {
			modalError = e instanceof Error ? e.message : 'Ошибка';
		} finally {
			modalLoading = false;
		}
	}

	async function handleCreateSubmit() {
		switch (tab) {
			case 'calvings':
				await createCalving({
					animal_id: Number(calvingForm.animal_id),
					calving_date: calvingForm.calving_date,
					remarks: calvingForm.remarks || undefined,
					lac_number: calvingForm.lac_number ? Number(calvingForm.lac_number) : undefined,
				});
				break;
			case 'inseminations':
				await createInsemination({
					animal_id: Number(inseminationForm.animal_id),
					insemination_date: inseminationForm.insemination_date,
					sire_code: inseminationForm.sire_code || undefined,
					insemination_type: inseminationForm.insemination_type || undefined,
					charge_number: inseminationForm.charge_number || undefined,
				});
				break;
			case 'pregnancies':
				await createPregnancy({
					animal_id: Number(pregnancyForm.animal_id),
					pregnancy_date: pregnancyForm.pregnancy_date,
					pregnancy_type: pregnancyForm.pregnancy_type || undefined,
					insemination_date: pregnancyForm.insemination_date || undefined,
				});
				break;
			case 'heats':
				await createHeat({ animal_id: Number(heatForm.animal_id), heat_date: heatForm.heat_date });
				break;
			case 'dryoffs':
				await createDryOff({
					animal_id: Number(dryoffForm.animal_id),
					dry_off_date: dryoffForm.dry_off_date,
				});
				break;
		}
		toasts.success('Запись создана');
	}

	async function handleEditSubmit() {
		switch (tab) {
			case 'calvings':
				await updateCalving(editingId, {
					calving_date: editCalvingForm.calving_date || undefined,
					remarks: editCalvingForm.remarks || undefined,
					lac_number: editCalvingForm.lac_number ? Number(editCalvingForm.lac_number) : undefined,
				});
				break;
			case 'inseminations':
				await updateInsemination(editingId, {
					insemination_date: editInseminationForm.insemination_date || undefined,
					sire_code: editInseminationForm.sire_code || undefined,
					insemination_type: editInseminationForm.insemination_type || undefined,
					charge_number: editInseminationForm.charge_number || undefined,
				});
				break;
			case 'pregnancies':
				await updatePregnancy(editingId, {
					pregnancy_date: editPregnancyForm.pregnancy_date || undefined,
					pregnancy_type: editPregnancyForm.pregnancy_type || undefined,
					insemination_date: editPregnancyForm.insemination_date || undefined,
				});
				break;
			case 'heats':
				await updateHeat(editingId, { heat_date: editHeatForm.heat_date || undefined });
				break;
			case 'dryoffs':
				await updateDryOff(editingId, { dry_off_date: editDryoffForm.dry_off_date || undefined });
				break;
		}
		toasts.success('Запись обновлена');
	}

	async function handleDelete() {
		try {
			deleteLoading = true;
			switch (tab) {
				case 'calvings':
					await deleteCalving(deleteId);
					break;
				case 'inseminations':
					await deleteInsemination(deleteId);
					break;
				case 'pregnancies':
					await deletePregnancy(deleteId);
					break;
				case 'heats':
					await deleteHeat(deleteId);
					break;
				case 'dryoffs':
					await deleteDryOff(deleteId);
					break;
			}
			showDelete = false;
			toasts.success('Запись удалена');
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
<FilterBar bind:fromDate bind:tillDate bind:animalId onsearch={load} />
<ErrorAlert message={error} />

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
		{loading}
	>
		{#if calvings.length === 0}
			<tr
				><td colspan="6" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
					>Нет данных</td
				></tr
			>
		{:else}
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
							class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
							><Pencil size={14} /></button
						>
						<button
							onclick={() => confirmDelete(c.id)}
							class="text-red-500 hover:text-red-700 text-sm cursor-pointer"><Trash2 size={14} /></button
						>
					</td>
				</tr>
			{/each}
		{/if}
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
		{loading}
	>
		{#if inseminations.length === 0}
			<tr
				><td colspan="7" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
					>Нет данных</td
				></tr
			>
		{:else}
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
					<td class="px-4 py-3 font-mono text-slate-600 dark:text-slate-400"
						>{i.sire_code || '—'}</td
					>
					<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{i.insemination_type || '—'}</td>
					<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{i.charge_number || '—'}</td>
					<td class="px-4 py-3 text-right">
						<button
							onclick={() => openEditInsemination(i)}
							class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
							><Pencil size={14} /></button
						>
						<button
							onclick={() => confirmDelete(i.id)}
							class="text-red-500 hover:text-red-700 text-sm cursor-pointer"><Trash2 size={14} /></button
						>
					</td>
				</tr>
			{/each}
		{/if}
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
		{loading}
	>
		{#if pregnancies.length === 0}
			<tr
				><td colspan="6" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
					>Нет данных</td
				></tr
			>
		{:else}
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
							class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
							><Pencil size={14} /></button
						>
						<button
							onclick={() => confirmDelete(p.id)}
							class="text-red-500 hover:text-red-700 text-sm cursor-pointer"><Trash2 size={14} /></button
						>
					</td>
				</tr>
			{/each}
		{/if}
	</DataTable>
{:else if tab === 'heats'}
	<DataTable
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'heat_date', label: 'Дата охоты' },
			{ key: 'actions', label: '', align: 'right' },
		]}
		{loading}
	>
		{#if heats.length === 0}
			<tr
				><td colspan="4" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
					>Нет данных</td
				></tr
			>
		{:else}
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
							class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
							><Pencil size={14} /></button
						>
						<button
							onclick={() => confirmDelete(h.id)}
							class="text-red-500 hover:text-red-700 text-sm cursor-pointer"><Trash2 size={14} /></button
						>
					</td>
				</tr>
			{/each}
		{/if}
	</DataTable>
{:else}
	<DataTable
		columns={[
			{ key: 'id', label: 'ID' },
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'dry_off_date', label: 'Дата запуска' },
			{ key: 'actions', label: '', align: 'right' },
		]}
		{loading}
	>
		{#if dryoffs.length === 0}
			<tr
				><td colspan="4" class="px-4 py-8 text-center text-slate-400 dark:text-slate-500"
					>Нет данных</td
				></tr
			>
		{:else}
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
							class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400 text-sm cursor-pointer mr-1"
							><Pencil size={14} /></button
						>
						<button
							onclick={() => confirmDelete(d.id)}
							class="text-red-500 hover:text-red-700 text-sm cursor-pointer"><Trash2 size={14} /></button
						>
					</td>
				</tr>
			{/each}
		{/if}
	</DataTable>
{/if}

<Modal open={showModal} title={modalTitle} onclose={() => (showModal = false)}>
	<ErrorAlert message={modalError} />
	<form onsubmit={handleSubmit} class="space-y-4">
		{#if modalMode === 'create'}
			{#if tab === 'calvings'}
				<FormField
					id="m-animal"
					label="ID животного"
					type="number"
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
				<FormField
					id="m-animal"
					label="ID животного"
					type="number"
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
				<FormField
					id="m-animal"
					label="ID животного"
					type="number"
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
				<FormField
					id="m-animal"
					label="ID животного"
					type="number"
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
				<FormField
					id="m-animal"
					label="ID животного"
					type="number"
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
	title="Удалить запись?"
	message="Это действие нельзя отменить."
	loading={deleteLoading}
	onconfirm={handleDelete}
	oncancel={() => (showDelete = false)}
/>

<Pagination bind:page {total} {perPage} />
