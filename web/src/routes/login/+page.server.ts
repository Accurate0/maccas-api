import { prisma } from '$lib/prisma';
import type { Actions } from './$types';
import { z } from 'zod';
import bcrypt from 'bcrypt';
import type { Role } from '@prisma/client';
import { randomBytes } from 'crypto';
import { SessionId } from '$lib';
import { redirect } from '@sveltejs/kit';

const schema = z.object({
	username: z.string().min(1),
	password: z.string().min(1)
});

export type LoginState = {
	error: string | null;
};

const legacyLoginResponseSchema = z.object({
	token: z.string().min(1),
	refreshToken: z.string().min(1),
	role: z.union([z.literal('admin'), z.literal('privileged'), z.literal('none')])
});

export const actions = {
	default: async ({ request, fetch, cookies }) => {
		const createSession = async (userId: string) => {
			const sessionId = randomBytes(30).toString('base64');
			const sevenDaysInMs = 604800000;
			const expires = new Date(Date.now() + sevenDaysInMs);

			await prisma.session.create({
				data: {
					userId,
					id: sessionId,
					expires
				}
			});

			cookies.set(SessionId, sessionId, { path: '/', httpOnly: true, expires });
		};

		const formData = await request.formData();
		const validatedFields = schema.safeParse({
			username: formData.get('username'),
			password: formData.get('password')
		});

		if (!validatedFields.success) {
			return {
				error: 'Invalid details'
			};
		}

		const { username, password } = validatedFields.data;
		const existingUser = await prisma.user.findUnique({ where: { username } });
		if (existingUser) {
			const isPasswordCorrect = await bcrypt.compare(
				password,
				existingUser.passwordHash.toString()
			);

			if (!isPasswordCorrect) {
				return {
					error: 'Invalid details'
				};
			}

			await createSession(existingUser.id);
		} else {
			// FIXME: will need to be old.api.maccas.one or something
			const response = await fetch('https://api.maccas.one/v1/auth/login', {
				method: 'POST',
				body: formData
			});

			if (!response.ok) {
				return {
					error: 'Invalid details'
				};
			}

			const result = await legacyLoginResponseSchema.safeParseAsync(await response.json());

			if (!result.success) {
				return {
					error: 'Invalid details'
				};
			}

			const { role, token } = result.data;

			const existingUserId = JSON.parse(atob(token.split('.')[1] ?? ''))['oid'] as string;

			const passwordHash = await bcrypt.hash(password, 10);
			// TODO: fetch existing config
			// https://api.maccas.one/v1/user/config
			await prisma.user.create({
				data: {
					id: existingUserId,
					username: username,
					passwordHash: Buffer.from(passwordHash),
					// the prisma one is just uppercase, this should be fine
					role: role.toUpperCase() as Role
				}
			});

			await createSession(existingUserId);
		}

		redirect(303, '/offers');
	}
} satisfies Actions;