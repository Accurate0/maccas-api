import { HoudiniClient } from '$houdini';
import { env } from '$env/dynamic/private';

export default new HoudiniClient({
	url: `${env.API_BASE_URL}/graphql`,
	fetchParams({ session }) {
		return {
			headers: {
				Authorization: `Bearer ${session?.accessToken}`
			}
		};
	}
});
