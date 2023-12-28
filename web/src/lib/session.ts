import type { Session, User } from '@prisma/client';
import type { Cookies } from '@sveltejs/kit';
import { prisma } from './prisma';

export const SessionId = 'session-id';

export const getSession = async (cookies: Cookies): Promise<Session> => {
	return prisma.session.findUniqueOrThrow({ where: { id: cookies.get(SessionId) } });
};

export const getUser = async (cookies: Cookies): Promise<User> => {
	return prisma.user.findUniqueOrThrow({ where: { id: (await getSession(cookies)).userId } });
};
