import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { VitePWA } from 'vite-plugin-pwa';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit(),
		VitePWA({
			registerType: 'autoUpdate',
			manifest: false,
			workbox: {
				runtimeCaching: [
					{
						urlPattern: /^https?:\/\/.*\/api\/v1\/(health|healthz|readyz|stats|alerts\/summary).*/i,
						handler: 'NetworkFirst',
						options: {
							cacheName: 'api-fast',
							expiration: { maxEntries: 20, maxAgeSeconds: 60 },
						},
					},
					{
						urlPattern: /^https?:\/\/.*\/api\/v1\/(animals|milk|feed|reproduction|tasks|vet|inventory).*/i,
						handler: 'NetworkFirst',
						options: {
							cacheName: 'api-data',
							expiration: { maxEntries: 200, maxAgeSeconds: 300 },
							networkTimeoutSeconds: 5,
						},
					},
				],
				navigateFallback: '/',
				navigateFallbackDenylist: [/^\/api\//],
			},
			devOptions: { enabled: false },
		}),
	],
	build: {
		rollupOptions: {
			output: {
				manualChunks(id) {
					if (id.includes('node_modules/chart.js') || id.includes('node_modules/chartjs')) {
						return 'chart.js';
					}
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
