<script lang="ts">
	let {
		open = false,
		title = 'Подтвердите действие',
		message = '',
		confirmText = 'Удалить',
		loading = false,
		destructive = true,
		onconfirm,
		oncancel,
	}: {
		open?: boolean;
		title?: string;
		message?: string;
		confirmText?: string;
		loading?: boolean;
		destructive?: boolean;
		onconfirm: () => void;
		oncancel: () => void;
	} = $props();

	let dialogRef: HTMLDivElement | undefined = $state();
	let previousFocus: HTMLElement | null = $state(null);
	let confirmBtn: HTMLButtonElement | undefined = $state();

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && open) {
			oncancel();
			return;
		}
		if (e.key === 'Tab' && dialogRef) {
			const focusable = Array.from(
				dialogRef.querySelectorAll<HTMLElement>('button:not([disabled])'),
			);
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
			previousFocus = document.activeElement as HTMLElement;
			window.addEventListener('keydown', handleKeydown);
			return () => {
				window.removeEventListener('keydown', handleKeydown);
				if (previousFocus) previousFocus.focus();
			};
		}
	});

	$effect(() => {
		if (open && confirmBtn) {
			setTimeout(() => confirmBtn!.focus(), 0);
		}
	});
</script>

{#if open}
	<div
		class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
		role="dialog"
		aria-modal="true"
	>
		<div
			bind:this={dialogRef}
			class="bg-white dark:bg-slate-800 rounded-xl shadow-xl p-6 max-w-sm w-full mx-4"
		>
			<h3 class="text-lg font-semibold text-slate-800 dark:text-slate-100 mb-2">{title}</h3>
			{#if message}
				<p class="text-sm text-slate-500 dark:text-slate-400 mb-4">{message}</p>
			{/if}
			<div class="flex gap-3 justify-end">
				<button
					onclick={oncancel}
					class="px-4 py-2 text-sm border border-slate-300 dark:border-slate-600 rounded-lg hover:bg-slate-50 dark:bg-slate-800/50 cursor-pointer"
					>Отмена</button
				>
				<button
					bind:this={confirmBtn}
					onclick={onconfirm}
					disabled={loading}
					class="px-4 py-2 text-sm {destructive
						? 'bg-red-600 hover:bg-red-700'
						: 'bg-blue-600 hover:bg-blue-700'} text-white rounded-lg disabled:opacity-50 cursor-pointer"
				>
					{loading ? 'Выполняется...' : confirmText}
				</button>
			</div>
		</div>
	</div>
{/if}
