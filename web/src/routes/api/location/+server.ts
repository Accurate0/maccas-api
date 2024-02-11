import { LocationByCoordinatesStore, LocationByTextStore, StoreByIdStore } from '$houdini';
import { json } from '@sveltejs/kit';
import { schema } from './schema';
import { prisma } from '$lib/prisma';

export async function GET(event) {
	const query = event.url.searchParams.get('query');

	if (query) {
		const store = new LocationByTextStore();
		const { data } = await store.fetch({
			event,
			variables: { query }
		});

		return json(data?.location.text ?? []);
	}

	const latitude = Number(event.url.searchParams.get('latitude'));
	const longitude = Number(event.url.searchParams.get('longitude'));

	if (!isNaN(latitude) && !isNaN(longitude)) {
		const store = new LocationByCoordinatesStore();
		const { data } = await store.fetch({ event, variables: { latitude, longitude } });

		return json(data?.location.coordinate ?? []);
	}

	return new Response(null, { status: 400 });
}

export async function POST(event) {
	const body = await event.request.json();
	const validatedBody = await schema.safeParseAsync(body);
	if (!validatedBody.success) {
		return new Response('Invalid body', { status: 400 });
	}

	const store = new StoreByIdStore();
	const { data } = await store.fetch({ event, variables: { storeId: validatedBody.data.storeId } });
	const user = await prisma.user.findUniqueOrThrow({ where: { id: event.locals.session.userId } });
	if (user.configId) {
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
	} else {
		const createdConfig = await prisma.config.create({
			data: {
				userId: user.id,
				storeId: validatedBody.data.storeId,
				storeName: data?.location.storeId.name
			}
		});

		await prisma.user.update({
			where: { id: event.locals.session.userId },
			data: {
				configId: createdConfig.id
			}
		});
	}

	return new Response(null, { status: 204 });
}
