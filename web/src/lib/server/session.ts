import type { Role } from '@prisma/client';
import { randomBytes } from 'crypto';
import jwt from 'jsonwebtoken';
import { prisma } from './prisma';
import type { Cookies } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';

export const SessionId = 'session-id';

export const createSession = async (userId: string, role: Role[], cookies: Cookies) => {
	const sessionId = randomBytes(64).toString('base64');
	const sevenDaysInMs = 604800000;
	const expires = new Date(Date.now() + sevenDaysInMs);
	const accessToken = jwt.sign({ userId, sessionId, role }, env.AUTH_SECRET, {
		expiresIn: sevenDaysInMs / 1000,
		issuer: 'Maccas Web',
		audience: 'Maccas API',
		subject: 'Maccas API'
	});

	await prisma.session.create({
		data: {
			userId,
			id: sessionId,
			expires,
			accessToken
		}
	});

	cookies.set(SessionId, sessionId, { path: '/', httpOnly: true, expires });
};
