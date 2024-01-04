<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { writable, type Writable } from 'svelte/store';
	import DealCode from '../deal-code.svelte';
	import { slide } from 'svelte/transition';

	export let offersList: Promise<import('$houdini').Index$result['offers'] | undefined>;
	let state: Writable<Record<number, Array<{ id: string }>>> = writable({});

	const addOffer = (offerId: number, id: string) => {
		// FIXME: :)
		if (!$state[offerId]) {
			$state[offerId] = [];
		}

		$state[offerId].push({ id });

		$state = $state;
	};

	const removeOffer = async (offerId: number, id: string) => {
		state.update((s) => ({ ...s, [offerId]: s[offerId].filter((o) => o.id !== id) }));
		console.log($state);
	};
</script>

<div class="grid grid-flow-row gap-4">
	{#await offersList}
		{#each Array(30) as _}
			<Card.Root>
				<div class="flex">
					<Card.Header class="flex flex-grow flex-col">
						<Skeleton class="h-[22px] w-[20%] rounded-xl" />
						<Skeleton class="h-[26px] w-[50%] rounded-xl" />
						<Skeleton class="h-[24px] w-[7%] rounded-xl" />
					</Card.Header>
					<Card.Header>
						<Skeleton class="h-[90px] w-[90px] rounded-xl" />
					</Card.Header>
				</div>
			</Card.Root>
		{/each}
	{:then offersList}
		{#each offersList ?? [] as { shortName, name, count, imageBasename, offerId }}
			<Card.Root
				on:click={() => {
					addOffer(offerId, crypto.randomUUID());
				}}
			>
				<div class="grid grid-flow-col justify-between">
					<Card.Header class="grid justify-between">
						<Card.Title>{shortName}</Card.Title>
						<Card.Description>{name}</Card.Description>
						<Badge class="h-fit w-fit">{count} available</Badge>
					</Card.Header>
					<Card.Header>
						<img
							class="rounded-xl"
							src={`api/images/${imageBasename}`}
							alt={shortName}
							width={90}
							height={90}
						/>
					</Card.Header>
				</div>
				{#if $state[offerId] && $state[offerId].length > 0}
					<Card.Footer>
						<div in:slide out:slide class="grid h-full w-full grid-flow-row gap-2">
							{#each $state[offerId].sort((a, b) => a.id.localeCompare(b.id)) as { id }}
								<span in:slide out:slide>
									<DealCode {offerId} {id} removeSelf={() => removeOffer(offerId, id)} />
								</span>
							{/each}
						</div>
					</Card.Footer>
				{/if}
			</Card.Root>
		{/each}
	{/await}
</div>
