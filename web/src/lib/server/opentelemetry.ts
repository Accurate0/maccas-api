import opentelemetry from '@opentelemetry/sdk-node';
import { Resource } from '@opentelemetry/resources';
import {
	SEMRESATTRS_DEPLOYMENT_ENVIRONMENT,
	SEMRESATTRS_SERVICE_NAME
} from '@opentelemetry/semantic-conventions';
import { NODE_ENV } from '$env/static/private';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';
import { BatchSpanProcessor } from '@opentelemetry/sdk-trace-base';

import primsa from '@prisma/instrumentation';
import { env } from '$env/dynamic/private';
const { PrismaInstrumentation } = primsa;

const traceExporter = new OTLPTraceExporter({
	url: 'https://api.axiom.co/v1/traces',
	headers: {
		Authorization: `Bearer ${env.AXIOM_TOKEN}`,
		'X-Axiom-Dataset': env.AXIOM_DATASET
	}
});

const otelSdk = new opentelemetry.NodeSDK({
	resource: new Resource({
		[SEMRESATTRS_SERVICE_NAME]: 'web',
		[SEMRESATTRS_DEPLOYMENT_ENVIRONMENT]: NODE_ENV
	}),
	spanProcessors: [new BatchSpanProcessor(traceExporter)],
	traceExporter,
	instrumentations: [new PrismaInstrumentation()]
});

otelSdk.start();
