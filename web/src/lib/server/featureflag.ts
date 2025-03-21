import { NOOP_PROVIDER, OpenFeature } from '@openfeature/server-sdk';
import { FliptProvider } from '@openfeature/flipt-provider';
import { ClientTokenAuthentication } from '@flipt-io/flipt';
import { env } from '$env/dynamic/private';

const url = env.FLIPT_URL ?? '';
const token = env.FLIPT_TOKEN;

if (token) {
	const provider = new FliptProvider('default', {
		url,
		authenticationStrategy: new ClientTokenAuthentication(token)
	});
	await OpenFeature.setProviderAndWait(provider);
} else {
	await OpenFeature.setProviderAndWait(NOOP_PROVIDER);
}

export { OpenFeature as FeatureFlagClientFactory };
