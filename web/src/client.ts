import { HoudiniClient } from '$houdini';
import { env } from '$env/dynamic/private';

export default new HoudiniClient({
	url: `${env.API_BASE_URL}/graphql`

	// uncomment this to configure the network call (for things like authentication)
	// for more information, please visit here: https://www.houdinigraphql.com/guides/authentication
	// fetchParams({ session }) {
	//     return {
	//         headers: {
	//             Authentication: `Bearer ${session.token}`,
	//         }
	//     }
	// }
});
