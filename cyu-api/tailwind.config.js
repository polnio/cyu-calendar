/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./assets/views/**/*.hbs"],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: ["light", "light"],
  },
};
