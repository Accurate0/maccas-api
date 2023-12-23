import { PUBLIC_API_BASE } from '$env/static/public';
import { HoudiniClient } from '$houdini';

export default new HoudiniClient({
	url: `${PUBLIC_API_BASE}/graphql`,
	fetchParams({ session }) {
		return {
			headers: {
				Authorization: `Bearer ${session?.token}`
			}
		};
	}
});
