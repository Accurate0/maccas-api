<script lang="ts">
	import type { PageData } from './$types';

	import * as Card from '$lib/components/ui/card';
	import { toast } from 'svelte-sonner';
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
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let filters = writable<Array<string> | undefined>();
	let offerState: Writable<Record<string, Array<{ id: string }>>> = writable({});
	let sortByAsc = $state(true);
	const userConfig = $page.data.config;

	const addOffer = (offerId: string, id: string) => {
		// FIXME: :)
		if (!$offerState[offerId]) {
			$offerState[offerId] = [];
		}

		$offerState[offerId].push({ id });

		$offerState = $offerState;
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

	const removeOffer = async (offerId: string, id: string) => {
		offerState.update((s) => ({ ...s, [offerId]: s[offerId].filter((o) => o.id !== id) }));
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
			<Skeleton class="h-12 min-w-12 rounded-sm" />
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
					<span class="sr-only">Toggle price</span>
				</Button>
			</div>
		</div>
		{#each (offersList ?? []).sort((a, b) => {
			if (!a.price) {
				return 0;
			}

			if (!b.price) {
				return 0;
			}

			if (sortByAsc) {
				return a.price - b.price;
			} else {
				return b.price - a.price;
			}
		}) as { shortName, count, imageUrl, offerPropositionId, validFrom, validTo, categories } (shortName)}
			{@const isValid = isOfferValid({ validFrom, validTo })}
			{@const validInFuture = isFuture(parseJSON(validFrom))}
			{@const matchesFilter = checkIfFilterMatch(categories, $filters)}
			{#if matchesFilter}
				<Card.Root
					on:click={async () => {
						if (!userConfig) {
							toast.error('A store location must be set');
							await goto('/location');
						}

						if (($offerState[shortName]?.length ?? 0) < count) {
							addOffer(shortName, crypto.randomUUID());
						}
					}}
					class={isValid ? undefined : 'opacity-30'}
				>
					<div class="grid grid-flow-col justify-between">
						<Card.Header class="grid justify-between pr-0">
							<Card.Title class="max-w-[12rem] overflow-hidden text-ellipsis"
								>{shortName}</Card.Title
							>
							<Card.Description class="mt-0">
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
							<div class="flex flex-row self-end">
								<Badge class="mr-1 h-fit w-fit"
									>{count}{categories.length < 2 ? ' available' : ''}</Badge
								>
								{#each categories as category}
									<Badge class="mr-1 h-fit w-fit">{category}</Badge>
								{/each}
							</div>
						</Card.Header>
						<Card.Header class="pl-0">
							<img class="rounded-xl" src={imageUrl} alt={shortName} width={90} height={90} />
						</Card.Header>
					</div>
					{#if $offerState[shortName] && $offerState[shortName].length > 0}
						<div in:slide={{ duration: 600 }} out:slide={{ duration: 600 }}>
							<Card.Footer>
								<div class="grid h-full w-full grid-flow-row gap-2">
									{#each $offerState[shortName] as { id } (id)}
										<span in:slide={{ duration: 800 }} out:slide={{ duration: 800 }}>
											<DealCode
												offerId={offerPropositionId}
												{id}
												removeSelf={() => removeOffer(shortName, id)}
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
