import { GetAccountCodeStore } from '$houdini';
import { prisma } from '$lib/server/prisma.js';
import { Role } from '@prisma/client';
import { json } from '@sveltejs/kit';

export type AddOfferResponse = {
	code: string;
	id: string;
};

export async function GET(event) {
	const {
		params: { accountId },
		locals
	} = event;

	const config = await prisma.config.findUnique({
		where: { userId: locals.session.userId },
		include: { User: true }
	});

	if (!config || !config.storeId) {
		return new Response(null, { status: 400 });
	}

	if (config.User?.role !== Role.PRIVILEGED && config.User?.role !== Role.ADMIN) {
		return new Response(null, { status: 403 });
	}

	const store = new GetAccountCodeStore();
	const { data } = await store.fetch({ event, variables: { accountId, storeId: config.storeId } });

	return json({ code: data?.pointsByAccountId.code });
}
