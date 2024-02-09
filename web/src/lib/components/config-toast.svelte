<script lang="ts">
	import { afterNavigate, goto } from '$app/navigation';
	import type { Config } from '@prisma/client';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/stores';

	export let config: Exclude<Config, 'id' | 'userId'> | null;

	const configMissing = config === null || config.storeId === null || config.storeName === null;
	const forceLocationTab = () => {
		if (configMissing && $page.url.pathname !== '/location') {
			toast.error('No location selected');
			goto('/location');
		}
	};

	onMount(() => {
		forceLocationTab();
	});

	afterNavigate(() => {
		forceLocationTab();
	});
</script>
