import { prisma } from '$lib/server/prisma';
import type { Actions } from './$types';
import bcrypt from 'bcrypt';
import { Role } from '@prisma/client';
import { fail } from '@sveltejs/kit';
import { message, setError, superValidate } from 'sveltekit-superforms/server';
import { schema } from './schema';

export type RegisterState = {
	error: string | null;
};

export const load = async () => {
	const form = await superValidate(schema);
	return { form };
};

export const actions = {
	default: async ({ request }) => {
		const form = await superValidate(request, schema);
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

		await prisma.user.create({
			data: {
				username: username.toLowerCase(),
				passwordHash: Buffer.from(passwordHash),
				role: Role.USER,
				active: false
			}
		});

		return message(form, 'Account created but not active');
	}
} satisfies Actions;
