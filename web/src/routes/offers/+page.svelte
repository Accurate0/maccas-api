<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import type { PageData } from './$houdini';
	import { Skeleton } from '$lib/components/ui/skeleton';

	export let data: PageData;

	$: ({ GetOffers } = data);
</script>

<svelte:head>
	<title>Offers | Maccas</title>
</svelte:head>

{#if $GetOffers.data}
	<div class="grid grid-flow-row gap-4">
		{#each $GetOffers.data.offers as { shortName, name, count, imageUrl }}
			<Card.Root>
				<div class="grid grid-flow-col justify-between">
					<Card.Header class="grid justify-between">
						<Card.Title>{shortName}</Card.Title>
						<Card.Description>{name}</Card.Description>
						<Badge class="h-fit w-fit">{count} available</Badge>
					</Card.Header>
					<Card.Header>
						<img src={imageUrl} alt={shortName} width={100} height={100} />
					</Card.Header>
				</div>
				<!-- <Card.Footer>
					<Skeleton class="bg-primary/50 h-2 w-full" />
				</Card.Footer> -->
			</Card.Root>
		{/each}
	</div>
{/if}
