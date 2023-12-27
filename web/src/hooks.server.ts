import { SessionId } from '$lib';
import { prisma } from '$lib/prisma';
import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
	if (event.url.pathname !== '/login') {
		const sessionId = event.cookies.get(SessionId);
		if (!sessionId) {
			return new Response(null, {
				status: 307,
				headers: { location: '/login' }
			});
		}

		const session = await prisma.session.findUnique({ where: { id: sessionId } });
		if (!session || new Date() > session.expires) {
			return new Response(null, {
				status: 307,
				headers: { location: '/login' }
			});
		}
	}

	return await resolve(event);
};
