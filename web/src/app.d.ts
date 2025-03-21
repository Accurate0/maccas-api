// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
import { Session as DbSession } from '@prisma/client';
import { Client } from '@openfeature/server-sdk';
declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			session: DbSession;
			featureFlagClient: Client;
		}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
		interface Session extends DbSession {}
	}
}

export {};
