import type { Config } from '@prisma/client';
import { writable } from 'svelte/store';

export const configStore = writable<Exclude<Config, 'id' | 'userId'>>(undefined);
