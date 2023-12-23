import type { Actions } from './$types';
import { fail } from '@sveltejs/kit';
// import { graphql } from '$houdini';

// const searchLocations = graphql(`
// 	query searchLocations($query: String!) {
// 		location {
// 			text(input: { query: $query }) {
// 				name
// 				storeNumber
// 				address {
// 					addressLine
// 				}
// 			}
// 		}
// 	}
// `);

export const actions = {
	searchLocation: async (event) => {
		const formData = await event.request.formData();
		const query = formData.get('query');

		if (!query) {
			return fail(400, { error: true });
		}

		// const { data } = await searchLocations.fetch({ event, variables: { query: query.toString() } });

		return null;
	}
} satisfies Actions;
