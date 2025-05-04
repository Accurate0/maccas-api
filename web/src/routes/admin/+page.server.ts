import { prisma } from '$lib/server/prisma.js';
import { env } from '$env/dynamic/private';
import { Role } from '@prisma/client';
import { redirect } from '@sveltejs/kit';

interface EventsResponse {
	events: string[];
}

export const load = async (event) => {
	const user = await prisma.user.findUniqueOrThrow({ where: { id: event.locals.session.userId } });
	if (!user.role.some((role) => role === Role.ADMIN)) {
		redirect(303, '/');
	}

	const { locals } = event;

	const allEvents = await fetch(`${env.EVENT_BASE_URL}/event/all`, {
		headers: {
			Authorization: `Bearer ${locals.session.accessToken}`
		}
	})
		.then((r) => r.json())
		.then((b) => b as EventsResponse)
		.then((e) => e.events);

	return {
		notifications: prisma.notification.findMany({ orderBy: { createdAt: 'desc' } }),
		events: allEvents,
		users: prisma.user
			.findMany()
			.then((p) => p.filter((u) => !u.role.some((role) => role === Role.ADMIN)))
			.then((p) =>
				p.map((u) => ({ username: u.username, active: u.active, id: u.id, roles: u.role }))
			)
	};
};
