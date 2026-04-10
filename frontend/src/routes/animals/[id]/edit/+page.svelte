<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { getAnimal, updateAnimal, type Animal, type UpdateAnimal } from '$lib/api/animals';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import { toasts } from '$lib/stores/toast';

	let animal = $state<Animal | null>(null);
	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');

	let form = $state<UpdateAnimal>({});

	let id = $derived(Number($page.params.id));

	async function load() {
		try {
			loading = true;
			animal = (await getAnimal(id)).data;
			form = {
				name: animal.name ?? '',
				hair_color_code: animal.hair_color_code ?? '',
				description: animal.description ?? '',
				ucn_number: animal.ucn_number ?? '',
				use_as_sire: animal.use_as_sire ?? false,
				location: animal.location ?? '',
				group_number: animal.group_number ?? undefined,
				keep: animal.keep ?? false,
				responder_number: animal.responder_number ?? '',
				active: animal.active,
			};
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка загрузки';
		} finally {
			loading = false;
		}
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		try {
			saving = true;
			await updateAnimal(id, form);
			toasts.success('Животное обновлено');
			goto(`/animals/${id}`);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка сохранения';
		} finally {
			saving = false;
		}
	}

	onMount(load);
</script>

<svelte:head>
	<title>Редактирование — {animal?.name || 'Животное'} — Молочная ферма</title>
</svelte:head>

<div class="mb-6">
	<a
		href="/animals/{id}"
		class="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
		>&larr; Назад к животному</a
	>
</div>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-6">
	Редактирование: {animal?.name || 'Без имени'}
</h1>

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
{:else}
	<form
		onsubmit={handleSubmit}
		class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
	>
		<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
			<FormField id="f-name" label="Имя" bind:value={form.name} placeholder="Кличка" />
			<FormField id="f-ucn" label="UCN номер" bind:value={form.ucn_number} placeholder="UCN" />
			<FormField id="f-loc" label="Локация" bind:value={form.location} placeholder="Локация" />
			<FormField
				id="f-group"
				label="Группа"
				type="number"
				bind:value={form.group_number}
				placeholder="№ группы"
			/>
			<FormField
				id="f-hair"
				label="Код масти"
				bind:value={form.hair_color_code}
				placeholder="Код"
			/>
			<FormField
				id="f-resp"
				label="Номер респондера"
				bind:value={form.responder_number}
				placeholder="Номер"
			/>
			<FormField
				label="Использовать как производителя"
				type="checkbox"
				bind:checked={form.use_as_sire}
			/>
			<FormField label="Оставить" type="checkbox" bind:checked={form.keep} />
			<FormField label="Активно" type="checkbox" bind:checked={form.active} />
			<div class="md:col-span-2">
				<FormField
					id="f-desc"
					label="Описание"
					type="textarea"
					bind:value={form.description}
					placeholder="Примечания..."
				/>
			</div>
		</div>

		<div class="flex gap-3 mt-6 pt-4 border-t border-slate-200 dark:border-slate-700">
			<button
				type="submit"
				disabled={saving}
				class="px-6 py-2.5 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg disabled:opacity-50 cursor-pointer transition-colors"
			>
				{saving ? 'Сохранение...' : 'Сохранить'}
			</button>
			<a
				href="/animals/{id}"
				class="px-6 py-2.5 border border-slate-300 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
				>Отмена</a
			>
		</div>
	</form>
{/if}
