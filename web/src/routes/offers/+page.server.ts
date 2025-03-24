import { GetOffersStore } from '$houdini';

export const load = async (event) => {
	const index = new GetOffersStore();
	const data = index.fetch({
		event
	});

	const featureFlagClient = event.locals.featureFlagClient;
	const showNewBadge = await featureFlagClient.getBooleanValue('maccas-web-show-new-badge', false);
	const isRecommendationsEnabled = await featureFlagClient.getBooleanValue(
		'maccas-web-add-recommendations',
		false
	);

	return {
		offers: data.then((c) => c.data?.offers),
		categories: data.then((c) => c.data?.categories),
		recommendationIds: data.then((c) => c.data?.recommendations.map((r) => r.offerPropositionId)),
		isRecommendationsEnabled,
		showNewBadge
	};
};
