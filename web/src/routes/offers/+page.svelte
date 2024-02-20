<script lang="ts">
	import type { PageData } from './$types';

	export let data: PageData;

	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { writable, type Writable } from 'svelte/store';
	import { slide } from 'svelte/transition';
	import { isFuture, isPast, parseJSON, formatDistanceToNow } from 'date-fns';
	import * as Select from '$lib/components/ui/select';
	import type { Selected } from 'bits-ui';
	import { ChevronDown, ChevronUp } from 'radix-icons-svelte';
	import { Button } from '$lib/components/ui/button';
	import DealCode from '$lib/components/deal-code.svelte';

	let filters = writable<Array<string> | undefined>();
	let state: Writable<Record<number, Array<{ id: string }>>> = writable({});
	let sortByAsc = true;

	const addOffer = (offerId: number, id: string) => {
		// FIXME: :)
		if (!$state[offerId]) {
			$state[offerId] = [];
		}

		$state[offerId].push({ id });

		$state = $state;
	};

	const defaultSelected: Selected<string>[] = [];

	const modifyFilter = (selected: Selected<string>[] | undefined) => {
		if (selected?.length === 0) {
			filters.set(undefined);
		} else {
			filters.set(selected?.map((s) => s.value));
		}
	};

	const checkIfFilterMatch = (
		offerCategories: Array<string>,
		filters: Array<string> | undefined
	) => {
		if (filters === undefined) {
			return true;
		}

		if (filters?.includes('Other') && offerCategories.length === 0) {
			return true;
		}

		if (offerCategories.some((c) => filters?.includes(c))) {
			return true;
		}

		return false;
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
	{#await Promise.all([data.offers, data.categories])}
		<div class="flex flex-row gap-2">
			<Skeleton class="h-[48px] w-full rounded-sm" />
			<Skeleton class="h-12 w-[50px] rounded-sm" />
		</div>
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
	{:then [offersList, categories]}
		<div class="flex flex-row gap-2">
			<Select.Root
				selected={defaultSelected}
				multiple
				closeOnOutsideClick
				closeOnEscape
				onSelectedChange={(e) => modifyFilter(e)}
			>
				<Select.Trigger class="grid h-12 grid-flow-col">
					<Select.Value placeholder="Filter by type" />
				</Select.Trigger>
				<Select.Content>
					{#each (categories ?? []).sort((a, b) => a.localeCompare(b)) as category}
						<Select.Item value={category}>{category}</Select.Item>
					{/each}
					<Select.Item value="Other">Other</Select.Item>
				</Select.Content>
			</Select.Root>
			<div>
				<Button
					on:click={() => (sortByAsc = !sortByAsc)}
					variant="outline"
					size="icon"
					class="h-12 w-12"
				>
					{#if sortByAsc}
						<ChevronDown />
					{:else}
						<ChevronUp />
					{/if}
					<span class="sr-only">Toggle theme</span>
				</Button>
			</div>
		</div>
		{#each (offersList ?? []).sort((a, b) => {
			if (!a.price) {
				return Number.MAX_SAFE_INTEGER;
			}

			if (!b.price) {
				return Number.MIN_SAFE_INTEGER;
			}

			if (sortByAsc) {
				return a.price - b.price;
			} else {
				return b.price - a.price;
			}
		}) as { shortName, count, imageBasename, offerPropositionId, validFrom, validTo, categories }}
			{@const isValid = isOfferValid({ validFrom, validTo })}
			{@const validInFuture = isFuture(parseJSON(validFrom))}
			{@const matchesFilter = checkIfFilterMatch(categories, $filters)}
			{#if matchesFilter}
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
								{#if validInFuture}
									Active
									{formatDistanceToNow(new Date(validFrom + 'Z'), {
										addSuffix: true
									})}
								{:else}
									{isFuture(parseJSON(validTo)) ? 'Expires' : 'Expired'}
									{formatDistanceToNow(new Date(validTo + 'Z'), {
										addSuffix: true
									})}
								{/if}
							</Card.Description>
							<div class="self-end">
								<Badge class="h-fit w-fit">{count} available</Badge>
								{#each categories as category}
									<Badge class="h-fit w-fit">{category}</Badge>
								{/each}
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
									{#each $state[offerPropositionId] as { id } (id)}
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
			{/if}
		{/each}
	{/await}
</div>
