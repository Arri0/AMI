/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],

	theme: {
		extend: {
			fontSize: {
				'2xs': '0.65rem',
			},
			gridTemplateColumns: {
				'auto-1fr': 'auto 1fr'
			},
			gridTemplateRows: {
				'1fr-auto': '1fr auto',
				'auto-1fr': 'auto 1fr',
				'section': 'auto 1fr auto',
				'file-browser': 'auto auto 1fr auto'
			}
		}
	},

	plugins: [
		require('tailwindcss-animated'),
		require('tailwind-scrollbar'),
	]
};
