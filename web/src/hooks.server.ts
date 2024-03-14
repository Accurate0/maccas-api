import { prisma } from '$lib/server/prisma';
import type { Handle } from '@sveltejs/kit';
import { setSession } from '$houdini';
import { SessionId } from '$lib/server/session';
import '$lib/server/opentelemetry';
import type { HandleFetch } from '@sveltejs/kit';
import opentelemetry, { SpanStatusCode, type Span } from '@opentelemetry/api';
import { IMAGE_CDN } from '$lib/server/constants';

export const handle: Handle = async ({ event, resolve }) => {
	// don't query db for this... public images...
	if (event.url.pathname.startsWith('/api/images')) {
		return await resolve(event);
	}

	const tracer = opentelemetry.trace.getTracer('default');

	return tracer.startActiveSpan(`fetch ${event.url.pathname}`, async (span) => {
		if (event.url.pathname !== '/login' && event.url.pathname !== '/register') {
			const sessionId = event.cookies.get(SessionId);
			if (!sessionId) {
				span.setStatus({ code: SpanStatusCode.ERROR });
				span.end();

				return new Response(null, {
					status: 307,
					headers: { location: '/login' }
				});
			}

			const session = await prisma.session.findUnique({ where: { id: sessionId } });
			if (!session || new Date() > session.expires) {
				span.setStatus({ code: SpanStatusCode.ERROR });
				span.end();

				return new Response(null, {
					status: 307,
					headers: { location: '/login' }
				});
			}

			event.locals.session = session;
			setSession(event, { ...session });
		}

		const response = await resolve(event);

		if (response.ok) {
			span.setStatus({ code: SpanStatusCode.OK });
		} else {
			span.setStatus({ code: SpanStatusCode.ERROR });
		}

		span.setAttribute('statusCode', response.status);

		span.end();
		return response;
	});
};

export const handleFetch: HandleFetch = async ({ request, fetch }) => {
	if (request.url.startsWith(IMAGE_CDN)) {
		return fetch(request);
	}

	const tracer = opentelemetry.trace.getTracer('default');

	return tracer.startActiveSpan(`fetch external ${request.url}`, async (span: Span) => {
		const output: { traceparent?: string; tracestate?: string } = {};
		opentelemetry.propagation.inject(opentelemetry.context.active(), output);

		const { traceparent, tracestate } = output;

		if (traceparent) {
			request.headers.set('traceparent', traceparent);
		}

		if (tracestate) {
			request.headers.set('tracestate', tracestate);
		}

		const response = await fetch(request);

		if (response.ok) {
			span.setStatus({ code: SpanStatusCode.OK });
		} else {
			span.setStatus({ code: SpanStatusCode.ERROR });
		}

		span.setAttribute('statusCode', response.status);

		span.end();
		return response;
	});
};
