import { GetPointsStore } from '$houdini';

export const load = async (event) => {
	const showPoints = await event.parent().then((p) => p.showPoints);
	if (!showPoints) {
		return { points: Promise.resolve([] as const) };
	}

	const index = new GetPointsStore();
	const data = index.fetch({
		event
	});

	return {
		points: data.then((c) => c.data?.points)
	};
};
