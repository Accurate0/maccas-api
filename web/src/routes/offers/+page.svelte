<script lang="ts">
	import Offers from '$lib/components/tabs/offers.svelte';
	import type { PageData } from './$types';

	export let data: PageData;
</script>

<Offers offersList={data.offers} categories={data.categories} />
