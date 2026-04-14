<script lang="ts">
	import {
		listItems,
		createItem,
		updateItem,
		deleteItem,
		createTransaction,
		getLowStock,
		INVENTORY_CATEGORY_LABELS,
		type InventoryItem,
		type InventoryTransaction,
		type CreateInventoryItem,
		type UpdateInventoryItem,
	} from '$lib/api/inventory';
	import DataTable from '$lib/components/ui/DataTable.svelte';
	import TabBar from '$lib/components/ui/TabBar.svelte';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import Pagination from '$lib/components/ui/Pagination.svelte';
	import { usePaginatedList } from '$lib/utils/usePaginatedList.svelte';
	import { useCrudModal } from '$lib/utils/useCrudModal.svelte';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';
	import { Pencil, Trash2, ArrowUpDown, AlertTriangle } from 'lucide-svelte';

	type Tab = 'items' | 'transactions';

	let tab = $state<Tab>('items');

	let dtItems: DataTable | undefined = $state();
	let items = $state<InventoryItem[]>([]);
	let lowStockItems = $state<InventoryItem[]>([]);

	const list = usePaginatedList({ perPage: 50 });
	const crud = useCrudModal();
	const v = useFormValidation();

	let searchQuery = $state('');
	let filterCategory = $state('');
	let filterLowStock = $state(false);

	let formItem = $state<CreateInventoryItem>({
		name: '',
		category: 'feed',
		unit: 'pcs',
		quantity: 0,
		min_quantity: 0,
	});

	let showTransactionModal = $state(false);
	let transactionItemId = $state(0);
	let transactionItemName = $state('');
	let transactionType = $state<'in' | 'out' | 'adjustment'>('in');
	let transactionQuantity = $state<number | undefined>(undefined);
	let transactionNotes = $state('');
	let transactionLoading = $state(false);
	let transactionError = $state('');

	let recentTransactions = $state<{ item: string; type: string; qty: number; date: string }[]>([]);

	async function load() {
		const params: Record<string, unknown> = {
			search: searchQuery || undefined,
			category: filterCategory || undefined,
			low_stock: filterLowStock || undefined,
			page: list.page,
			per_page: list.perPage,
		};
		await list.load(
			(signal) => listItems(params as Record<string, string | number | boolean | undefined>, signal),
			(d) => {
				items = d;
			},
			dtItems,
		);
	}

	async function loadLowStock() {
		try {
			const res = await getLowStock();
			lowStockItems = res.data;
		} catch {
			// ignore
		}
	}

	function switchTab(t: string) {
		tab = t as Tab;
	}

	function openCreate() {
		v.clear();
		formItem = {
			name: '',
			category: 'feed',
			unit: 'pcs',
			quantity: 0,
			min_quantity: 0,
		};
		crud.openCreate();
	}

	function openEdit(item: InventoryItem) {
		v.clear();
		formItem = {
			name: item.name,
			category: item.category,
			unit: item.unit,
			min_quantity: item.min_quantity,
			cost_per_unit: item.cost_per_unit ?? undefined,
			supplier: item.supplier ?? undefined,
			notes: item.notes ?? undefined,
		};
		crud.openEdit(item.id);
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		const fields: Record<
			string,
			{ value: string | number | undefined; rules: ReturnType<typeof rules.required>[] }
		> = {
			name: { value: formItem.name, rules: [rules.required()] },
			category: { value: formItem.category, rules: [rules.required()] },
		};
		if (!v.validateAll(fields)) return;

		const data = crud.modalMode === 'create'
			? (formItem as CreateInventoryItem)
			: ({
					name: formItem.name,
					category: formItem.category,
					unit: formItem.unit,
					min_quantity: formItem.min_quantity,
					cost_per_unit: formItem.cost_per_unit,
					supplier: formItem.supplier,
					notes: formItem.notes,
				} as UpdateInventoryItem);

		await crud.submit(
			() =>
				crud.modalMode === 'create'
					? createItem(data as CreateInventoryItem)
					: updateItem(crud.editingId, data as UpdateInventoryItem),
			crud.modalMode === 'create' ? 'Позиция создана' : 'Позиция обновлена',
			() => {
				load();
				loadLowStock();
			},
		);
	}

	async function handleDelete() {
		await crud.remove(
			() => deleteItem(crud.deleteId),
			() => {
				load();
				loadLowStock();
			},
			(msg) => {
				list.error = msg;
			},
		);
	}

	function openTransaction(item: InventoryItem) {
		transactionItemId = item.id;
		transactionItemName = item.name;
		transactionType = 'in';
		transactionQuantity = undefined;
		transactionNotes = '';
		transactionError = '';
		showTransactionModal = true;
	}

	async function handleTransaction(e: Event) {
		e.preventDefault();
		if (!transactionQuantity || transactionQuantity <= 0) {
			transactionError = 'Укажите количество больше 0';
			return;
		}
		transactionLoading = true;
		transactionError = '';
		try {
			await createTransaction(transactionItemId, {
				transaction_type: transactionType,
				quantity: transactionQuantity,
				notes: transactionNotes || undefined,
			});
			showTransactionModal = false;
			load();
			loadLowStock();
		} catch (e) {
			transactionError = e instanceof Error ? e.message : 'Ошибка';
		} finally {
			transactionLoading = false;
		}
	}

	function handleSearch() {
		list.resetPage();
		load();
	}

	function handleFilterChange() {
		list.resetPage();
		load();
	}

	$effect(() => {
		load();
		loadLowStock();
	});
