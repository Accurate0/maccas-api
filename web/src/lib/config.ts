import { writable } from 'svelte/store';

type Config = { storeName: string | null; storeId: string | null } | null | undefined;
export const configStore = writable<Config>(undefined);
