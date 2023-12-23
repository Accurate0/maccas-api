<script lang="ts">
	import '@material/web/tabs/tabs';
	import '@material/web/icon/icon';
	import '@material/web/tabs/primary-tab';
	import '@material/web/progress/linear-progress';
	import '@material/web/list/list';
	import '@material/web/list/list-item';
	import '@material/web/iconbutton/icon-button';
	import '@material/web/divider/divider';
	import '@material/web/textfield/outlined-text-field';
	import '@material/web/button/filled-button';

	import type { PageData } from './$houdini';
	import { writable, type Writable } from 'svelte/store';
	import Code from '../components/code.svelte';

	export let data: PageData;

	let openDeals: Writable<{ [key: string]: string[] }> = writable({});
	const removeDeal = (uuid: string, id: string) => {
		openDeals.set({ ...$openDeals, [uuid]: $openDeals[uuid].filter((c) => c !== id) });
	};

	$: ({ Index } = data);
</script>

<svelte:head>
	<title>Maccas</title>
</svelte:head>

{#if $Index.data}
	<div class="grid grid-flow-row gap-4 container mx-auto p-4">
		{#each $Index.data.offers as offer}
			<div class="border rounded p-4 grid grid-flow-row gap-4">
				<button
					class="grid grid-flow-col w-full gap-8 justify-between text-left"
					on:click={() => {
						const id = crypto.randomUUID();
						openDeals.update((c) => ({
							...c,
							[offer.id]: [...(c[offer.id] ?? []), id]
						}));
					}}
				>
					<div class="grid grid-flow-row">
						<h2 class="font-medium">{offer.shortName}</h2>
						<p><small class="text-xs">{offer.count} available</small></p>
					</div>
					<img src={offer.imageUrl} alt={offer.shortName} class="h-24" />
				</button>
				<Code deals={$openDeals[offer.id]} uuid={offer.id} remove={removeDeal} />
			</div>
		{/each}
	</div>
{/if}
