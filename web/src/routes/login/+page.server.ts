import { prisma } from '$lib/server/prisma';
import type { Actions } from './$types';
import { z } from 'zod';
import bcrypt from 'bcrypt';
import { NotificationType, Priority, Role } from '@prisma/client';
import { fail, redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { setError, superValidate } from 'sveltekit-superforms/server';
import { RateLimiter } from '$lib/server/ratelimiter';
import { schema } from './schema';
import { zod } from 'sveltekit-superforms/adapters';
import { createSession } from '$lib/server/session';

export type LoginState = {
	error: string | null;
};

const legacyLoginResponseSchema = z.object({
	token: z.string().min(1),
	refreshToken: z.string().min(1),
	role: z.union([z.literal('admin'), z.literal('privileged'), z.literal('none')])
});

const configSchema = z.object({
	storeId: z.string().min(1),
	storeName: z.string().min(1)
});

type OldRole = z.infer<typeof legacyLoginResponseSchema>['role'];
const roleMap = {
	none: Role.USER,
	privileged: Role.POINTS,
	admin: Role.ADMIN
} as const satisfies Record<OldRole, Role>;

export const load = async (event) => {
	await RateLimiter.cookieLimiter?.preflight(event);
	const form = await superValidate(zod(schema));
	return { form };
};

export const actions = {
	default: async (event) => {
		const { request, fetch, cookies } = event;
		const form = await superValidate(request, zod(schema));

		const { limited, retryAfter } = await RateLimiter.check(event);
		console.log(`Rate limiter check: ${event.getClientAddress()} ${limited}`);
		if (limited) {
			await prisma.notification.create({
				data: {
					content: `Rate limit reached by ${event.getClientAddress()}`,
					context: { ipAddress: event.getClientAddress(), page: 'login' },
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
		const password = passwordUntrimmed.trim();

		const existingUser = await prisma.user.findFirst({
			where: { username: { equals: username, mode: 'insensitive' } }
		});

		if (existingUser) {
			const isPasswordCorrect = await bcrypt.compare(
				password,
				existingUser.passwordHash.toString()
			);

			if (!isPasswordCorrect) {
				return setError(form, 'password', 'Invalid details');
			}

			await createSession(existingUser.id, existingUser.role, cookies);
		} else {
			// FIXME: will need to be old.api.maccas.one or something
			const formData = new FormData();
			formData.set('username', username);
			formData.set('password', password);

			const response = await fetch(`${env.OLD_API_BASE_URL}/auth/login`, {
				method: 'POST',
				body: formData
			});

			if (!response.ok) {
				return setError(form, 'password', 'Invalid details');
			}

			const result = await legacyLoginResponseSchema.safeParseAsync(await response.json());
			if (!result.success) {
				return setError(form, 'password', 'Invalid details');
			}

			const { role, token } = result.data;
			const existingUserId = JSON.parse(atob(token.split('.')[1] ?? ''))['oid'] as string;
			const configResponse = await fetch(`${env.OLD_API_BASE_URL}/user/config`, {
				method: 'GET',
				headers: { Authorization: `Bearer ${result.data.token}` }
			});

			const config = configResponse.ok
				? await configSchema.safeParseAsync(await configResponse.json())
				: null;

			const passwordHash = await bcrypt.hash(password, 10);
			// TODO: fetch existing config
			// https://api.maccas.one/v1/user/config

			const configParsed = config?.success
				? { storeId: config.data.storeId, storeName: config.data.storeName }
				: {};

			const newRole = roleMap[role];

			await prisma.user.create({
				data: {
					id: existingUserId,
					username: username.toLowerCase(),
					passwordHash: Buffer.from(passwordHash),
					// the prisma one is just uppercase, this should be fine
					role: [newRole],
					config: {
						create: {
							userId: existingUserId,
							...configParsed
						}
					}
				}
			});

			await createSession(existingUserId, [newRole], cookies);
		}

		redirect(303, '/');
	}
} satisfies Actions;
