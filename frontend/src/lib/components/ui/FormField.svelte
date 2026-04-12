<script lang="ts">
	let {
		label,
		type = 'text',
		value = $bindable(undefined as string | number | undefined),
		placeholder = '',
		required = false,
		step,
		rows = 2,
		options = [],
		checked = $bindable(false),
		id = '',
		disabled = false,
		error = '',
		onblur,
	}: {
		label: string;
		type?: string;
		value?: string | number | undefined;
		placeholder?: string;
		required?: boolean;
		step?: string;
		rows?: number;
		options?: { value: string; label: string }[];
		checked?: boolean;
		id?: string;
		disabled?: boolean;
		error?: string;
		onblur?: (() => void) | undefined;
	} = $props();

	const baseClass =
		'w-full px-3 py-2 border rounded-lg text-sm focus:ring-2 focus:border-blue-500 outline-none transition-colors bg-white dark:bg-slate-900 text-slate-800 dark:text-slate-200';
	let inputClass = $derived(
		error
			? baseClass + ' border-red-300 dark:border-red-500 focus:ring-red-500'
			: baseClass + ' border-slate-300 dark:border-slate-600 focus:ring-blue-500',
	);
</script>

{#if type === 'checkbox'}
	<label class="flex items-center gap-2 text-sm font-medium text-slate-700 dark:text-slate-300">
		<input type="checkbox" bind:checked class="rounded" />
		{label}
	</label>
{:else if type === 'select'}
	<div>
		<label for={id} class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
			>{label}{required ? ' *' : ''}</label
		>
		<select {id} bind:value class={inputClass} {disabled} {onblur}>
			{#each options as opt (opt.value)}
				<option value={opt.value}>{opt.label}</option>
			{/each}
		</select>
		{#if error}
			<p class="mt-1 text-xs text-red-500 dark:text-red-400">{error}</p>
		{/if}
	</div>
{:else if type === 'textarea'}
	<div>
		<label for={id} class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
			>{label}{required ? ' *' : ''}</label
		>
		<textarea
			{id}
			bind:value
			{rows}
			{placeholder}
			{disabled}
			{onblur}
			class="{inputClass} resize-none"
		></textarea>
		{#if error}
			<p class="mt-1 text-xs text-red-500 dark:text-red-400">{error}</p>
		{/if}
	</div>
{:else}
	<div>
		<label for={id} class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1"
			>{label}{required ? ' *' : ''}</label
		>
		<input
			{id}
			{type}
			bind:value
			{placeholder}
			{required}
			{step}
			{disabled}
			{onblur}
			class={inputClass}
		/>
		{#if error}
			<p class="mt-1 text-xs text-red-500 dark:text-red-400">{error}</p>
		{/if}
	</div>
{/if}
