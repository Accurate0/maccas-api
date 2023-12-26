import { env } from '$env/dynamic/public';
import { HoudiniClient } from '$houdini';

export default new HoudiniClient({
	url: `${env.PUBLIC_API_BASE}/graphql`,
	fetchParams({ session }) {
		return {
			headers: {
				Authorization: `Bearer ${session?.token}`
			}
		};
	}
});
