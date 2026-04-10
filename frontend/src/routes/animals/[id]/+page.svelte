<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { getAnimal, deleteAnimal, type Animal } from '$lib/api/animals';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { toasts } from '$lib/stores/toast';

	let animal = $state<Animal | null>(null);
	let loading = $state(true);
	let error = $state('');
	let showDelete = $state(false);
	let deleteLoading = $state(false);

	let id = $derived(Number($page.params.id));

	async function load() {
		try {
			loading = true;
			animal = (await getAnimal(id)).data;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			loading = false;
		}
	}

	async function handleDelete() {
		try {
			deleteLoading = true;
			await deleteAnimal(id);
			toasts.success('Животное удалено');
			goto('/animals');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка удаления';
			showDelete = false;
		} finally {
			deleteLoading = false;
		}
	}

	onMount(load);
</script>

<svelte:head>
	<title>{animal?.name || 'Животное'} — Молочная ферма</title>
</svelte:head>

<div class="mb-6">
	<a
		href="/animals"
		class="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
		>&larr; Назад к списку</a
	>
</div>

<ErrorAlert message={error} />

{#if loading}
	<div
		class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
	>
		<div class="animate-pulse space-y-4">
			{#each Array(6) as _, i (i)}
				<div class="h-4 bg-slate-200 dark:bg-slate-700 rounded w-1/3"></div>
			{/each}
		</div>
	</div>
{:else if animal}
	<div class="flex items-center justify-between mb-6">
		<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">
			{animal.name || 'Без имени'}
			<span class="text-base font-normal text-slate-400 dark:text-slate-500 ml-2"
				>#{animal.life_number || animal.user_number || animal.id}</span
			>
		</h1>
		<div class="flex gap-2">
			<a
				href="/animals/{animal.id}/edit"
				class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-lg transition-colors"
				>Редактировать</a
			>
			<button
				onclick={() => (showDelete = true)}
				class="px-4 py-2 border border-red-300 text-red-600 hover:bg-red-50 dark:bg-red-900/50 text-sm rounded-lg transition-colors cursor-pointer"
				>Удалить</button
			>
		</div>
	</div>

	<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4">
				Основная информация
			</h2>
			<dl class="space-y-3">
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Пол</dt>
					<dd>
						<span
							class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {animal.gender ===
							'female'
								? 'bg-pink-100 dark:bg-pink-900/50 text-pink-700'
								: 'bg-blue-100 dark:bg-blue-900/50 text-blue-700'}"
						>
							{animal.gender === 'female' ? 'Корова' : 'Бык'}
						</span>
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Дата рождения</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">{animal.birth_date}</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Статус</dt>
					<dd>
						<span
							class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {animal.active
								? 'bg-green-100 dark:bg-green-900/50 text-green-700'
								: 'bg-slate-100 dark:bg-slate-900 text-slate-500 dark:text-slate-400'}"
						>
							{animal.active ? 'Активно' : 'Неактивно'}
						</span>
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Локация</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">{animal.location || '—'}</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Группа</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">{animal.group_number ?? '—'}</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Код масти</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">
						{animal.hair_color_code || '—'}
					</dd>
				</div>
			</dl>
		</div>

		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
		>
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-4">
				Номера и идентификация
			</h2>
			<dl class="space-y-3">
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Жизненный номер</dt>
					<dd class="text-sm font-mono text-slate-800 dark:text-slate-100">
						{animal.life_number || '—'}
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Пользовательский номер</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">{animal.user_number ?? '—'}</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">UCN номер</dt>
					<dd class="text-sm font-mono text-slate-800 dark:text-slate-100">
						{animal.ucn_number || '—'}
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Номер отца</dt>
					<dd class="text-sm font-mono text-slate-800 dark:text-slate-100">
						{animal.father_life_number || '—'}
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Номер матери</dt>
					<dd class="text-sm font-mono text-slate-800 dark:text-slate-100">
						{animal.mother_life_number || '—'}
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="text-sm text-slate-500 dark:text-slate-400">Номер респондера</dt>
					<dd class="text-sm text-slate-800 dark:text-slate-100">
						{animal.responder_number || '—'}
					</dd>
				</div>
			</dl>
		</div>
	</div>

	{#if animal.description}
		<div
			class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6 mt-6"
		>
			<h2 class="text-lg font-semibold text-slate-700 dark:text-slate-300 mb-2">Описание</h2>
			<p class="text-sm text-slate-600 dark:text-slate-400 whitespace-pre-wrap">
				{animal.description}
			</p>
		</div>
	{/if}
{/if}

<ConfirmDialog
	open={showDelete}
	title="Удалить животное?"
	message="Это действие нельзя отменить."
	loading={deleteLoading}
	onconfirm={handleDelete}
	oncancel={() => (showDelete = false)}
/>
