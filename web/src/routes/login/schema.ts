import { z } from 'zod';

export const schema = z.object({
	username: z.string().min(1, { message: 'Invalid username' }),
	password: z.string().min(1, { message: 'Invalid password' })
});
