import js from "@eslint/js";
import ts from "typescript-eslint";
import svelte from "eslint-plugin-svelte";
import globals from "globals";

/** @type {import('eslint').Linter.Config[]} */
export default [
	js.configs.recommended,
	...ts.configs.recommended,
	...svelte.configs["flat/recommended"],
	{
		files: ["**/*.svelte"],
		languageOptions: {
			parserOptions: {
				parser: ts.parser,
			},
		},
	},
	{
		languageOptions: {
			parserOptions: {
				project: true,
				tsconfigRootDir: import.meta.dirname,
				extraFileExtensions: [".svelte"],
			},
			globals: {
				...globals.browser,
				...globals.node,
			},
		},
	},
	{
		rules: {
			// Enforce exhaustive handling — no explicit-any in production code
			"@typescript-eslint/no-explicit-any": "error",
			// Unsafe operations must be handled
			"@typescript-eslint/no-floating-promises": "error",
			// Array index access returns T | undefined with noUncheckedIndexedAccess;
			// make sure we don't inadvertently use 'as' to silence it
			"@typescript-eslint/no-non-null-assertion": "error",
		},
	},
	{
		ignores: ["build/", ".svelte-kit/", "dist/", "src-tauri/", "docs/", "node_modules/"],
	},
];
