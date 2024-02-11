import type { Config } from '@prisma/client';
import { writable } from 'svelte/store';

export const configStore = writable<Omit<Config, 'id' | 'userId'>>(undefined);
