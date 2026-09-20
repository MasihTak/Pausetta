import js from "@eslint/js";
import pluginVue from "eslint-plugin-vue";

export default [
  {
    ignores: ["node_modules", "dist", "src-tauri/target", "coverage"],
  },

  js.configs.recommended,

  ...pluginVue.configs["flat/recommended"],

  {
    languageOptions: {
      globals: {
        process: "readonly",
        fetch: "readonly",
        localStorage: "readonly",
        setInterval: "readonly",
        clearInterval: "readonly",
        setTimeout: "readonly",
        clearTimeout: "readonly",
        console: "readonly",
        window: "readonly",
        document: "readonly",
        globalThis: "readonly",
      },
    },
    files: ["**/*.{js,vue}"],
    rules: {
      "no-console": "warn",
      "no-debugger": "error",
      "vue/no-unused-vars": "error",
      "vue/max-attributes-per-line": "error",
      "vue/component-name-in-template-casing": [
        "error",
        "PascalCase",
        {
          registeredComponentsOnly: true,
          ignores: [],
          globals: [
            "RouterView",
            "Teleport",
            "Component",
            "Transition",
            "TransitionGroup",
          ],
        },
      ],
      "vue/html-self-closing": [
        "error",
        {
          html: {
            void: "always",
            normal: "always",
          },
        },
      ],
    },
  },

  {
    // Tests run in jsdom, so they reach for browser globals the app itself never needs,
    // and assert on console output the app deliberately writes.
    files: ["tests/**/*.js"],
    languageOptions: {
      globals: {
        KeyboardEvent: "readonly",
      },
    },
    rules: {
      "no-console": "off",
    },
  },
];
