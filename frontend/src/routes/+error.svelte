<script lang="ts">
	import { page } from '$app/stores';

	let status = $derived($page.status);
	let message = $derived($page.error?.message);

	const titles: Record<number, string> = {
		400: 'Некорректный запрос',
		401: 'Не авторизован',
		403: 'Доступ запрещён',
		404: 'Страница не найдена',
		500: 'Ошибка сервера',
		503: 'Сервис недоступен',
	};

	let title = $derived(titles[status] || 'Что-то пошло не так');
</script>

<svelte:head>
	<title>{status} — Молочная ферма</title>
</svelte:head>

<div class="flex items-center justify-center min-h-screen bg-slate-100 dark:bg-slate-900">
	<div class="text-center max-w-md mx-auto px-4">
		<h1 class="text-8xl font-bold text-slate-200 dark:text-slate-700">{status}</h1>
		<h2 class="mt-4 text-2xl font-semibold text-slate-700 dark:text-slate-200">{title}</h2>
		{#if message}
			<p class="mt-2 text-sm text-slate-500 dark:text-slate-400">{message}</p>
		{/if}
		<div class="mt-8 flex items-center justify-center gap-3">
			<a
				href="/"
				class="px-5 py-2.5 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors"
			>
				На главную
			</a>
			<button
				onclick={() => history.back()}
				class="px-5 py-2.5 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 hover:bg-slate-50 dark:hover:bg-slate-700 text-slate-700 dark:text-slate-300 font-medium rounded-lg transition-colors cursor-pointer"
			>
				Назад
			</button>
		</div>
	</div>
</div>
