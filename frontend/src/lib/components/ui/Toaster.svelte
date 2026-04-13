<script lang="ts">
	import { toasts } from '$lib/stores/toast';

	let expanded = $state<Set<number>>(new Set());

	function toggle(id: number) {
		const s = new Set(expanded);
		if (s.has(id)) s.delete(id); else s.add(id);
		expanded = s;
	}

	const colors: Record<string, string> = {
		success: 'bg-green-600 text-white',
		error: 'bg-red-600 text-white',
		warning: 'bg-yellow-500 text-white',
		info: 'bg-blue-600 text-white',
	};
	const icons: Record<string, string> = {
		success: '✓',
		error: '✕',
		warning: '⚠',
		info: 'ℹ',
	};
</script>

<div
	class="fixed top-4 right-4 z-[100] flex flex-col gap-2 max-w-sm"
	aria-live="polite"
	aria-atomic="false"
>
	{#each $toasts as toast (toast.id)}
		<div class="px-4 py-3 rounded-lg shadow-lg text-sm font-medium flex items-start gap-2 animate-in {colors[toast.type] || colors.info}">
			<span class="mt-0.5">{icons[toast.type] || 'ℹ'}</span>
			<div class="flex-1">
				<span>{toast.message}</span>
				{#if toast.details && toast.details.length > 0}
					<button
						onclick={() => toggle(toast.id)}
						class="block text-xs opacity-80 hover:opacity-100 mt-1 underline cursor-pointer"
					>
						{expanded.has(toast.id) ? 'Скрыть' : `${toast.details.length} ошибок`}
					</button>
					{#if expanded.has(toast.id)}
						<ul class="text-xs mt-1 list-disc ml-3 max-h-32 overflow-y-auto opacity-90">
							{#each toast.details as d}
								<li>{d}</li>
							{/each}
						</ul>
					{/if}
				{/if}
			</div>
			<button onclick={() => toasts.dismiss(toast.id)} class="hover:opacity-70 cursor-pointer mt-0.5"
				>✕</button
			>
		</div>
	{/each}
</div>

<style>
	.animate-in {
		animation: slide-in 0.2s ease-out;
	}
	@keyframes slide-in {
		from {
			transform: translateX(100%);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}
</style>
