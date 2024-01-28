/// <references types="houdini-svelte">

/** @type {import('houdini').ConfigFile} */
const config = {
	watchSchema: {
		url: 'http://localhost:8000/v1/graphql',
		interval: 2000
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
