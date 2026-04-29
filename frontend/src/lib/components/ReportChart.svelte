<script lang="ts">
	import { onMount } from 'svelte';
	import { theme } from '$lib/stores/theme';
	import { ensureChartRegistered, Chart } from '$lib/utils/chartRegistration';

	ensureChartRegistered();

	let { labels, datasets, type = 'line', height = '250px' }: {
		labels: string[];
		datasets: { label: string; data: (number | null)[]; color?: string; fill?: boolean }[];
		type?: 'line' | 'bar';
		height?: string;
	} = $props();

	let canvas: HTMLCanvasElement;
	let chart: Chart | null = null;

	function buildChart() {
		if (chart) chart.destroy();
		const isDark = $theme === 'dark';
		const gridColor = isDark ? 'rgba(148,163,184,0.15)' : 'rgba(100,116,139,0.12)';
		const textColor = isDark ? '#94a3b8' : '#64748b';

		chart = new Chart(canvas, {
			type,
			data: {
				labels,
				datasets: datasets.map((ds, i) => {
					const color = ds.color || (i === 0 ? '#2563eb' : i === 1 ? '#f59e0b' : '#10b981');
					return {
						label: ds.label,
						data: ds.data,
						borderColor: color,
						backgroundColor: ds.fill !== false && type === 'line' ? color + '20' : type === 'bar' ? color + '90' : undefined,
						fill: ds.fill !== false && type === 'line',
						tension: 0.3,
						pointRadius: type === 'line' ? 2 : 0,
						borderWidth: 2,
					};
				}),
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: { display: datasets.length > 1, labels: { color: textColor, boxWidth: 12 } },
					tooltip: { mode: 'index', intersect: false },
				},
				scales: {
					x: { grid: { color: gridColor }, ticks: { color: textColor, maxTicksLimit: 15 } },
					y: { grid: { color: gridColor }, ticks: { color: textColor } },
				},
			},
		});
	}

	onMount(() => {
		buildChart();
		return () => { if (chart) chart.destroy(); };
	});

	$effect(() => {
		$theme;
		if (canvas) buildChart();
	});
</script>

<div style="height: {height}; width: 100%;">
	<canvas bind:this={canvas}></canvas>
</div>
