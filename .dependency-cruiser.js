/** @type {import('dependency-cruiser').IConfiguration} */
module.exports = {
	forbidden: [
		{
			name: "no-circular",
			severity: "warn",
			comment:
				"This dependency is part of a circular relationship. You might want to revise " +
				"your solution (i.e. use dependency inversion, make sure the modules have a single responsibility).",
			from: {},
			to: {
				circular: true,
			},
		},
		{
			name: "no-orphans",
			comment:
				"This is an orphan module - it's likely not used (anymore?). Either use it or " +
				"remove it. If it's logical this module is an orphan (i.e. it's a config file), " +
				"add an exception for it in your dependency-cruiser configuration.",
			severity: "warn",
			from: {
				orphan: true,
				pathNot: [
					"(^|/)[.][^/]+[.](?:js|cjs|mjs|ts|cts|mts|json)$", // dot files
					"[.]d[.]ts$", // TypeScript declaration files
					"(^|/)tsconfig[.]json$", // TypeScript config
					"(^|/)(?:babel|webpack)[.]config[.](?:js|cjs|mjs|ts|cts|mts|json)$",
					"(^|/)vite[.]config[.](?:js|ts)$", // vite config
					"(^|/)svelte[.]config[.](?:js|ts)$", // svelte config
					"(^|/)eslint[.]config[.](?:js|cjs|mjs)$", // eslint config
					"(^|/)app[.](?:html|css)$", // SvelteKit app shell
					"(^|/)[+]layout[.](?:ts|svelte)$", // SvelteKit layout roots
					"(^|/)[+]page[.](?:ts|svelte)$", // SvelteKit page roots
				],
			},
			to: {},
		},
		{
			name: "no-deprecated-core",
			comment:
				"A module depends on a node core module that has been deprecated. Find an alternative.",
			severity: "warn",
			from: {},
			to: {
				dependencyTypes: ["core"],
				path: ["^punycode$", "^domain$", "^querystring$"],
			},
		},
		{
			name: "no-non-package-json",
			severity: "error",
			comment:
				"Don't allow dependencies to packages not in package.json. This catch missing deps early.",
			from: {},
			to: {
				dependencyTypes: [
					"unknown",
					"undetermined",
					"npm-no-pkg",
					"npm-unknown",
				],
			},
		},
		{
			name: "not-to-unresolvable",
			comment: "Don't allow dependencies to modules that cannot be resolved.",
			severity: "error",
			from: {},
			to: {
				couldNotResolve: true,
			},
		},
		{
			name: "no-duplicate-dep-types",
			comment:
				"Warn when a dependency appears in both dependencies and devDependencies in package.json.",
			severity: "warn",
			from: {},
			to: {
				moreThanOneDependencyType: true,
				dependencyTypesNot: ["type-only"],
			},
		},
	],
	options: {
		doNotFollow: {
			path: "node_modules",
		},
		tsPreCompilationDeps: true,
		tsConfig: {
			fileName: "tsconfig.json",
		},
		reporterOptions: {
			dot: {
				collapsePattern: "node_modules/[^/]+",
			},
			archi: {
				collapsePattern: "^(node_modules|src/lib|src/routes)/[^/]+",
			},
		},
	},
};
