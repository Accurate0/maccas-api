<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import type { LocationByText$result } from '$houdini';
	import { toast } from 'svelte-sonner';
	import { scale, slide } from 'svelte/transition';
	import { flip } from 'svelte/animate';
	import { configStore } from '$lib/config';
	import { Crosshair1 } from 'radix-icons-svelte';
	import type { UpdateLocationBody } from '../api/location/schema';

	let disabled = $state(false);
	let optionsDisabled = $state(false);
	let query: string = $state('');
	let options: LocationByText$result['location']['text'] = $state([]);

	const positionOptions: PositionOptions = {
		enableHighAccuracy: true,
		timeout: 5000,
		maximumAge: 0
	};

	const searchLocations = async () => {
		if (!query) {
			toast.error('Gotta type something');
			return;
		}

		disabled = true;
		const url = `/api/location?query=${encodeURIComponent(query)}`;

		const response = await fetch(url, {
			method: 'GET'
		});

		options = (await response.json()) as LocationByText$result['location']['text'];
		disabled = false;
	};

	const getLocation = async () => {
		if (!navigator.geolocation) {
			toast.error('Location not available');
		}

		disabled = true;
		const result = await navigator.permissions.query({ name: 'geolocation' });
		switch (result.state) {
			case 'granted':
			case 'prompt':
				navigator.geolocation.getCurrentPosition(
					async (position) => {
						const { latitude, longitude } = position.coords;
						const url = `/api/location?latitude=${encodeURIComponent(
							latitude
						)}&longitude=${encodeURIComponent(longitude)}`;

						const response = await fetch(url, {
							method: 'GET'
						});

						options = (await response.json()) as LocationByText$result['location']['text'];
						disabled = false;
					},
					(err: GeolocationPositionError) => {
						toast.error(`Error getting location: ${err.message}`);
						disabled = false;
					},
					positionOptions
				);
				break;
			case 'denied':
				toast.error('Location access denied');
				disabled = false;
				break;
		}
	};

	const setLocation = async (storeId: string, newStoreName: string) => {
		optionsDisabled = true;
		const body: UpdateLocationBody = { storeId };
		const response = await fetch('/api/location', { method: 'POST', body: JSON.stringify(body) });
		configStore.set({ storeName: newStoreName, storeId });

		if (response.ok) {
			toast('Location updated');
		} else {
			toast.error('Something went wrong');
		}
		optionsDisabled = false;
	};
</script>

<Card.Root>
	<Card.Header>
		<Card.Title>Search</Card.Title>
	</Card.Header>
	<Card.Content>
		<div class="flex gap-2">
			<div class="w-full">
				<Input
					id="query"
					type="username"
					placeholder="e.g. Armadale"
					name="query"
					bind:value={query}
					on:keyup={async (e) => {
						if (e.key === 'Enter') {
							await searchLocations();
						}
					}}
					required
				/>
			</div>
			<div>
				<Button
					{disabled}
					aria-disabled={disabled}
					variant="outline"
					size="icon"
					on:click={getLocation}
				>
					<Crosshair1 />
					<span class="sr-only">Search by coordinates</span>
				</Button>
			</div>
		</div>
		{#if options.length > 0}
			<div transition:slide>
				<div class="grid w-full items-center gap-4 pt-4">
					{#each options as location (location)}
						<div animate:flip transition:scale>
							<Button
								class="w-full"
								variant="outline"
								disabled={optionsDisabled}
								aria-disabled={optionsDisabled}
								on:click={() => setLocation(location.storeNumber, location.name)}
							>
								{location.name}
							</Button>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</Card.Content>
	<Card.Footer>
		<Button class="w-full" type="submit" on:click={searchLocations} {disabled}>Find</Button>
	</Card.Footer>
</Card.Root>
