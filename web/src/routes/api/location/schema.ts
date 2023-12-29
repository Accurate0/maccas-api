import { z } from 'zod';

export const schema = z.object({
	storeId: z.string().min(3)
});

export type UpdateLocationBody = z.infer<typeof schema>;
