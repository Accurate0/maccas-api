import { prisma } from '$lib/server/prisma.js';
import { Role } from '@prisma/client';
import { redirect } from '@sveltejs/kit';

export const load = async (event) => {
	const user = await prisma.user.findUniqueOrThrow({ where: { id: event.locals.session.userId } });
	if (user.role !== Role.ADMIN) {
		redirect(303, '/');
	}

	return {
		users: prisma.user
			.findMany()
			.then((p) => p.filter((u) => u.role !== Role.ADMIN))
			.then((p) => p.map((u) => ({ username: u.username, active: u.active, id: u.id })))
	};
};
