import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	build: {
		rollupOptions: {
			output: {
				manualChunks: {
					'chart.js': ['chart.js'],
				},
			},
		},
	},
	server: {
		proxy: {
			'/api': 'http://localhost:3000',
		},
	},
});
