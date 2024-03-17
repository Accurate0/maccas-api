import { env } from '$env/dynamic/private';
import { RetryAfterRateLimiter as RateLimiterObject } from 'sveltekit-rate-limiter/server';

export const RateLimiter = new RateLimiterObject({
	IP: [20, 'h'],
	IPUA: [10, 'm'],
	cookie: {
		name: 'session-limit',
		secret: env.AUTH_SECRET,
		rate: [5, 'm'],
		preflight: true
	}
});
