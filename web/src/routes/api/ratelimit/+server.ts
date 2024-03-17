import { RateLimiter } from '$lib/server/ratelimiter.js';
import { validateAdminUser } from '$lib/server/validateAdminUser';

export async function DELETE(event) {
	const { locals } = event;

	const validationResult = await validateAdminUser(locals.session.userId);
	if (validationResult != null) {
		return validationResult;
	}

	await RateLimiter.clear();

	return new Response(null, { status: 204 });
}
