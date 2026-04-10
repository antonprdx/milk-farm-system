<script lang="ts">
	let {
		message = 'Нет данных',
	}: {
		message?: string;
	} = $props();
</script>

<div class="flex flex-col items-center justify-center py-12 text-slate-400 dark:text-slate-500">
	<svg width="64" height="64" viewBox="0 0 64 64" fill="none" class="mb-4 opacity-40">
		<rect x="8" y="16" width="48" height="36" rx="4" stroke="currentColor" stroke-width="2" fill="none" />
		<line x1="8" y1="28" x2="56" y2="28" stroke="currentColor" stroke-width="2" />
		<circle cx="18" cy="22" r="2" fill="currentColor" />
		<circle cx="26" cy="22" r="2" fill="currentColor" />
		<line x1="20" y1="36" x2="44" y2="36" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
		<line x1="20" y1="42" x2="36" y2="42" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
	</svg>
	<p class="text-sm">{message}</p>
</div>
