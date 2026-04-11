<script lang="ts">
	import { goto } from '$app/navigation';
	import { createAnimal, type CreateAnimal } from '$lib/api/animals';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import { toasts } from '$lib/stores/toast';
	import { useFormDirty } from '$lib/utils/useFormDirty.svelte';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';

	const dirty = useFormDirty();

	let form = $state<CreateAnimal>({
		gender: 'female',
		birth_date: '',
		name: '',
	});

	let loading = $state(false);
	let error = $state('');

	const v = useFormValidation();

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		if (
			!v.validateAll({
				birth_date: { value: form.birth_date, rules: [rules.required()] },
			})
		)
			return;
		try {
			loading = true;
			const res = await createAnimal(form);
			dirty.reset();
			toasts.success('Животное создано');
			goto(`/animals/${res.data.id}`);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ошибка создания';
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Новое животное — Молочная ферма</title>
</svelte:head>

<div class="mb-6">
	<a
		href="/animals"
		class="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-400"
		>&larr; Назад к списку</a
	>
</div>

<h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100 mb-6">Новое животное</h1>

<ErrorAlert message={error} />

<form
	onsubmit={handleSubmit}
	oninput={dirty.markDirty}
	onchange={dirty.markDirty}
	class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-100 dark:border-slate-700 p-6"
>
	<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
		<FormField
			id="f-gender"
			label="Пол"
			type="select"
			bind:value={form.gender}
			options={[
				{ value: 'female', label: 'Корова' },
				{ value: 'male', label: 'Бык' },
			]}
		/>
		<FormField
			id="f-birth"
			label="Дата рождения"
			type="date"
			bind:value={form.birth_date}
			required
			error={v.getError('birth_date')}
		/>
		<FormField id="f-name" label="Имя" bind:value={form.name} placeholder="Кличка" />
		<FormField
			id="f-life"
			label="Жизненный номер"
			bind:value={form.life_number}
			placeholder="Номер"
		/>
		<FormField
			id="f-user"
			label="Пользовательский номер"
			type="number"
			bind:value={form.user_number}
			placeholder="Номер"
		/>
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
			id="f-father"
			label="Номер отца"
			bind:value={form.father_life_number}
			placeholder="Жизненный номер отца"
		/>
		<FormField
			id="f-mother"
			label="Номер матери"
			bind:value={form.mother_life_number}
			placeholder="Жизненный номер матери"
		/>
		<FormField id="f-hair" label="Код масти" bind:value={form.hair_color_code} placeholder="Код" />
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
			disabled={loading}
			class="px-6 py-2.5 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg disabled:opacity-50 cursor-pointer transition-colors"
		>
			{loading ? 'Создание...' : 'Создать'}
		</button>
		<a
			href="/animals"
			class="px-6 py-2.5 border border-slate-300 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 transition-colors"
			>Отмена</a
		>
	</div>
</form>
