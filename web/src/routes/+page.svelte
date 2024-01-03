<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs';
	import * as Card from '$lib/components/ui/card';
	import { Toaster } from 'svelte-sonner';
	import type { PageData } from './$types';
	import Offers from '$lib/components/tabs/offers.svelte';
	import Points from '$lib/components/tabs/points.svelte';
	import Locations from '$lib/components/tabs/locations.svelte';

	export let data: PageData;
	let storeName = data.config?.storeName;
</script>

<svelte:head>
	<title>Maccas</title>
</svelte:head>

<Toaster richColors />

<Tabs.Root value="offers" class="w-[100%]">
	<Tabs.List class="m-4 mb-0">
		<Tabs.Trigger value="offers">Offers</Tabs.Trigger>
		{#await data.pointsList then pointsList}
			{#if pointsList}
				<Tabs.Trigger value="points">Points</Tabs.Trigger>
			{/if}
		{/await}
		<Tabs.Trigger value="location">Location</Tabs.Trigger>
	</Tabs.List>

	<div class="m-4 grid grid-flow-row gap-4">
		<Card.Root>
			<div class="grid grid-flow-col justify-between">
				<Card.Header class="grid justify-between">
					<Card.Title>Store</Card.Title>
					<Card.Description>{storeName}</Card.Description>
				</Card.Header>
			</div>
		</Card.Root>
	</div>

	<Tabs.Content value="offers" class="m-4">
		<Offers offersList={data.offersList} />
	</Tabs.Content>
	<Tabs.Content value="points" class="m-4">
		<Points pointsList={data.pointsList} />
	</Tabs.Content>
	<Tabs.Content value="location" class="m-4">
		<Locations config={data.config} bind:storeName />
	</Tabs.Content>
</Tabs.Root>
