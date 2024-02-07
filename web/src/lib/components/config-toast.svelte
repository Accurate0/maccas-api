<script lang="ts">
	import { afterNavigate, goto } from '$app/navigation';
	import type { Config } from '@prisma/client';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { page } from '$app/stores';

	export let config: Config | null;

	const forceLocationTab = () => {
		if (!config && $page.url.pathname !== '/location') {
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
