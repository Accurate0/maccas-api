import type { Session, User } from '@prisma/client';
import { prisma } from './prisma';

export const SessionId = 'session-id';

export const getSession = async (sessionId: string): Promise<Session> => {
	return prisma.session.findUniqueOrThrow({ where: { id: sessionId } });
};

export const getUser = async (sessionId: string): Promise<User> => {
	const session = await getSession(sessionId);
	return prisma.user.findUniqueOrThrow({ where: { id: session.userId } });
};
