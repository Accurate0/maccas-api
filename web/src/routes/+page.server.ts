import { IndexStore } from '$houdini';
import { Role } from '@prisma/client';
import type { PageServerLoad } from './$houdini';
import { prisma } from '$lib/prisma';

export const load: PageServerLoad = async (event) => {
	const user = await prisma.user.findUniqueOrThrow({
		where: { id: event.locals.session.userId },
		include: { config: true }
	});

	const index = new IndexStore();
	const { data } = await index.fetch({
		event,
		variables: {
			includePoints: user.role === Role.ADMIN || user.role === Role.PRIVILEGED,
			minimumCurrentPoints: 2500
		}
	});

	return { offersList: data?.offers, pointsList: data?.points, config: user.config };
};
