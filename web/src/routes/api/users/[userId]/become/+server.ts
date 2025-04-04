import { prisma } from '$lib/server/prisma';
import { createSession, SessionId } from '$lib/server/session.js';
import { validateAdminUser } from '$lib/server/validateAdminUser';

export async function POST(event) {
	const {
		params: { userId },
		locals,
		cookies
	} = event;

	const validationResult = await validateAdminUser(locals.session.userId);
	if (validationResult != null) {
		return validationResult;
	}

	const userToImpersonate = await prisma.user.findUnique({ where: { id: userId } });
	if (!userToImpersonate) {
		return new Response(null, { status: 400 });
	}

	const { sessionId, expires } = await createSession(
		userToImpersonate.id,
		userToImpersonate.role,
		locals.session.userId
	);

	cookies.set(SessionId, sessionId, { path: '/', httpOnly: true, expires });

	return new Response(null, { status: 204 });
}
