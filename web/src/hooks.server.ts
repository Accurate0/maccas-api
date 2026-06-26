import { prisma } from '$lib/server/prisma';
import type { Handle } from '@sveltejs/kit';
import { setSession } from '$houdini';
import { SessionId } from '$lib/server/session';
import '$lib/server/featureflag';
import { trace } from '@opentelemetry/api';
import { FeatureFlagClientFactory, type EvaluationContext } from '$lib/server/featureflag';
import { env } from '$env/dynamic/private';

export const handle: Handle = async ({ event, resolve }) => {
	// don't query db for this... public images...
	if (event.url.pathname.startsWith('/api/images') || event.url.pathname.startsWith('/favicon')) {
		return await resolve(event);
	}

	const span = trace.getActiveSpan();

	let evaluationContext: EvaluationContext = {};
	if (event.url.pathname !== '/login' && event.url.pathname !== '/register') {
		const sessionId = event.cookies.get(SessionId);
		span?.setAttribute('sessionId', sessionId ?? '(unknown)');

		if (!sessionId) {
			return new Response(null, {
				status: 307,
				headers: { location: '/login' }
			});
		}

		const session = await prisma.session.findUnique({
			where: { id: sessionId }
		});
		span?.setAttribute('userId', session?.userId ?? '(unknown)');

		if (!session || new Date() > session.expires) {
			return new Response(null, {
				status: 307,
				headers: { location: '/login' }
			});
		}

		// evaluate the impersonator over the real user
		evaluationContext = {
			targetingKey: session.impersonatorUserId ?? session.userId,
			user_id: session.impersonatorUserId ?? session.userId
		};
		event.locals.session = session;
		setSession(event, { ...session });
	}

	event.locals.featureFlagClient = FeatureFlagClientFactory.getClient({
		environment: env.NODE_ENV ?? 'development',
		...evaluationContext
	} satisfies EvaluationContext);

	return resolve(event);
};

