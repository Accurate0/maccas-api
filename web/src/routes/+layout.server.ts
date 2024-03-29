import { prisma } from '$lib/server/prisma';
import { Role } from '@prisma/client';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async (event) => {
	const isLoginOrRegister =
		event.url.pathname === '/login' ||
		event.url.pathname === '/register' ||
		event.url.pathname === '/inactive';

	if (isLoginOrRegister) {
		return {
			hideAll: true
		};
	}

	const user = await prisma.user.findUniqueOrThrow({
		where: { id: event.locals.session.userId },
		include: { config: true }
	});

	return {
		showPoints: user.role.some((role) => role === Role.POINTS),
		showUsers: user.role.some((role) => role === Role.ADMIN),
		config: user.config,
		isUserActive: user.active
	};
};
