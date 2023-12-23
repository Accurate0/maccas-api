import { setSession } from '$houdini';

export const handle = async ({ event, resolve }) => {
	const token = event.cookies.get('token') ?? null;

	setSession(event, { token });

	return await resolve(event);
};
