<script lang="ts">
	import type { Snippet } from 'svelte';
	import { enableUnloadGuard, disableUnloadGuard } from '$lib/utils/unloadGuard';
	import { X } from 'lucide-svelte';

	let {
		open = false,
		title = '',
		maxWidth = 'max-w-md',
		onclose,
		children,
		footer,
	}: {
		open?: boolean;
		title?: string;
		maxWidth?: string;
		onclose?: () => void;
		children: Snippet;
		footer?: Snippet;
	} = $props();

	let dialogRef: HTMLDivElement | undefined = $state();
	let previousFocus: HTMLElement | null = $state(null);
	let titleId = $derived(title ? 'modal-title' : undefined);

	function getFocusableElements(el: HTMLElement): HTMLElement[] {
		return Array.from(
			el.querySelectorAll<HTMLElement>(
				'a[href], button:not([disabled]), textarea, input, select, [tabindex]:not([tabindex="-1"])',
			),
		);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && open && onclose) {
			onclose();
			return;
		}
		if (e.key === 'Tab' && dialogRef) {
			const focusable = getFocusableElements(dialogRef);
			if (focusable.length === 0) return;
			const first = focusable[0];
			const last = focusable[focusable.length - 1];
			if (e.shiftKey && document.activeElement === first) {
				e.preventDefault();
				last.focus();
			} else if (!e.shiftKey && document.activeElement === last) {
				e.preventDefault();
				first.focus();
			}
		}
	}

	$effect(() => {
		if (open) {
			enableUnloadGuard();
			previousFocus = document.activeElement as HTMLElement;
			window.addEventListener('keydown', handleKeydown);
			return () => {
				disableUnloadGuard();
				window.removeEventListener('keydown', handleKeydown);
				if (previousFocus) previousFocus.focus();
			};
		}
	});

	$effect(() => {
		if (open && dialogRef) {
			const focusable = getFocusableElements(dialogRef);
			if (focusable.length > 0) {
				setTimeout(() => focusable[0].focus(), 0);
			}
		}
	});
</script>

{#if open}
	<div
		class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
		role="dialog"
		aria-modal="true"
		aria-labelledby={titleId}
		tabindex={-1}
		onclick={(e) => {
			if (e.target === e.currentTarget && onclose) onclose();
		}}
		onkeydown={(e) => {
			if (e.key === 'Escape' && onclose) onclose();
		}}
	>
		<div
			bind:this={dialogRef}
			class="bg-white dark:bg-slate-800 rounded-xl shadow-xl p-6 {maxWidth} w-full mx-4 max-h-[90vh] overflow-y-auto relative"
		>
			{#if title}
				<div class="flex items-center justify-between mb-4">
					<h3 id={titleId} class="text-lg font-semibold text-slate-800 dark:text-slate-100">{title}</h3>
					{#if onclose}
						<button
							onclick={onclose}
							class="p-1 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 cursor-pointer"
						>
							<X size={18} />
						</button>
					{/if}
				</div>
			{/if}
			{@render children()}
			{#if footer}
				<div class="flex gap-3 justify-end pt-4">
					{@render footer()}
				</div>
			{/if}
		</div>
	</div>
{/if}
