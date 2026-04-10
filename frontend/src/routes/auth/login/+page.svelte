<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { login } from '$lib/api/auth';
	import { goto } from '$app/navigation';
	import ErrorAlert from '$lib/components/ui/ErrorAlert.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import { useFormValidation } from '$lib/utils/useFormValidation.svelte';
	import { rules } from '$lib/utils/validators';

	let username = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);

	const v = useFormValidation();

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';

		if (
			!v.validateAll({
				username: { value: username, rules: [rules.required()] },
				password: { value: password, rules: [rules.required()] },
			})
		)
			return;

		try {
			loading = true;
			const result = await login(username, password);
			auth.login(result.username, result.role, result.must_change_password);
			goto('/');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Ошибка входа';
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Вход — Молочная ферма</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-slate-100 dark:bg-slate-900">
	<div class="w-full max-w-md">
		<div class="bg-white dark:bg-slate-800 rounded-2xl shadow-lg dark:shadow-slate-950/50 p-8">
			<div class="text-center mb-8">
				<h1 class="text-3xl font-bold text-slate-800 dark:text-white">Молочная ферма</h1>
				<p class="text-slate-500 dark:text-slate-400 mt-2">Система управления животноводством</p>
			</div>

			<form onsubmit={handleSubmit} class="space-y-5">
				<ErrorAlert message={error} />
				<FormField
					id="username"
					label="Логин"
					bind:value={username}
					placeholder="Введите логин"
					disabled={loading}
					error={v.getError('username')}
				/>
				<FormField
					id="password"
					label="Пароль"
					type="password"
					bind:value={password}
					placeholder="Введите пароль"
					disabled={loading}
					error={v.getError('password')}
				/>

				<button
					type="submit"
					disabled={loading}
					class="w-full py-2.5 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 dark:disabled:bg-blue-500 text-white font-medium rounded-lg transition-colors cursor-pointer disabled:cursor-not-allowed"
				>
					{loading ? 'Входим...' : 'Войти'}
				</button>
			</form>
		</div>
	</div>
</div>
