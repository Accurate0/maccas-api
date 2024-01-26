<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { writable, type Writable } from 'svelte/store';
	import DealCode from '../deal-code.svelte';
	import { slide } from 'svelte/transition';
	import { isFuture, isPast, parseJSON } from 'date-fns';

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
	};

	const isOfferValid = (offer: { validTo: string; validFrom: string }) => {
		const from = parseJSON(offer.validFrom);
		const to = parseJSON(offer.validTo);

		return isPast(from) && isFuture(to);
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
		{#each offersList ?? [] as { shortName, name, count, imageBasename, offerPropositionId, validFrom, validTo }}
			{@const isValid = isOfferValid({ validFrom, validTo })}
			<Card.Root
				on:click={() => {
					if (($state[offerPropositionId]?.length ?? 0) < count) {
						addOffer(offerPropositionId, crypto.randomUUID());
					}
				}}
				class={isValid ? undefined : 'opacity-30'}
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
				{#if $state[offerPropositionId] && $state[offerPropositionId].length > 0}
					<Card.Footer>
						<div in:slide out:slide class="grid h-full w-full grid-flow-row gap-2">
							{#each $state[offerPropositionId] as { id }}
								<span in:slide out:slide>
									<DealCode
										offerId={offerPropositionId}
										{id}
										removeSelf={() => removeOffer(offerPropositionId, id)}
									/>
								</span>
							{/each}
						</div>
					</Card.Footer>
				{/if}
			</Card.Root>
		{/each}
	{/await}
</div>
