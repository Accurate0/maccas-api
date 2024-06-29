import { prisma } from '$lib/server/prisma';
import { validateAdminUser } from '$lib/server/validateAdminUser';
import { NotificationType, Priority } from '@prisma/client';

export async function POST(event) {
	const {
		params: { userId },
		locals
	} = event;

	const validationResult = await validateAdminUser(locals.session.userId);
	if (validationResult != null) {
		return validationResult;
	}

	const user = await prisma.user.update({
		where: { id: userId },
		data: {
			active: true
		}
	});

	await prisma.notification.create({
		data: {
			content: `User set to active ${user.username}`,
			context: { username: user.username },
			read: false,
			priority: Priority.NORMAL,
			type: NotificationType.USER_ACTIVATED
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

	const user = await prisma.user.update({
		where: { id: userId },
		data: {
			active: false
		}
	});

	await prisma.notification.create({
		data: {
			content: `User set to inactive ${user.username}`,
			context: { username: user.username },
			read: false,
			priority: Priority.NORMAL,
			type: NotificationType.USER_DEACTIVATED
		}
	});

	return new Response(null, { status: 204 });
}
