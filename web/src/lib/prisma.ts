import { env } from '$env/dynamic/private';
import { PrismaClient } from '@prisma/client';

const globalForPrisma = globalThis as unknown as {
	prisma: PrismaClient | undefined;
};

export const prisma =
	globalForPrisma.prisma ??
	new PrismaClient({
		log: env.NODE_ENV === 'development' ? ['query', 'error', 'warn'] : ['error']
	});
