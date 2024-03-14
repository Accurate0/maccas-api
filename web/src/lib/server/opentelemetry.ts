import opentelemetry from '@opentelemetry/sdk-node';
import { Resource } from '@opentelemetry/resources';
import {
	SEMRESATTRS_DEPLOYMENT_ENVIRONMENT,
	SEMRESATTRS_SERVICE_NAME
} from '@opentelemetry/semantic-conventions';
import { NODE_ENV } from '$env/static/private';
import { env } from '$env/dynamic/private';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';

const otelSdk = new opentelemetry.NodeSDK({
	resource: new Resource({
		[SEMRESATTRS_SERVICE_NAME]: 'web',
		[SEMRESATTRS_DEPLOYMENT_ENVIRONMENT]: NODE_ENV,
		['highlight.project_id']: env.OTEL_PROJECT_ID
	}),
	traceExporter: new OTLPTraceExporter({
		url: 'https://otel.highlight.io:4318/v1/traces'
	})
});

otelSdk.start();
