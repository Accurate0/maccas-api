<script lang="ts">
	import '../app.pcss';

	import { goto, onNavigate } from '$app/navigation';
	import { Toaster } from 'svelte-sonner';
	import { page } from '$app/stores';
	import * as Tabs from '$lib/components/ui/tabs';
	import * as Card from '$lib/components/ui/card';
	import { QueryClient, QueryClientProvider } from '@sveltestack/svelte-query';
	import type { LayoutData } from './$types';
	import { derived } from 'svelte/store';
	import { configStore } from '$lib/config';
	import { ModeWatcher } from 'mode-watcher';
	import { Sun, Moon } from 'radix-icons-svelte';
	import { toggleMode } from 'mode-watcher';
	import { Button } from '$lib/components/ui/button';

	const queryClient = new QueryClient();
	export let data: LayoutData;

	onNavigate((navigation) => {
		// @ts-expect-error
		if (!document.startViewTransition) {
			return;
		}

		return new Promise((resolve) => {
			// @ts-expect-error
			document.startViewTransition(async () => {
				resolve();
				await navigation.complete;
			});
		});
	});

	configStore.set(data.config);
	const storeName = derived(configStore, (c) => c?.storeName);
</script>

<svelte:head>
	<title>Maccas</title>
</svelte:head>

<Toaster richColors />
<ModeWatcher />

<h2 class="m-4 mb-1 p-1 text-3xl font-bold tracking-tight">
	<a href="/">Maccas</a>
</h2>
<div class="flex h-full justify-center">
	<QueryClientProvider client={queryClient}>
		<div class="w-full">
			<div class="space-bet flex justify-between align-baseline">
				<Tabs.Root value={$page.route.id?.replace('/', '') ?? undefined}>
					{#if !data.hideAll}
						<Tabs.List class="m-4 mb-0">
							<Tabs.Trigger on:click={() => goto('/offers')} value="offers">Offers</Tabs.Trigger>
							{#if data.showPoints}
								<Tabs.Trigger on:click={() => goto('/points')} value="points">Points</Tabs.Trigger>
							{/if}
							<Tabs.Trigger on:click={() => goto('/location')} value="location"
								>Location</Tabs.Trigger
							>
						</Tabs.List>
					{/if}
				</Tabs.Root>
				<div class="m-4 mb-0">
					<Button on:click={toggleMode} variant="outline" size="icon">
						<Sun
							class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0"
						/>
						<Moon
							class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
						/>
						<span class="sr-only">Toggle theme</span>
					</Button>
				</div>
			</div>

			{#if $storeName && !data.hideAll}
				<div class="m-4 grid grid-flow-row gap-4">
					<Card.Root>
						<div class="grid grid-flow-col justify-between">
							<Card.Header class="grid justify-between">
								<Card.Title>Store</Card.Title>
								<Card.Description>{$storeName}</Card.Description>
							</Card.Header>
						</div>
					</Card.Root>
				</div>
			{/if}
			<div class="m-4 grid grid-flow-row gap-4">
				<slot />
			</div>
		</div>
	</QueryClientProvider>
</div>
