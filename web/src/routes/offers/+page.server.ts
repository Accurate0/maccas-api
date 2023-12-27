import { GetOffersStore } from '$houdini';
import type { PageServerLoad } from './$houdini';

export const load: PageServerLoad = async (event) => {
	const getOffers = new GetOffersStore();
	const { data } = await getOffers.fetch({ event });

	return data;
};
