<script module lang="ts">
	import type { ClusterResponse } from '$lib/api/analytics';
</script>

<script lang="ts">
	import { ensureChartRegistered, Chart } from '$lib/utils/chartRegistration';
	import { theme } from '$lib/stores/theme';
	import { themeColors, defaultTooltip } from '$lib/utils/chartHelpers';
	import { debounce } from '$lib/utils/debounce';

	ensureChartRegistered();

	interface Props {
		data: ClusterResponse;
	}

	let { data }: Props = $props();

	let canvas: HTMLCanvasElement | undefined = $state();
	let chart: Chart | null = null;

	const CLUSTER_COLORS = [
		{ bg: 'rgba(59,130,246,0.6)', border: 'rgba(59,130,246,0.9)' },
		{ bg: 'rgba(16,185,129,0.6)', border: 'rgba(16,185,129,0.9)' },
		{ bg: 'rgba(245,158,11,0.6)', border: 'rgba(245,158,11,0.9)' },
		{ bg: 'rgba(239,68,68,0.6)', border: 'rgba(239,68,68,0.9)' },
		{ bg: 'rgba(139,92,246,0.6)', border: 'rgba(139,92,246,0.9)' },
		{ bg: 'rgba(236,72,153,0.6)', border: 'rgba(236,72,153,0.9)' },
		{ bg: 'rgba(20,184,166,0.6)', border: 'rgba(20,184,166,0.9)' },
		{ bg: 'rgba(249,115,22,0.6)', border: 'rgba(249,115,22,0.9)' },
	];

	function buildChart() {
		if (!canvas) return;

		if (chart) {
			chart.destroy();
			chart = null;
		}

		const isDark = $theme === 'dark';
		const { gridColor, textColor } = themeColors(isDark);

		const grouped = new Map<number, typeof data.clusters>();
		for (const c of data.clusters) {
			if (!grouped.has(c.cluster_id)) grouped.set(c.cluster_id, []);
			grouped.get(c.cluster_id)!.push(c);
		}

		const datasets = [...grouped.entries()].map(([cid, items]) => {
			const colorIdx = cid % CLUSTER_COLORS.length;
			return {
				label: data.cluster_names[String(cid)] ?? `Кластер ${cid}`,
				data: items.map((c) => ({ x: c.avg_milk, y: c.avg_rumination, name: c.animal_name ?? `#${c.animal_id}` })),
				backgroundColor: CLUSTER_COLORS[colorIdx].bg,
				borderColor: CLUSTER_COLORS[colorIdx].border,
				borderWidth: 1,
				pointRadius: 6,
				pointHoverRadius: 8,
			};
		});

		chart = new Chart(canvas, {
			type: 'scatter',
			data: { datasets },
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: {
						position: 'top',
						labels: { color: textColor, usePointStyle: true, padding: 16, font: { size: 12 } },
					},
					tooltip: defaultTooltip(isDark, {
						label: (ctx: unknown) => {
							const c = ctx as { parsed: { x: number; y: number }; raw: { name: string } };
							return `${c.raw.name}: надой ${c.parsed.x.toFixed(1)} л, жвачка ${c.parsed.y.toFixed(0)} мин`;
						},
					}),
				},
				scales: {
					x: {
						title: { display: true, text: 'Средний надой (л)', color: textColor, font: { size: 12 } },
						grid: { color: gridColor },
						ticks: { color: textColor },
					},
					y: {
						title: { display: true, text: 'Средняя жвачка (мин)', color: textColor, font: { size: 12 } },
						grid: { color: gridColor },
						ticks: { color: textColor },
					},
				},
			},
		});
	}

	let debouncedBuild = debounce(() => buildChart(), 50);

	$effect(() => {
		data;
		$theme;
		debouncedBuild();
	});

	$effect(() => {
		return () => {
			if (chart) {
				chart.destroy();
				chart = null;
			}
		};
	});
</script>

<div class="relative h-80">
	<canvas bind:this={canvas}></canvas>
</div>
