<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';

	export let offersList: Promise<import('$houdini').Index$result['offers'] | undefined>;
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
		{#each offersList ?? [] as { shortName, name, count, imageBasename }}
			<Card.Root>
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
				<!-- <Card.Footer>
                <Skeleton class="bg-primary/50 h-2 w-full" />
            </Card.Footer> -->
			</Card.Root>
		{/each}
	{/await}
</div>
