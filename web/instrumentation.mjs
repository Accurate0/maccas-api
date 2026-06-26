import { register } from 'node:module';
import { createAddHookMessageChannel } from 'import-in-the-middle';

const { registerOptions, waitForAllMessagesAcknowledged } = createAddHookMessageChannel();
register('import-in-the-middle/hook.mjs', import.meta.url, registerOptions);

import opentelemetry from '@opentelemetry/sdk-node';
import { Resource } from '@opentelemetry/resources';
import {
	SEMRESATTRS_DEPLOYMENT_ENVIRONMENT,
	SEMRESATTRS_SERVICE_NAME
} from '@opentelemetry/semantic-conventions';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';
import { BatchSpanProcessor } from '@opentelemetry/sdk-trace-base';
import primsa from '@prisma/instrumentation';
import { GrpcInstrumentation } from '@opentelemetry/instrumentation-grpc';
import { HttpInstrumentation } from '@opentelemetry/instrumentation-http';
import { UndiciInstrumentation } from '@opentelemetry/instrumentation-undici';

const { PrismaInstrumentation } = primsa;

const NODE_ENV = process.env.NODE_ENV ?? 'development';

const traceExporter = new OTLPTraceExporter({
	url: process.env.OTEL_TRACING_URL
});

const otelSdk = new opentelemetry.NodeSDK({
	resource: new Resource({
		[SEMRESATTRS_SERVICE_NAME]: 'maccas-web',
		[SEMRESATTRS_DEPLOYMENT_ENVIRONMENT]: NODE_ENV
	}),
	spanProcessors: [new BatchSpanProcessor(traceExporter)],
	traceExporter,
	instrumentations: [
		new PrismaInstrumentation(),
		new GrpcInstrumentation(),
		new HttpInstrumentation({
			ignoreIncomingRequestHook: (req) => {
				const path = (req.url ?? '').split('?')[0];
				return path === '/health';
			}
		}),
		new UndiciInstrumentation()
	]
});

otelSdk.start();

await waitForAllMessagesAcknowledged();
