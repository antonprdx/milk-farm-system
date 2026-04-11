<script lang="ts">
	import { toasts } from '$lib/stores/toast';
</script>

<div class="fixed top-4 right-4 z-[100] flex flex-col gap-2 max-w-sm" aria-live="polite" aria-atomic="false">
	{#each $toasts as toast (toast.id)}
		<div
			class="px-4 py-3 rounded-lg shadow-lg text-sm font-medium flex items-center gap-2 animate-in {toast.type ===
			'success'
				? 'bg-green-600 text-white'
				: 'bg-red-600 text-white'}"
		>
			<span>{toast.type === 'success' ? '✓' : '✕'}</span>
			<span class="flex-1">{toast.message}</span>
			<button onclick={() => toasts.dismiss(toast.id)} class="hover:opacity-70 cursor-pointer"
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
