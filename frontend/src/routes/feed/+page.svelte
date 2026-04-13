<script lang="ts">
	import {
		listDayAmounts,
		listVisits,
		listTypes,
		listGroups,
		createFeedType,
		updateFeedType,
		deleteFeedType,
		createFeedGroup,
		updateFeedGroup,
		deleteFeedGroup,
		type FeedDayAmount,
		type FeedVisit,
		type FeedType,
		type FeedGroup,
		type CreateFeedType,
		type UpdateFeedType,
		type CreateFeedGroup,
		type UpdateFeedGroup,
	} from '$lib/api/feed';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import FilterBar from '$lib/components/ui/FilterBar.svelte';
	import TabBar from '$lib/components/ui/TabBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { formatDatetime } from '$lib/utils/format';
	import { usePaginatedList } from '$lib/utils/usePaginatedList.svelte';
	import { useCrudModal } from '$lib/utils/useCrudModal.svelte';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';
	import { Pencil, Trash2 } from 'lucide-svelte';

	type Tab = 'amounts' | 'visits' | 'types' | 'groups';

	let tab = $state<Tab>('amounts');

	let dtAmounts: DataTable | undefined = $state();
	let dtVisits: DataTable | undefined = $state();
	let dtTypes: DataTable | undefined = $state();
	let dtGroups: DataTable | undefined = $state();
	let amounts = $state<FeedDayAmount[]>([]);
	let visits = $state<FeedVisit[]>([]);
	let types = $state<FeedType[]>([]);
	let groups = $state<FeedGroup[]>([]);

	const list = usePaginatedList({ perPage: 50 });
	const crudType = useCrudModal();
	const crudGroup = useCrudModal();
	const vType = useFormValidation();
	const vGroup = useFormValidation();

	let formType = $state<CreateFeedType>({
		number_of_feed_type: 1,
		feed_type: '',
		name: '',
		dry_matter_percentage: 85,
		price: 0,
	});

	let formGroup = $state<CreateFeedGroup>({
		name: '',
	});

	async function load() {
		const params = {
			animal_id: list.animalId || undefined,
			from_date: list.fromDate || undefined,
			till_date: list.tillDate || undefined,
			page: list.page,
			per_page: list.perPage,
		};
		switch (tab) {
			case 'amounts':
				await list.load(
					(signal) => listDayAmounts(params, signal),
					(d) => {
						amounts = d;
					},
					dtAmounts,
				);
				break;
			case 'visits':
				await list.load(
					(signal) => listVisits(params, signal),
					(d) => {
						visits = d;
					},
					dtVisits,
				);
				break;
			case 'types': {
				try {
					list.error = '';
					const res = await listTypes();
					types = res.data;
					dtTypes?.setHasRows(types.length > 0);
				} catch (e) {
					list.error = e instanceof Error ? e.message : 'Ошибка загрузки';
				}
				break;
			}
			case 'groups': {
				try {
					list.error = '';
					const res = await listGroups();
					groups = res.data;
					dtGroups?.setHasRows(groups.length > 0);
				} catch (e) {
					list.error = e instanceof Error ? e.message : 'Ошибка загрузки';
				}
				break;
			}
		}
	}

	function switchTab(t: Tab) {
		tab = t;
		list.resetPage();
		load();
	}

	function openCreateType() {
		vType.clear();
		formType = {
			number_of_feed_type: 1,
			feed_type: '',
			name: '',
			dry_matter_percentage: 85,
			price: 0,
		};
		crudType.openCreate();
	}

	function openEditType(t: FeedType) {
		vType.clear();
		formType = {
			number_of_feed_type: t.number_of_feed_type,
			feed_type: t.feed_type,
			name: t.name,
			dry_matter_percentage: t.dry_matter_percentage,
			price: t.price,
		};
		crudType.openEdit(t.id);
	}

	async function handleSubmitType(e: Event) {
		e.preventDefault();
		const fields: Record<
			string,
			{ value: string | number | undefined; rules: ReturnType<typeof rules.required>[] }
		> = {
			name: { value: formType.name, rules: [rules.required()] },
			feed_type: { value: formType.feed_type, rules: [rules.required()] },
			dry_matter_percentage: { value: formType.dry_matter_percentage, rules: [rules.percentage()] },
			price: { value: formType.price, rules: [rules.nonNegative()] },
		};
		if (!vType.validateAll(fields)) return;

		const data: CreateFeedType = {
			number_of_feed_type: formType.number_of_feed_type,
			feed_type: formType.feed_type,
			name: formType.name,
			dry_matter_percentage: formType.dry_matter_percentage,
			price: formType.price,
		};
		await crudType.submit(
			() =>
				crudType.modalMode === 'create'
					? createFeedType(data)
					: updateFeedType(crudType.editingId, { ...data } as UpdateFeedType),
			crudType.modalMode === 'create' ? 'Тип корма создан' : 'Тип корма обновлён',
			load,
		);
	}

	async function handleDeleteType() {
		await crudType.remove(
			() => deleteFeedType(crudType.deleteId),
			load,
			(msg) => {
				list.error = msg;
			},
		);
	}

	function openCreateGroup() {
		vGroup.clear();
		formGroup = { name: '' };
		crudGroup.openCreate();
	}

	function openEditGroup(g: FeedGroup) {
		vGroup.clear();
		formGroup = {
			name: g.name,
			min_milk_yield: g.min_milk_yield ?? undefined,
			max_milk_yield: g.max_milk_yield ?? undefined,
			avg_milk_yield: g.avg_milk_yield ?? undefined,
			avg_milk_fat: g.avg_milk_fat ?? undefined,
			avg_milk_protein: g.avg_milk_protein ?? undefined,
			avg_weight: g.avg_weight ?? undefined,
			max_robot_feed_types: g.max_robot_feed_types ?? undefined,
			max_feed_intake_robot: g.max_feed_intake_robot ?? undefined,
			min_feed_intake_robot: g.min_feed_intake_robot ?? undefined,
			number_of_cows: g.number_of_cows ?? undefined,
		};
		crudGroup.openEdit(g.id);
	}

	async function handleSubmitGroup(e: Event) {
		e.preventDefault();
		const fields: Record<
			string,
			{ value: string | number | undefined; rules: ReturnType<typeof rules.required>[] }
		> = {
			name: { value: formGroup.name, rules: [rules.required()] },
		};
		if (!vGroup.validateAll(fields)) return;

		const data: CreateFeedGroup = {
			name: formGroup.name,
			min_milk_yield: formGroup.min_milk_yield || undefined,
			max_milk_yield: formGroup.max_milk_yield || undefined,
			avg_milk_yield: formGroup.avg_milk_yield || undefined,
			avg_milk_fat: formGroup.avg_milk_fat || undefined,
			avg_milk_protein: formGroup.avg_milk_protein || undefined,
			avg_weight: formGroup.avg_weight || undefined,
			max_robot_feed_types: formGroup.max_robot_feed_types || undefined,
			max_feed_intake_robot: formGroup.max_feed_intake_robot || undefined,
			min_feed_intake_robot: formGroup.min_feed_intake_robot || undefined,
			number_of_cows: formGroup.number_of_cows || undefined,
		};
		await crudGroup.submit(
			() =>
				crudGroup.modalMode === 'create'
					? createFeedGroup(data)
					: updateFeedGroup(crudGroup.editingId, { ...data } as UpdateFeedGroup),
			crudGroup.modalMode === 'create' ? 'Группа создана' : 'Группа обновлена',
			load,
		);
	}

	async function handleDeleteGroup() {
		await crudGroup.remove(
			() => deleteFeedGroup(crudGroup.deleteId),
			load,
			(msg) => {
				list.error = msg;
			},
		);
	}

	$effect(() => {
		list.page;
		load();
	});
