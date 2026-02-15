import type { Config } from 'tailwindcss'

const config: Config = {
  darkMode: 'class',
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        emerald: {
          50: '#f0fdf4',
          100: '#dcfce7',
          200: '#bbf7d0',
          300: '#86efac',
          400: '#4ade80',
          500: '#22c55e',
          600: '#16a34a',
          700: '#15803d',
          800: '#166534',
          900: '#145231',
        },
      },
      fontSize: {
        display: ['32px', { lineHeight: '1.2', fontWeight: '700' }],
        heading: ['20px', { lineHeight: '1.3', fontWeight: '600' }],
        body: ['14px', { lineHeight: '1.5', fontWeight: '400' }],
        mono: ['12px', { fontFamily: 'ui-monospace', fontWeight: '500' }],
      },
      spacing: {
        xs: '4px',
        s: '8px',
        m: '16px',
        l: '24px',
        xl: '32px',
        xxl: '48px',
      },
    },
  },
  plugins: [],
}

export default config
