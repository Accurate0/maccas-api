import { GetOffersStore } from '$houdini';
import { featureFlagClient } from '$lib/server/featureflag';

export const load = async (event) => {
	const index = new GetOffersStore();
	const data = index.fetch({
		event
	});

	const showNewBadge = await featureFlagClient.getBooleanValue('maccas-web-show-new-badge', false, {
		user_id: event.locals.session.userId ?? 'unknown'
	});

	return {
		offers: data.then((c) => c.data?.offers),
		categories: data.then((c) => c.data?.categories),
		showNewBadge
	};
};
