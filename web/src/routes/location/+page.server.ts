import { prisma } from '$lib/server/prisma';

export const load = async (event) => {
	const user = await prisma.user.findUniqueOrThrow({
		where: { id: event.locals.session.userId },
		include: { config: true }
	});

	return {
		config: user.config,
	};
};
