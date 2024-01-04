import { AddOfferStore, GetOfferCodeStore, RemoveOfferStore } from '$houdini';
import { json } from '@sveltejs/kit';

export type AddOfferResponse = {
	code: string;
	id: string;
};

export async function GET(event) {
	const {
		params: { offerId }
	} = event;

	const store = new GetOfferCodeStore();
	const { data } = await store.fetch({ event, variables: { id: offerId } });

	if (!data?.offerById.code) {
		return new Response(null, { status: 500 });
	}

	return json({ code: data.offerById.code });
}

export async function POST(event) {
	const {
		params: { offerId }
	} = event;

	if (Number.isNaN(Number(offerId))) {
		return new Response('Invalid body', { status: 400 });
	}

	const store = new AddOfferStore();
	const { data } = await store.mutate(
		{
			offerPropositionId: Number(offerId)
		},
		{ event }
	);

	if (!data?.addOffer.code) {
		return new Response(null, { status: 500 });
	}

	return json({ code: data.addOffer.code, id: data.addOffer.id });
}

export async function DELETE(event) {
	const {
		params: { offerId }
	} = event;

	const store = new RemoveOfferStore();
	const { data } = await store.mutate(
		{
			id: offerId
		},
		{ event }
	);

	if (!data?.removeOffer) {
		return new Response(null, { status: 500 });
	}

	return new Response(null, { status: 204 });
}
