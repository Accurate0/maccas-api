<script lang="ts">
	import Locations from '$lib/components/tabs/locations.svelte';
	import type { PageData } from './$types';
	export let data: PageData;

	let storeName = data.storeName;
</script>

<Locations bind:storeName />
