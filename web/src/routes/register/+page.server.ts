import { prisma } from '$lib/server/prisma';
import type { Actions } from './$types';
import bcrypt from 'bcrypt';
import { NotificationType, Priority, Role } from '@prisma/client';
import { fail } from '@sveltejs/kit';
import { message, setError, superValidate } from 'sveltekit-superforms/server';
import { schema } from './schema';
import { RateLimiter } from '$lib/server/ratelimiter';
import { zod } from 'sveltekit-superforms/adapters';
import { createSession } from '$lib/server/session';
import { featureFlagClient } from '$lib/server/featureflag';

export type RegisterState = {
	error: string | null;
};

export const load = async (event) => {
	await RateLimiter.cookieLimiter?.preflight(event);
	const form = await superValidate(zod(schema));
	return { form };
};

export const actions = {
	default: async (event) => {
		const { request, cookies } = event;

		const form = await superValidate(request, zod(schema));
		const { limited, retryAfter } = await RateLimiter.check(event);
		console.log(`Rate limiter check: ${event.getClientAddress()} ${limited}`);

		if (limited) {
			await prisma.notification.create({
				data: {
					content: `Rate limit reached by ${event.getClientAddress()}`,
					context: { ipAddress: event.getClientAddress(), page: 'register' },
					read: false,
					priority: Priority.HIGH,
					type: NotificationType.RATELIMIT_REACHED
				}
			});

			return setError(
				form,
				'password',
				`Too many attempts, try again after ${retryAfter} seconds`,
				{
					status: 429
				}
			);
		}

		if (!form.valid) {
			return fail(400, { form });
		}

		const { username: usernameUntrimmed, password: passwordUntrimmed } = form.data;
		const username = usernameUntrimmed.trim();
		if (await prisma.user.findUnique({ where: { username } })) {
			return setError(form, 'password', 'Username already exists');
		}

		const password = passwordUntrimmed.trim();
		const passwordHash = await bcrypt.hash(password, 10);
		const isUserActive = await featureFlagClient.getBooleanValue(
			'maccas-web-allow-active-registration',
			false
		);

		console.log('evaluated ff for active:', isUserActive);

		const user = await prisma.user.create({
			data: {
				username: username.toLowerCase(),
				passwordHash: Buffer.from(passwordHash),
				role: [Role.USER],
				active: isUserActive
			}
		});

		await prisma.notification.create({
			data: {
				content: `New user created ${username}`,
				context: { username },
				read: false,
				priority: Priority.NORMAL,
				type: NotificationType.USER_CREATED
			}
		});

		await createSession(user.id, user.role, cookies);

		return message(form, 'Account created');
	}
} satisfies Actions;
