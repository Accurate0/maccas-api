<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { writable, type Writable } from 'svelte/store';
	import DealCode from '../deal-code.svelte';
	import { slide } from 'svelte/transition';
	import { isFuture, isPast, parseJSON, formatDistanceToNow } from 'date-fns';
	import { Check } from 'radix-icons-svelte';
	import Separator from '../ui/separator/separator.svelte';

	type OfferList = import('$houdini').GetOffers$result['offers'] | undefined;
	export let categories: Promise<Array<string> | undefined>;
	export let offersList: Promise<OfferList>;
	let state: Writable<Record<number, Array<{ id: string }>>> = writable({});

	const categoriseOffers = (list: OfferList, categories: Array<string> | undefined) => {
		const categorised = categories
			?.sort((a, b) => a.localeCompare(b))
			?.reduce(
				(prev, curr) => {
					return {
						...prev,
						[curr]: [
							...(prev[curr] ?? []),
							...(list?.filter((o) => o.categories.includes(curr)) ?? [])
						]
					};
				},
				{} as Record<string, Exclude<OfferList, 'undefined'>>
			);

		if (categorised) {
			categorised['Other'] = list?.filter((o) => o.categories.length === 0) ?? [];
		}

		return categorised;
	};

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
					<Card.Header class="grid w-full">
						<Skeleton class="h-[22px] w-[50%] rounded-xl" />
						<Skeleton class="h-[24px] w-[30%] self-end rounded-xl" />
					</Card.Header>
					<Card.Header>
						<Skeleton class="h-[90px] w-[90px] rounded-xl" />
					</Card.Header>
				</div>
			</Card.Root>
		{/each}
	{:then offersList}
		{#await categories then categories}
			{@const offers = categoriseOffers(offersList, categories)}
			{#each Object.entries(offers ?? {}) as [category, offerList]}
				{#if (offerList?.length ?? 0) > 0}
					<h2 class="text-lg font-bold tracking-tight">{category}</h2>
					<Separator />
					{#each offerList ?? [] as { shortName, count, imageBasename, offerPropositionId, validFrom, validTo, categories }}
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
									<Card.Description>
										{isFuture(parseJSON(validTo)) ? 'Expires' : 'Expired'}
										{formatDistanceToNow(new Date(validTo + 'Z'), {
											addSuffix: true
										})}
									</Card.Description>
									<div class="self-end">
										<Badge class="h-fit w-fit">{count} available</Badge>
									</div>
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
								<div in:slide={{ duration: 600 }} out:slide={{ duration: 600 }}>
									<Card.Footer>
										<div class="grid h-full w-full grid-flow-row gap-2">
											{#each $state[offerPropositionId] as { id }}
												<span in:slide={{ duration: 800 }} out:slide={{ duration: 800 }}>
													<DealCode
														offerId={offerPropositionId}
														{id}
														removeSelf={() => removeOffer(offerPropositionId, id)}
													/>
												</span>
											{/each}
										</div>
									</Card.Footer>
								</div>
							{/if}
						</Card.Root>
					{/each}
				{/if}
			{/each}
		{/await}
	{/await}
</div>