</script>

<svelte:head>
	<title>Кормление — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-6">Кормление</h1>

<TabBar
	tabs={[
		{ key: 'amounts', label: 'Дневные нормы' },
		{ key: 'visits', label: 'Визиты' },
		{ key: 'types', label: 'Типы кормов' },
		{ key: 'groups', label: 'Группы' },
	]}
	bind:active={tab}
	onchange={(t: string) => switchTab(t as Tab)}
/>

{#if tab === 'amounts' || tab === 'visits'}
	<FilterBar
		bind:fromDate={list.fromDate}
		bind:tillDate={list.tillDate}
		bind:animalId={list.animalId}
		onsearch={load}
	/>
{/if}

<ErrorAlert message={list.error} />

{#if tab === 'amounts'}
	<DataTable
		columns={[
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'feed_date', label: 'Дата' },
			{ key: 'feed_number', label: '№ корма', align: 'right' },
			{ key: 'total', label: 'Всего, кг', align: 'right' },
			{ key: 'rest_feed', label: 'Остаток', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtAmounts}
		emptyText="Нет данных"
	>
		{#each amounts as a, i (i)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3"
					><a
						href="/animals/{a.animal_id}"
						class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
						>#{a.animal_id}</a
					></td
				>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{a.feed_date}</td>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400">{a.feed_number}</td>
				<td class="px-4 py-3 text-right font-medium">{a.total.toFixed(1)}</td>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400">{a.rest_feed ?? '—'}</td
				>
			</tr>
		{/each}
	</DataTable>
	<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />
{:else if tab === 'visits'}
	<DataTable
		columns={[
			{ key: 'animal_id', label: 'Животное' },
			{ key: 'visit_datetime', label: 'Время' },
			{ key: 'feed_number', label: '№ корма', align: 'right' },
			{ key: 'amount', label: 'Кол-во, кг', align: 'right' },
			{ key: 'duration_seconds', label: 'Длительность, с', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtVisits}
		emptyText="Нет данных"
	>
		{#each visits as v, i (i)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
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
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{v.feed_number ?? '—'}</td
				>
				<td class="px-4 py-3 text-right font-medium"
					>{v.amount != null ? v.amount.toFixed(1) : '—'}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{v.duration_seconds ?? '—'}</td
				>
			</tr>
		{/each}
	</DataTable>
	<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />
{:else if tab === 'types'}
	<div class="flex justify-end mb-3">
		<button
			onclick={openCreateType}
			class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
		>
			+ Добавить тип
		</button>
	</div>

	<DataTable
		columns={[
			{ key: 'number_of_feed_type', label: '№' },
			{ key: 'name', label: 'Название' },
			{ key: 'feed_type', label: 'Тип' },
			{ key: 'dry_matter_percentage', label: 'Сухое вещество, %', align: 'right' },
			{ key: 'price', label: 'Цена', align: 'right' },
			{ key: 'stock_attention_level', label: 'Уровень запаса', align: 'right' },
			{ key: 'actions', label: 'Действия', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtTypes}
		emptyText="Нет данных"
	>
		{#each types as t (t.id)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{t.number_of_feed_type}</td>
				<td class="px-4 py-3 font-medium text-slate-800 dark:text-slate-100">{t.name}</td>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{t.feed_type}</td>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{t.dry_matter_percentage.toFixed(0)}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400">{t.price.toFixed(2)}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{t.stock_attention_level ?? '—'}</td
				>
				<td class="px-4 py-3 text-right">
					<div class="flex gap-2 justify-end">
						<button
							onclick={() => openEditType(t)}
							aria-label="Редактировать {t.name}"
							class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/50 rounded transition-colors cursor-pointer"
							><Pencil size={14} /></button
						>
						<button
							onclick={() => crudType.confirmDelete(t.id)}
							aria-label="Удалить {t.name}"
							class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-red-600 hover:bg-red-50 dark:bg-red-900/50 rounded transition-colors cursor-pointer"
							><Trash2 size={14} /></button
						>
					</div>
				</td>
			</tr>
		{/each}
	</DataTable>
{:else}
	<div class="flex justify-end mb-3">
		<button
			onclick={openCreateGroup}
			class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
		>
			+ Добавить группу
		</button>
	</div>

	<DataTable
		columns={[
			{ key: 'name', label: 'Название' },
			{ key: 'min_milk_yield', label: 'Надой мин', align: 'right' },
			{ key: 'max_milk_yield', label: 'Надой макс', align: 'right' },
			{ key: 'avg_milk_yield', label: 'Средний надой', align: 'right' },
			{ key: 'avg_weight', label: 'Средний вес', align: 'right' },
			{ key: 'number_of_cows', label: 'Коров', align: 'right' },
			{ key: 'actions', label: 'Действия', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtGroups}
		emptyText="Нет данных"
	>
		{#each groups as g (g.id)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3 font-medium text-slate-800 dark:text-slate-100">{g.name}</td>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{g.min_milk_yield?.toFixed(1) ?? '—'}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{g.max_milk_yield?.toFixed(1) ?? '—'}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{g.avg_milk_yield?.toFixed(1) ?? '—'}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{g.avg_weight?.toFixed(0) ?? '—'}</td
				>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400"
					>{g.number_of_cows ?? '—'}</td
				>
				<td class="px-4 py-3 text-right">
					<div class="flex gap-2 justify-end">
						<button
							onclick={() => openEditGroup(g)}
							aria-label="Редактировать {g.name}"
							class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/50 rounded transition-colors cursor-pointer"
							><Pencil size={14} /></button
						>
						<button
							onclick={() => crudGroup.confirmDelete(g.id)}
							aria-label="Удалить {g.name}"
							class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-red-600 hover:bg-red-50 dark:bg-red-900/50 rounded transition-colors cursor-pointer"
							><Trash2 size={14} /></button
						>
					</div>
				</td>
			</tr>
		{/each}
	</DataTable>
{/if}

<Modal
	open={crudType.showModal}
	title={crudType.modalMode === 'create' ? 'Новый тип корма' : 'Редактирование типа'}
	maxWidth="max-w-lg"
	onclose={crudType.close}
>
	<ErrorAlert message={crudType.modalError} />
	<form onsubmit={handleSubmitType} class="space-y-4">
		<div class="grid grid-cols-2 gap-4">
			<FormField
				id="ft-num"
				label="Номер типа"
				type="number"
				bind:value={formType.number_of_feed_type}
				required
			/>
			<FormField
				id="ft-code"
				label="Код типа"
				bind:value={formType.feed_type}
				required
				error={vType.getError('feed_type')}
			/>
		</div>
		<FormField
			id="ft-name"
			label="Название"
			bind:value={formType.name}
			required
			error={vType.getError('name')}
		/>
		<FormField id="ft-desc" label="Описание" type="textarea" bind:value={formType.description} />
		<div class="grid grid-cols-3 gap-4">
			<FormField
				id="ft-dm"
				label="Сухое вещество, %"
				type="number"
				bind:value={formType.dry_matter_percentage}
				required
				error={vType.getError('dry_matter_percentage')}
			/>
			<FormField
				id="ft-price"
				label="Цена"
				type="number"
				bind:value={formType.price}
				required
				error={vType.getError('price')}
			/>
			<FormField
				id="ft-stock"
				label="Уровень запаса"
				type="number"
				bind:value={formType.stock_attention_level}
			/>
		</div>
		<div class="flex gap-3 justify-end pt-2">
			<button
				type="button"
				onclick={crudType.close}
				class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 cursor-pointer"
				>Отмена</button
			>
			<button
				type="submit"
				disabled={crudType.modalLoading}
				class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50 cursor-pointer"
			>
				{crudType.modalLoading ? 'Сохранение...' : 'Сохранить'}
			</button>
		</div>
	</form>
</Modal>

<ConfirmDialog
	open={crudType.showDelete}
	title="Удалить тип корма?"
	message="Это действие нельзя отменить."
	loading={crudType.deleteLoading}
	onconfirm={handleDeleteType}
	oncancel={crudType.closeDelete}
/>

<Modal
	open={crudGroup.showModal}
	title={crudGroup.modalMode === 'create' ? 'Новая группа' : 'Редактирование группы'}
	maxWidth="max-w-lg"
	onclose={crudGroup.close}
>
	<ErrorAlert message={crudGroup.modalError} />
	<form onsubmit={handleSubmitGroup} class="space-y-4">
		<FormField
			id="fg-name"
			label="Название"
			bind:value={formGroup.name}
			required
			error={vGroup.getError('name')}
		/>
		<div class="grid grid-cols-3 gap-4">
			<FormField
				id="fg-min"
				label="Надой мин"
				type="number"
				bind:value={formGroup.min_milk_yield}
			/>
			<FormField
				id="fg-max"
				label="Надой макс"
				type="number"
				bind:value={formGroup.max_milk_yield}
			/>
			<FormField
				id="fg-avg"
				label="Средний надой"
				type="number"
				bind:value={formGroup.avg_milk_yield}
			/>
		</div>
		<div class="grid grid-cols-3 gap-4">
			<FormField id="fg-fat" label="Жир, %" type="number" bind:value={formGroup.avg_milk_fat} />
			<FormField
				id="fg-protein"
				label="Белок, %"
				type="number"
				bind:value={formGroup.avg_milk_protein}
			/>
			<FormField
				id="fg-weight"
				label="Средний вес"
				type="number"
				bind:value={formGroup.avg_weight}
			/>
		</div>
		<div class="grid grid-cols-3 gap-4">
			<FormField
				id="fg-maxrt"
				label="Макс. типов робота"
				type="number"
				bind:value={formGroup.max_robot_feed_types}
			/>
			<FormField
				id="fg-maxint"
				label="Макс. потребление"
				type="number"
				bind:value={formGroup.max_feed_intake_robot}
			/>
			<FormField
				id="fg-minint"
				label="Мин. потребление"
				type="number"
				bind:value={formGroup.min_feed_intake_robot}
			/>
		</div>
		<FormField
			id="fg-cows"
			label="Количество коров"
			type="number"
			bind:value={formGroup.number_of_cows}
		/>
		<div class="flex gap-3 justify-end pt-2">
			<button
				type="button"
				onclick={crudGroup.close}
				class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 cursor-pointer"
				>Отмена</button
			>
			<button
				type="submit"
				disabled={crudGroup.modalLoading}
				class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50 cursor-pointer"
			>
				{crudGroup.modalLoading ? 'Сохранение...' : 'Сохранить'}
			</button>
		</div>
	</form>
</Modal>

<ConfirmDialog
	open={crudGroup.showDelete}
	title="Удалить группу?"
	message="Это действие нельзя отменить."
	loading={crudGroup.deleteLoading}
	onconfirm={handleDeleteGroup}
	oncancel={crudGroup.closeDelete}
/>
