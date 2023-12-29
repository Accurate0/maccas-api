import { LocationByTextStore, StoreByIdStore } from '$houdini';
import { json } from '@sveltejs/kit';
import { schema } from './schema';
import { prisma } from '$lib/prisma';

export async function GET(event) {
	const query = event.url.searchParams.get('query')!;
	const store = new LocationByTextStore();
	const { data } = await store.fetch({
		event,
		variables: { query }
	});

	return json(data?.location.text ?? []);
}

export async function POST(event) {
	const body = await event.request.json();
	const validatedBody = await schema.safeParseAsync(body);
	if (!validatedBody.success) {
		return new Response('Invalid body', { status: 400 });
	}

	const store = new StoreByIdStore();
	const { data } = await store.fetch({ event, variables: { storeId: validatedBody.data.storeId } });
	await prisma.user.update({
		where: { id: event.locals.session.userId },
		data: {
			config: {
				update: {
					data: {
						storeId: validatedBody.data.storeId,
						storeName: data?.location.storeId.name
					}
				}
			}
		}
	});

	return new Response(null, { status: 204 });
}
