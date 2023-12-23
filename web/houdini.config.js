/// <references types="houdini-svelte">

/** @type {import('houdini').ConfigFile} */
const config = {
	watchSchema: {
		url: 'http://127.0.0.1:8000/graphql',
		headers: {
			Authorization(env) {
				return `Bearer ${env.AUTH_TOKEN}`;
			}
		},
		interval: 0
	},
	scalars: {
		UUID: {
			type: 'string'
		},
		NaiveDateTime: {
			type: 'Date'
		}
	},

	plugins: {
		'houdini-svelte': {}
	}
};

export default config;
