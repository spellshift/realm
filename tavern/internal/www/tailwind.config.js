/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{js,jsx,ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        gray: {
          950: '#030712',
        },
      },
      minWidth: {
        '128': '32rem',
      }
    },
  },
  plugins: [
  ],
}
