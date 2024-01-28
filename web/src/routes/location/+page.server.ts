export const load = async (event) => {
	return {
		storeName: await event.parent().then((p) => p.storeName)
	};
};
