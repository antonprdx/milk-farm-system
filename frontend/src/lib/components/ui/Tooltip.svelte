<script lang="ts">
	let {
		text,
		class: cls = '',
		iconSize = 14,
	}: {
		text: string;
		class?: string;
		iconSize?: number;
	} = $props();

	let visible = $state(false);
	let x = $state(0);
	let y = $state(0);

	function show(e: MouseEvent | FocusEvent) {
		const el = e.currentTarget as HTMLElement;
		const rect = el.getBoundingClientRect();
		x = rect.left + rect.width / 2;
		y = rect.bottom + 6;
		visible = true;
	}

	function hide() {
		visible = false;
	}
</script>

<span
	class="inline-flex items-center cursor-help text-slate-400 hover:text-slate-500 dark:text-slate-500 dark:hover:text-slate-400 {cls}"
	onmouseenter={show}
	onmouseleave={hide}
	onfocus={(e: FocusEvent) => show(e)}
	onblur={hide}
	role="button"
	aria-label={text}
	tabindex="0"
>
	<svg
		xmlns="http://www.w3.org/2000/svg"
		width={iconSize}
		height={iconSize}
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="2"
		stroke-linecap="round"
		stroke-linejoin="round"
	>
		<circle cx="12" cy="12" r="10" />
		<path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" />
		<path d="M12 17h.01" />
	</svg>
</span>

{#if visible}
	<div
		class="fixed z-50 max-w-xs px-3 py-2 text-xs font-normal leading-relaxed text-white bg-slate-800 dark:bg-slate-700 rounded-lg shadow-lg pointer-events-none"
		style="left: {x}px; top: {y}px; transform: translateX(-50%);"
	>
		{text}
		<div
			class="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-slate-800 dark:bg-slate-700 rotate-45"
		></div>
	</div>
{/if}
