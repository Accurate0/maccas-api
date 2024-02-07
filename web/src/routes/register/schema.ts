import { z } from 'zod';

export const schema = z.object({
	username: z.string().min(3),
	password: z.string().min(6)
});

export type FormSchema = typeof schema;
