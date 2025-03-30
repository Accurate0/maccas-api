import { GetOffersStore } from '$houdini';

export const load = async (event) => {
	const index = new GetOffersStore();
	const data = index.fetch({
		event
	});

	const featureFlagClient = event.locals.featureFlagClient;
	const isRecommendationsEnabled = await featureFlagClient.getBooleanValue(
		'maccas-web-add-recommendations',
		false
	);

	return {
		offers: data.then((c) => c.data?.offers),
		categories: data.then((c) => c.data?.categories),
		recommendations: data.then((c) => c.data?.recommendations.map((r) => r.shortName)),
		isRecommendationsEnabled
	};
};
