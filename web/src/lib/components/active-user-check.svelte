<script lang="ts">
	import { afterNavigate, goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';

	export let isUserActive: boolean;

	const forceLocationTab = () => {
		if (!isUserActive && $page.url.pathname !== '/inactive') {
			toast.error('This account is not yet activated');
			goto('/inactive');
		}
	};

	onMount(() => {
		forceLocationTab();
	});

	afterNavigate(() => {
		forceLocationTab();
	});
</script>
