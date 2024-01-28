import { GetOffersStore } from '$houdini';

export const load = async (event) => {
	const index = new GetOffersStore();
	const data = index.fetch({
		event
	});

	return {
		offers: data.then((c) => c.data?.offers)
	};
};
