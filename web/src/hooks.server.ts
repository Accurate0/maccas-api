import { prisma } from '$lib/prisma';
import type { Handle } from '@sveltejs/kit';
import { setSession } from '$houdini';
import { SessionId } from '$lib/session';

export const handle: Handle = async ({ event, resolve }) => {
	// don't query db for this... public images...
	if (event.url.pathname.startsWith('/api/images')) {
		return await resolve(event);
	}

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

		event.locals.session = session;
		setSession(event, { ...session });
	}

	return await resolve(event);
};
