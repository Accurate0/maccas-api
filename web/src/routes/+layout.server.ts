import { prisma } from '$lib/prisma';
import { Role } from '@prisma/client';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async (event) => {
	const isLogin = event.route.id === '/login';

	if (!isLogin) {
		const user = await prisma.user.findUniqueOrThrow({
			where: { id: event.locals.session.userId },
			include: { config: true }
		});

		return {
			showPoints: user.role === Role.ADMIN || user.role === Role.PRIVILEGED,
			config: user.config
		};
	}

	return {
		hideAll: true
	};
};
