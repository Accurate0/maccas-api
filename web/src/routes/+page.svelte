<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs';
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';

	export let data;
</script>

<svelte:head>
	<title>Maccas</title>
</svelte:head>

<Tabs.Root value="offers" class="w-[100%]">
	<Tabs.List class="m-4 mb-0">
		<Tabs.Trigger value="offers">Offers</Tabs.Trigger>
		{#if data.pointsList}
			<Tabs.Trigger value="points">Points</Tabs.Trigger>
		{/if}
		<Tabs.Trigger value="location">Location</Tabs.Trigger>
	</Tabs.List>
	<Tabs.Content value="offers" class="m-4">
		{#if data.offersList}
			<div class="grid grid-flow-row gap-4">
				{#each data.offersList as { shortName, name, count, imageBasename }}
					<Card.Root>
						<div class="grid grid-flow-col justify-between">
							<Card.Header class="grid justify-between">
								<Card.Title>{shortName}</Card.Title>
								<Card.Description>{name}</Card.Description>
								<Badge class="h-fit w-fit">{count} available</Badge>
							</Card.Header>
							<Card.Header>
								<img src={`api/images/${imageBasename}`} alt={shortName} width={90} height={90} />
							</Card.Header>
						</div>
						<!-- <Card.Footer>
							<Skeleton class="bg-primary/50 h-2 w-full" />
						</Card.Footer> -->
					</Card.Root>
				{/each}
			</div>
		{/if}
	</Tabs.Content>
	<Tabs.Content value="points" class="m-4">
		{#if data.pointsList}
			<div class="grid grid-flow-row gap-4">
				{#each data.pointsList as { currentPoints }}
					<Card.Root>
						<div class="grid grid-flow-col justify-between">
							<Card.Header class="justify-center">
								<Card.Title>{currentPoints}</Card.Title>
							</Card.Header>
							<Card.Header>
								{#if currentPoints >= 2500}
									<Badge class="h-fit w-fit">
										<span class="material-symbols-outlined">local_cafe</span>
									</Badge>
								{/if}
								{#if currentPoints >= 5000}
									<Badge class="h-fit w-fit">
										<span class="material-symbols-outlined">icecream</span>
									</Badge>
								{/if}
								{#if currentPoints >= 7500}
									<Badge class="h-fit w-fit">
										<span class="material-symbols-outlined">lunch_dining</span>
									</Badge>
								{/if}
							</Card.Header>
						</div>
					</Card.Root>
				{/each}
			</div>
		{/if}
	</Tabs.Content>
	<Tabs.Content value="location" class="m-4">
		<Card.Root>
			<Card.Header>
				<Card.Title>Search</Card.Title>
			</Card.Header>
			<Card.Content>
				<div class="grid w-full items-center gap-4">
					<Input id="search" type="username" placeholder="e.g. Armadale" name="search" />
				</div>
			</Card.Content>
			<Card.Footer>
				<Button class="w-full">Find</Button>
			</Card.Footer>
		</Card.Root>
	</Tabs.Content>
</Tabs.Root>