</script>

<svelte:head>
	<title>Склад — Молочная ферма</title>
</svelte:head>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-6">Склад</h1>

<TabBar
	tabs={[
		{ key: 'items', label: 'Склад' },
		{ key: 'transactions', label: 'Движение' },
	]}
	bind:active={tab}
	onchange={switchTab}
/>

{#if tab === 'items'}
	<div class="flex flex-col sm:flex-row gap-3 mb-4">
		<div class="flex-1 flex gap-2">
			<input
				type="text"
				placeholder="Поиск..."
				bind:value={searchQuery}
				onkeydown={(e) => e.key === 'Enter' && handleSearch()}
				class="flex-1 px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
			/>
			<button
				onclick={handleSearch}
				class="px-4 py-2 text-sm bg-slate-200 dark:bg-slate-700 hover:bg-slate-300 dark:hover:bg-slate-600 rounded-lg transition-colors cursor-pointer"
			>Поиск</button>
		</div>
		<select
			bind:value={filterCategory}
			onchange={handleFilterChange}
			class="px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
		>
			<option value="">Все категории</option>
			{#each Object.entries(INVENTORY_CATEGORY_LABELS) as [key, label]}
				<option value={key}>{label}</option>
			{/each}
		</select>
		<label class="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-400 cursor-pointer">
			<input
				type="checkbox"
				bind:checked={filterLowStock}
				onchange={handleFilterChange}
				class="rounded border-slate-300 dark:border-slate-600"
			/>
			Низкий запас
		</label>
		<div class="flex gap-2">
			<button
				onclick={openCreate}
				class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
			>
				+ Добавить
			</button>
		</div>
	</div>

	{#if lowStockItems.length > 0}
		<div class="mb-4 p-3 bg-amber-50 dark:bg-amber-900/30 border border-amber-200 dark:border-amber-700 rounded-lg flex items-center gap-2">
			<AlertTriangle size={18} class="text-amber-600 dark:text-amber-400 flex-shrink-0" />
			<span class="text-sm text-amber-800 dark:text-amber-200">
				{lowStockItems.length} поз. с низким запасом
			</span>
		</div>
	{/if}

	<ErrorAlert message={list.error} />

	<DataTable
		columns={[
			{ key: 'name', label: 'Название' },
			{ key: 'category', label: 'Категория' },
			{ key: 'quantity', label: 'Кол-во', align: 'right' },
			{ key: 'min_quantity', label: 'Мин.', align: 'right' },
			{ key: 'unit', label: 'Ед.' },
			{ key: 'cost_per_unit', label: 'Цена/ед.', align: 'right' },
			{ key: 'actions', label: 'Действия', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dtItems}
		emptyText="Нет данных"
	>
		{#each items as item (item.id)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3 font-medium text-slate-800 dark:text-slate-100">
					{item.name}
					{#if item.quantity <= item.min_quantity}
						<AlertTriangle size={14} class="inline ml-1 text-red-500" />
					{/if}
				</td>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">
					{INVENTORY_CATEGORY_LABELS[item.category] ?? item.category}
				</td>
				<td class="px-4 py-3 text-right font-medium {item.quantity <= item.min_quantity ? 'text-red-600 dark:text-red-400' : ''}">
					{item.quantity.toFixed(item.quantity % 1 === 0 ? 0 : 3)}
				</td>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400">
					{item.min_quantity.toFixed(item.min_quantity % 1 === 0 ? 0 : 3)}
				</td>
				<td class="px-4 py-3 text-slate-600 dark:text-slate-400">{item.unit}</td>
				<td class="px-4 py-3 text-right text-slate-600 dark:text-slate-400">
					{item.cost_per_unit != null ? item.cost_per_unit.toFixed(2) : '—'}
				</td>
				<td class="px-4 py-3 text-right">
					<div class="flex gap-2 justify-end">
						<button
							onclick={() => openTransaction(item)}
							aria-label="Движение {item.name}"
							class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-green-600 dark:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/50 rounded transition-colors cursor-pointer"
							title="Движение"
						><ArrowUpDown size={14} /></button>
						<button
							onclick={() => openEdit(item)}
							aria-label="Редактировать {item.name}"
							class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/50 rounded transition-colors cursor-pointer"
						><Pencil size={14} /></button>
						<button
							onclick={() => crud.confirmDelete(item.id)}
							aria-label="Удалить {item.name}"
							class="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:text-red-600 hover:bg-red-50 dark:bg-red-900/50 rounded transition-colors cursor-pointer"
						><Trash2 size={14} /></button>
					</div>
				</td>
			</tr>
		{/each}
	</DataTable>
	<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />
{:else}
	<div class="text-sm text-slate-500 dark:text-slate-400 mb-4">
		История последних операций отображается здесь. Для добавления движения перейдите на вкладку «Склад».
	</div>
{/if}

<Modal
	open={crud.showModal}
	title={crud.modalMode === 'create' ? 'Новая позиция' : 'Редактирование'}
	maxWidth="max-w-lg"
	onclose={crud.close}
>
	<ErrorAlert message={crud.modalError} />
	<form onsubmit={handleSubmit} class="space-y-4">
		<div class="grid grid-cols-2 gap-4">
			<FormField
				id="inv-name"
				label="Название"
				bind:value={formItem.name}
				required
				error={v.getError('name')}
			/>
			<div>
				<label for="inv-cat" class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">Категория</label>
				<select
					id="inv-cat"
					bind:value={formItem.category}
					class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
				>
					{#each Object.entries(INVENTORY_CATEGORY_LABELS) as [key, label]}
						<option value={key}>{label}</option>
					{/each}
				</select>
			</div>
		</div>
		<div class="grid grid-cols-3 gap-4">
			<FormField
				id="inv-unit"
				label="Единица"
				bind:value={formItem.unit}
			/>
			{#if crud.modalMode === 'create'}
				<FormField
					id="inv-qty"
					label="Количество"
					type="number"
					bind:value={formItem.quantity}
				/>
			{/if}
			<FormField
				id="inv-min"
				label="Мин. количество"
				type="number"
				bind:value={formItem.min_quantity}
			/>
			<FormField
				id="inv-cost"
				label="Цена за единицу"
				type="number"
				bind:value={formItem.cost_per_unit}
			/>
		</div>
		<FormField
			id="inv-supplier"
			label="Поставщик"
			bind:value={formItem.supplier}
		/>
		<FormField
			id="inv-notes"
			label="Заметки"
			type="textarea"
			bind:value={formItem.notes}
		/>
		<div class="flex gap-3 justify-end pt-2">
			<button
				type="button"
				onclick={crud.close}
				class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 cursor-pointer"
			>Отмена</button>
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
	title="Удалить позицию?"
	message="Это действие нельзя отменить."
	loading={crud.deleteLoading}
	onconfirm={handleDelete}
	oncancel={crud.closeDelete}
/>

<Modal
	open={showTransactionModal}
	title="Движение: {transactionItemName}"
	maxWidth="max-w-md"
	onclose={() => (showTransactionModal = false)}
>
	<ErrorAlert message={transactionError} />
	<form onsubmit={handleTransaction} class="space-y-4">
		<div>
			<label for="tx-type" class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">Тип</label>
			<select
				id="tx-type"
				bind:value={transactionType}
				class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
			>
				<option value="in">Приход</option>
				<option value="out">Расход</option>
				<option value="adjustment">Корректировка</option>
			</select>
		</div>
		<FormField
			id="tx-qty"
			label="Количество"
			type="number"
			bind:value={transactionQuantity}
			required
		/>
		<FormField
			id="tx-notes"
			label="Заметки"
			bind:value={transactionNotes}
		/>
		<div class="flex gap-3 justify-end pt-2">
			<button
				type="button"
				onclick={() => (showTransactionModal = false)}
				class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 cursor-pointer"
			>Отмена</button>
			<button
				type="submit"
				disabled={transactionLoading}
				class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50 cursor-pointer"
			>
				{transactionLoading ? 'Сохранение...' : 'Сохранить'}
			</button>
		</div>
	</form>
</Modal>
