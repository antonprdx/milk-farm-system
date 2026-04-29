<script lang="ts">
	import { onMount } from 'svelte';
	import { WifiOff } from 'lucide-svelte';

	let offline = $state(false);

	onMount(() => {
		offline = !navigator.onLine;
		const onOff = () => (offline = true);
		const onOn = () => (offline = false);
		window.addEventListener('offline', onOff);
		window.addEventListener('online', onOn);
		return () => {
			window.removeEventListener('offline', onOff);
			window.removeEventListener('online', onOn);
		};
	});
</script>

{#if offline}
	<div class="fixed bottom-4 left-1/2 -translate-x-1/2 z-50 bg-yellow-500 text-white px-4 py-2 rounded-lg shadow-lg flex items-center gap-2 text-sm font-medium">
		<WifiOff class="w-4 h-4" />
		Нет подключения к интернету
	</div>
{/if}
