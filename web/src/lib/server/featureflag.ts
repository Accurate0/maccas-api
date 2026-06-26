import { NOOP_PROVIDER, OpenFeature, type EvaluationContext } from '@openfeature/server-sdk';
import { FeatureFlagProvider } from '@accurate0/feature-flag-client/openfeature';
import { env } from '$env/dynamic/private';

const url = env.FEATURE_FLAGS_URL;
if (url) {
	console.log(`creating ff provider: ${url}`);
	await OpenFeature.setProviderAndWait(new FeatureFlagProvider(url, 'maccas-web'));
} else {
	console.warn('falling back to noop provider');
	await OpenFeature.setProviderAndWait(NOOP_PROVIDER);
}

export { OpenFeature as FeatureFlagClientFactory, type EvaluationContext };
