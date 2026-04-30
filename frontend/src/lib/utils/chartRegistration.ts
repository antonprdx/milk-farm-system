import {
	Chart,
	BarController,
	BarElement,
	CategoryScale,
	Filler,
	Legend,
	LinearScale,
	LineController,
	LineElement,
	PointElement,
	ScatterController,
	Tooltip,
} from 'chart.js';

let registered = false;

export function ensureChartRegistered(): void {
	if (registered) return;
	Chart.register(
		BarController,
		CategoryScale,
		LinearScale,
		PointElement,
		LineElement,
		BarElement,
		Filler,
		Tooltip,
		Legend,
		LineController,
		ScatterController,
	);
	registered = true;
}

export { Chart };

