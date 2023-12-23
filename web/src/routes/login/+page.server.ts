import { redirect } from '@sveltejs/kit';
import type { Actions } from './$types';
import { PUBLIC_API_BASE } from '$env/static/public';
import { fail } from '@sveltejs/kit';

export const actions = {
	default: async ({ request, cookies, fetch }) => {
		const formData = await request.formData();

		const result = await fetch(`${PUBLIC_API_BASE}/auth/login`, {
			body: formData,
			method: 'POST'
		});

		if (result.ok) {
			const response = await result.json();
			cookies.set('token', response['token']);
			throw redirect(303, '/');
		} else {
			return fail(400, { error: true });
		}
	}
} satisfies Actions;
