<script lang="ts">
	import {
		listTransactions,
		createTransaction,
		deleteTransaction,
		type Transaction,
		type CreateTransaction,
		FINANCE_CATEGORIES,
		FINANCE_CATEGORY_LABELS,
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
	import { Trash2 } from 'lucide-svelte';

	type Tab = 'transactions' | 'summary';
	let activeTab: Tab = $state('transactions');

	let dataTable: DataTable;
	let transactions = $state<Transaction[]>([]);

	const list = usePaginatedList();
	const crud = useCrudModal();
	const v = useFormValidation();

	let _skipLoad = true;
	let today = new Date().toISOString().slice(0, 10);

	let filterType = $state<string>('');
	let filterCategory = $state<string>('');

	let createForm = $state<CreateTransaction>({
		transaction_type: 'income',
		category: 'milk_sales',
		amount: 0,
		transaction_date: today,
	});

	const amountRules = [rules.required()];
	const dateRules = [rules.required()];

	let totalIncome = $derived(
		transactions
			.filter((t) => t.transaction_type === 'income')
			.reduce((sum, t) => sum + Number(t.amount), 0),
	);
	let totalExpense = $derived(
		transactions
			.filter((t) => t.transaction_type === 'expense')
			.reduce((sum, t) => sum + Number(t.amount), 0),
	);
	let balance = $derived(totalIncome - totalExpense);

	let incomeByCategory = $derived(
		transactions
			.filter((t) => t.transaction_type === 'income')
			.reduce<Record<string, number>>((acc, t) => {
				acc[t.category] = (acc[t.category] || 0) + Number(t.amount);
				return acc;
			}, {}),
	);
	let expenseByCategory = $derived(
		transactions
			.filter((t) => t.transaction_type === 'expense')
			.reduce<Record<string, number>>((acc, t) => {
				acc[t.category] = (acc[t.category] || 0) + Number(t.amount);
				return acc;
			}, {}),
	);

	let categoryOptions = $derived(
		createForm.transaction_type === 'income'
			? FINANCE_CATEGORIES.income
			: FINANCE_CATEGORIES.expense,
	);

	async function load() {
		await list.load(
			(signal) =>
				listTransactions(
					{
						transaction_type: filterType || undefined,
						category: filterCategory || undefined,
						from_date: list.fromDate || undefined,
						till_date: list.tillDate || undefined,
						page: list.page,
						per_page: list.perPage,
					},
					signal,
				),
			(data) => {
				transactions = data;
			},
			dataTable,
		);
	}

	function switchTab(tab: Tab) {
		activeTab = tab;
		list.page = 1;
		_skipLoad = false;
		load();
	}

	function openCreate() {
		createForm = {
			transaction_type: 'income',
			category: 'milk_sales',
			amount: 0,
			transaction_date: today,
		};
		v.clear();
		crud.openCreate();
	}

	function handleTypeChange() {
		const cats =
			createForm.transaction_type === 'income'
				? FINANCE_CATEGORIES.income
				: FINANCE_CATEGORIES.expense;
		createForm.category = cats[0];
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		const valid = v.validateAll({
			amount: { value: createForm.amount, rules: amountRules },
			transaction_date: { value: createForm.transaction_date, rules: dateRules },
		});
		if (!valid) return;
		await crud.submit(
			() => createTransaction(createForm),
			'Транзакция создана',
			load,
		);
	}

	async function handleDelete() {
		await crud.remove(
			() => deleteTransaction(crud.deleteId),
			load,
			(msg) => (list.error = msg),
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

	function selectCls(): string {
		return 'px-2 py-1 text-sm border border-slate-300 dark:border-slate-600 rounded bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200';
	}

	function fmtMoney(v: number): string {
		return v.toLocaleString('ru-RU', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
	}
</script>

<svelte:head>
	<title>Финансы — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-between mb-4">
	<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">Финансы</h1>
	<button
		onclick={openCreate}
		class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
	>
		+ Добавить
	</button>
</div>

<div class="flex gap-1 border-b border-slate-200 dark:border-slate-700 mb-4">
	<button
		onclick={() => switchTab('transactions')}
		class="px-4 py-2 text-sm font-medium transition-colors cursor-pointer relative {activeTab ===
		'transactions'
			? 'text-blue-600 dark:text-blue-400'
			: 'text-slate-500 hover:text-slate-700'}"
	>
		Доходы/Расходы
		{#if activeTab === 'transactions'}
			<span class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-600 dark:bg-blue-400 rounded-full" />
		{/if}
	</button>
	<button
		onclick={() => switchTab('summary')}
		class="px-4 py-2 text-sm font-medium transition-colors cursor-pointer relative {activeTab ===
		'summary'
			? 'text-blue-600 dark:text-blue-400'
			: 'text-slate-500 hover:text-slate-700'}"
	>
		Сводка
		{#if activeTab === 'summary'}
			<span class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-600 dark:bg-blue-400 rounded-full" />
		{/if}
	</button>
</div>

{#if activeTab === 'transactions'}
	<div
		class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-4 mb-6"
	>
		<div class="flex flex-wrap gap-3 items-end">
			<div>
				<label class="block text-xs text-slate-500 dark:text-slate-400 mb-1">Тип</label>
				<select bind:value={filterType} class={selectCls()}>
					<option value="">Все</option>
					<option value="income">Доход</option>
					<option value="expense">Расход</option>
				</select>
			</div>
			<div>
				<label class="block text-xs text-slate-500 dark:text-slate-400 mb-1">Категория</label>
				<select bind:value={filterCategory} class={selectCls()}>
					<option value="">Все</option>
					{#each Object.entries(FINANCE_CATEGORY_LABELS) as [key, label] (key)}
						<option value={key}>{label}</option>
					{/each}
				</select>
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
			{ key: 'transaction_date', label: 'Дата' },
			{ key: 'transaction_type', label: 'Тип' },
			{ key: 'category', label: 'Категория' },
			{ key: 'amount', label: 'Сумма', align: 'right' },
			{ key: 'description', label: 'Описание' },
			{ key: 'reference', label: 'Референс' },
			{ key: 'actions', label: '', align: 'right' },
		]}
		loading={list.loading}
		bind:this={dataTable}
		emptyText="Нет транзакций"
		initialRows={false}
	>
		{#each transactions as t (t.id)}
			<tr
				class="border-b border-slate-100 dark:border-slate-700 hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>
				<td class="px-4 py-3 text-slate-500 dark:text-slate-400">{t.id}</td>
				<td class="px-4 py-3">{t.transaction_date}</td>
				<td class="px-4 py-3">
					<span
						class="px-2 py-0.5 rounded-full text-xs font-medium {t.transaction_type === 'income'
							? 'bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-400'
							: 'bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-400'}"
					>
						{t.transaction_type === 'income' ? 'Доход' : 'Расход'}
					</span>
				</td>
				<td class="px-4 py-3 text-sm">{FINANCE_CATEGORY_LABELS[t.category] ?? t.category}</td>
				<td
					class="px-4 py-3 text-right font-semibold {t.transaction_type === 'income'
						? 'text-green-600 dark:text-green-400'
						: 'text-red-600 dark:text-red-400'}"
				>
					{t.transaction_type === 'income' ? '+' : '-'}{fmtMoney(Number(t.amount))}
				</td>
				<td class="px-4 py-3 text-sm text-slate-600 dark:text-slate-400 max-w-[200px] truncate"
					>{t.description ?? '—'}</td
				>
				<td class="px-4 py-3 text-sm text-slate-600 dark:text-slate-400"
					>{t.reference ?? '—'}</td
				>
				<td class="px-4 py-3 text-right">
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

	<Modal open={crud.showModal} title="Новая транзакция" onclose={crud.close}>
		<ErrorAlert message={crud.modalError} />
		<form onsubmit={handleSubmit} class="space-y-4">
			<div class="grid grid-cols-2 gap-3">
				<div>
					<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
						>Тип</label
					>
					<select
						bind:value={createForm.transaction_type}
						onchange={handleTypeChange}
						class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
					>
						<option value="income">Доход</option>
						<option value="expense">Расход</option>
					</select>
				</div>
				<div>
					<label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
						>Категория</label
					>
					<select
						bind:value={createForm.category}
						class="w-full px-3 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200"
					>
						{#each categoryOptions as key (key)}
							<option value={key}>{FINANCE_CATEGORY_LABELS[key]}</option>
						{/each}
					</select>
				</div>
			</div>
			<FormField
				id="c-amount"
				label="Сумма"
				type="number"
				step="0.01"
				bind:value={createForm.amount}
				required
				error={v.getError('amount')}
				onblur={() => v.validateField('amount', createForm.amount, amountRules)}
			/>
			<FormField
				id="c-date"
				label="Дата"
				type="date"
				bind:value={createForm.transaction_date}
				required
				error={v.getError('transaction_date')}
				onblur={() => v.validateField('transaction_date', createForm.transaction_date, dateRules)}
			/>
			<FormField id="c-desc" label="Описание" bind:value={createForm.description} />
			<div class="grid grid-cols-2 gap-3">
				<FormField id="c-animal" label="ID животного" type="number" bind:value={createForm.animal_id} />
				<FormField id="c-ref" label="Референс" bind:value={createForm.reference} />
			</div>
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

	<Pagination bind:page={list.page} total={list.total} perPage={list.perPage} />
{:else}
	<ErrorAlert message={list.error} />

	<div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-6">
		<div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4">
			<div class="text-xs text-slate-500 dark:text-slate-400 mb-1">Доходы</div>
			<div class="text-xl font-bold text-green-600 dark:text-green-400">{fmtMoney(totalIncome)}</div>
		</div>
		<div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4">
			<div class="text-xs text-slate-500 dark:text-slate-400 mb-1">Расходы</div>
			<div class="text-xl font-bold text-red-600 dark:text-red-400">{fmtMoney(totalExpense)}</div>
		</div>
		<div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4">
			<div class="text-xs text-slate-500 dark:text-slate-400 mb-1">Баланс</div>
			<div
				class="text-xl font-bold {balance >= 0
					? 'text-green-600 dark:text-green-400'
					: 'text-red-600 dark:text-red-400'}"
			>
				{fmtMoney(balance)}
			</div>
		</div>
	</div>

	<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
		<div
			class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4"
		>
			<h3 class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3">Доходы по категориям</h3>
			{#if Object.keys(incomeByCategory).length === 0}
				<div class="text-sm text-slate-400">Нет данных</div>
			{:else}
				<div class="space-y-2">
					{#each Object.entries(incomeByCategory).toSorted((a, b) => b[1] - a[1]) as [cat, sum] (cat)}
						<div class="flex justify-between text-sm">
							<span class="text-slate-600 dark:text-slate-400"
								>{FINANCE_CATEGORY_LABELS[cat] ?? cat}</span
							>
							<span class="font-medium text-green-600 dark:text-green-400"
								>{fmtMoney(sum)}</span
							>
						</div>
					{/each}
				</div>
			{/if}
		</div>
		<div
			class="bg-white dark:bg-slate-800 rounded-xl border border-slate-100 dark:border-slate-700 p-4"
		>
			<h3 class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3">Расходы по категориям</h3>
			{#if Object.keys(expenseByCategory).length === 0}
				<div class="text-sm text-slate-400">Нет данных</div>
			{:else}
				<div class="space-y-2">
					{#each Object.entries(expenseByCategory).toSorted((a, b) => b[1] - a[1]) as [cat, sum] (cat)}
						<div class="flex justify-between text-sm">
							<span class="text-slate-600 dark:text-slate-400"
								>{FINANCE_CATEGORY_LABELS[cat] ?? cat}</span
							>
							<span class="font-medium text-red-600 dark:text-red-400"
								>{fmtMoney(sum)}</span
							>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</div>
{/if}

<ConfirmDialog
	open={crud.showDelete}
	title="Удалить транзакцию?"
	message="Это действие нельзя отменить."
	loading={crud.deleteLoading}
	onconfirm={handleDelete}
	oncancel={crud.closeDelete}
/>
