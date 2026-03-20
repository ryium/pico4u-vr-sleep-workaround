import { light, dark } from '@charcoal-ui/theme';
import { createTailwindConfig } from '@charcoal-ui/tailwind-config';

const customTheme = {
  ...light,
  color: {
    ...light.color,
    brand: "#81C784",
  },
};

const customDarkTheme = {
  ...dark,
  color: {
    ...dark.color,
    brand: "#81C784",
  },
};

/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  presets: [
    createTailwindConfig({
      version: 'v3',
      theme: {
        ':root': customTheme,
        '.dark': customDarkTheme,
      },
    }),
  ],
  theme: {
    extend: {
      borderRadius: {
        'none': '0',
        'sm': '0.125rem',
        DEFAULT: '0.25rem',
        'md': '0.375rem',
        'lg': '0.5rem',
        'xl': '0.75rem',
        '2xl': '1rem',
        '3xl': '1.5rem',
        'full': '9999px',
      },
    },
  },
};
