import { prisma } from '$lib/server/prisma';
import { Role } from '@prisma/client';

export async function validateAdminUser(userId: string) {
	const user = await prisma.user.findUniqueOrThrow({ where: { id: userId } });
	if (user.role !== Role.ADMIN) {
		return new Response(null, { status: 403 });
	}

	return null;
}
