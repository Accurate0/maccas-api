/// <references types="houdini-svelte">

/** @type {import('houdini').ConfigFile} */
const config = {
	watchSchema: {
		url: 'http://localhost:8000/graphql',
		interval: 0
	},
	plugins: {
		'houdini-svelte': {}
	},
	scalars: {
		UUID: {
			type: 'string'
		},
		NaiveDateTime: {
			type: 'string'
		}
	}
};

export default config;
