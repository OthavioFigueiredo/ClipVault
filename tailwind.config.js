/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        vault: {
          950: '#0b0f17',
          900: '#111827',
          850: '#161f33',
          800: '#1e293b',
          750: '#263449',
          700: '#334155',
          600: '#475569',
          500: '#64748b',
          400: '#94a3b8',
          accent: '#6366f1',
          accentHover: '#4f46e5',
          cyan: '#06b6d4',
          danger: '#ef4444',
          success: '#10b981',
          warning: '#f59e0b',
        }
      },
      fontFamily: {
        sans: ['Inter', 'Ubuntu', 'system-ui', '-apple-system', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'monospace']
      }
    },
  },
  plugins: [],
}
