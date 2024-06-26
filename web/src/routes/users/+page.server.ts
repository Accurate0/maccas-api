import { prisma } from '$lib/server/prisma.js';
import { Role } from '@prisma/client';
import { redirect } from '@sveltejs/kit';

export const load = async (event) => {
	const user = await prisma.user.findUniqueOrThrow({ where: { id: event.locals.session.userId } });
	if (!user.role.some((role) => role === Role.ADMIN)) {
		redirect(303, '/');
	}

	return {
		notifications: prisma.notification.findMany({ orderBy: { createdAt: 'desc' } }),
		users: prisma.user
			.findMany()
			.then((p) => p.filter((u) => !u.role.some((role) => role === Role.ADMIN)))
			.then((p) =>
				p.map((u) => ({ username: u.username, active: u.active, id: u.id, roles: u.role }))
			)
	};
};
