<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs';
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import type { LocationByText$result } from '$houdini';
	import { Toaster, toast } from 'svelte-sonner';
	import type { UpdateLocationBody } from './api/location/schema';
	import { writable } from 'svelte/store';
	import type { PageData } from './$types';

	export let data: PageData;
	const config = writable<PageData['config']>(data.config);
	let disabled = false;
	let optionsDisabled = false;
	let query: string = '';
	let options: LocationByText$result['location']['text'] = [];

	const searchLocations = async () => {
		if (query) {
			disabled = true;
			const response = await fetch(`/api/location?query=${query}`, { method: 'GET' });
			options = (await response.json()) as LocationByText$result['location']['text'];
			disabled = false;
		} else {
			toast.error('Gotta type something');
		}
	};

	const setLocation = async (storeId: string, storeName: string) => {
		optionsDisabled = true;
		const body: UpdateLocationBody = { storeId };
		const response = await fetch(`/api/location`, { method: 'POST', body: JSON.stringify(body) });
		// ID is not used here, and the types are playing funny
		config.set({ storeName, storeId, id: '' });

		if (response.ok) {
			toast('Location updated');
		} else {
			toast.error('Something went wrong');
		}
		optionsDisabled = false;
	};
</script>

<svelte:head>
	<title>Maccas</title>
</svelte:head>

<Toaster />

<Tabs.Root value="location" class="w-[100%]">
	<Tabs.List class="m-4 mb-0">
		<Tabs.Trigger value="offers">Offers</Tabs.Trigger>
		{#if data.pointsList}
			<Tabs.Trigger value="points">Points</Tabs.Trigger>
		{/if}
		<Tabs.Trigger value="location">Location</Tabs.Trigger>
	</Tabs.List>

	<div class="m-4 grid grid-flow-row gap-4">
		<Card.Root>
			<div class="grid grid-flow-col justify-between">
				<Card.Header class="grid justify-between">
					<Card.Title>Store</Card.Title>
					<Card.Description>{$config?.storeName}</Card.Description>
				</Card.Header>
			</div>
			<!-- <Card.Footer>
			<Skeleton class="bg-primary/50 h-2 w-full" />
		</Card.Footer> -->
		</Card.Root>
	</div>

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
					<Input
						id="query"
						type="username"
						placeholder="e.g. Armadale"
						name="query"
						bind:value={query}
						required
					/>
				</div>
				{#if options.length > 0}
					<div class="grid w-full items-center gap-4 pt-4">
						{#each options as location}
							<Button
								variant="outline"
								disabled={optionsDisabled}
								aria-disabled={optionsDisabled}
								on:click={() => setLocation(location.storeNumber, location.name)}
							>
								{location.name}
							</Button>
						{/each}
					</div>
				{/if}
			</Card.Content>
			<Card.Footer>
				<Button class="w-full" type="submit" on:click={searchLocations} {disabled}>Find</Button>
			</Card.Footer>
		</Card.Root>
	</Tabs.Content>
</Tabs.Root>
