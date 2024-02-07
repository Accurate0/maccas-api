import { prisma } from '$lib/prisma';
import { Role } from '@prisma/client';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async (event) => {
	const isLoginOrRegister =
		event.url.pathname === '/login' ||
		event.url.pathname === '/register' ||
		event.url.pathname === '/inactive';

	if (!isLoginOrRegister) {
		const user = await prisma.user.findUniqueOrThrow({
			where: { id: event.locals.session.userId },
			include: { config: true }
		});

		return {
			showPoints: user.role === Role.ADMIN || user.role === Role.PRIVILEGED,
			config: user.config,
			isUserActive: user.active
		};
	}

	return {
		hideAll: true
	};
};
