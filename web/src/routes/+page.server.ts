import { IndexStore } from '$houdini';
import { getUser } from '$lib/session';
import { Role } from '@prisma/client';
import type { PageServerLoad } from './$houdini';

export const load: PageServerLoad = async (event) => {
	const user = await getUser(event.cookies);

	const index = new IndexStore();
	const { data } = await index.fetch({
		event,
		variables: {
			includePoints: user.role === Role.ADMIN || user.role === Role.PRIVILEGED
		}
	});

	return { offersList: data?.offers, pointsList: data?.points };
};
