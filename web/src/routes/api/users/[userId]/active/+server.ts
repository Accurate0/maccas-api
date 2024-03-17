import { prisma } from '$lib/server/prisma';
import { validateAdminUser } from '$lib/server/validateAdminUser';

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
