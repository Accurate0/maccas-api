import { prisma } from '$lib/server/prisma';

export const load = async (event) => {
	const user = await prisma.user.findUniqueOrThrow({
		where: { id: event.locals.session.userId },
		include: { config: true }
	});

	const shouldShowDistance = await event.locals.featureFlagClient.getBooleanValue(
		'maccas-web-show-distance-from-store',
		false
	);

	return {
		config: user.config,
		shouldShowDistance
	};
};
