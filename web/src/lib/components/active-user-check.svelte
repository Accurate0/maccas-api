<script lang="ts">
	import { afterNavigate, goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';

	interface Props {
		isUserActive: boolean;
	}

	let { isUserActive }: Props = $props();

	const forceInactivePage = () => {
		if (!isUserActive && $page.url.pathname !== '/inactive') {
			toast.error('This account is not yet activated');
			goto('/inactive');
		}
	};

	onMount(forceInactivePage);
	afterNavigate(forceInactivePage);
</script>
