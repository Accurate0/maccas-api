import { prisma } from '$lib/server/prisma';
import { Role } from '@prisma/client';

async function validateAdminUser(userId: string) {
	const user = await prisma.user.findUniqueOrThrow({ where: { id: userId } });
	if (user.role !== Role.ADMIN) {
		return new Response(null, { status: 403 });
	}

	return null;
}

export async function POST(event) {
	const {
		params: { userId },
		locals
	} = event;

	const validationResult = await validateAdminUser(locals.session.userId);
	if (validationResult != null) {
		return validationResult;
	}

	await prisma.user.update({
		where: { id: userId },
		data: {
			active: true
		}
	});

	return new Response(null, { status: 204 });
}

export async function DELETE(event) {
	const {
		params: { userId },
		locals
	} = event;

	const validationResult = await validateAdminUser(locals.session.userId);
	if (validationResult != null) {
		return validationResult;
	}

	await prisma.user.update({
		where: { id: userId },
		data: {
			active: false
		}
	});

	return new Response(null, { status: 204 });
}
