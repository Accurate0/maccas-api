<script lang="ts">
	import { useMutation, useQuery } from '@sveltestack/svelte-query';
	import { Skeleton } from './ui/skeleton';
	import type { AddOfferResponse } from '../../routes/api/offers/[offerId]/+server';
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { writable } from 'svelte/store';

	export let offerId: number;
	export let id: string;
	export let removeSelf: () => void;

	const offerCode = writable('');
	const addOffer = useMutation(
		`add-${id}`,
		async () =>
			await fetch(`/api/offers/${offerId}`, { method: 'POST' })
				.then((r) => r.json())
				.then((j) => j as AddOfferResponse)
	);

	const removeOffer = useMutation(
		`remove-${id}`,
		async ({ id }: { id: string }) => await fetch(`/api/offers/${id}`, { method: 'DELETE' })
	);

	const refreshOffer = useMutation(
		`code-${id}`,
		async () =>
			await fetch(`/api/offers/${$addOffer.data?.id}`, { method: 'GET' })
				.then((r) => r.json())
				.then((j) => j as Omit<AddOfferResponse, 'id'>),
		{
			onSuccess: (data) => {
				offerCode.update(() => data.code);
			}
		}
	);

	onMount(async () => {
		const response = await $addOffer.mutateAsync();
		offerCode.update(() => response.code);
	});
</script>

{#if $addOffer.isLoading}
	<Skeleton class="bg-primary/50 h-[54px] w-full rounded-sm" />
{:else}
	<div
		class="flex flex-grow flex-row items-center justify-between rounded-sm bg-slate-700/10 p-3 text-center"
	>
		<div class="font-mono">{$offerCode} {id}</div>
		<div>
			<Button
				class="material-symbols-outlined cursor-pointer"
				disabled={$refreshOffer.isLoading}
				on:click={async (e) => {
					e.stopPropagation();
					await $refreshOffer.mutateAsync();
				}}>refresh</Button
			>
			<Button
				class="material-symbols-outlined cursor-pointer"
				disabled={$removeOffer.isLoading}
				on:click={async (e) => {
					e.stopPropagation();
					await $removeOffer.mutateAsync({ id: $addOffer.data?.id ?? "this can't be null" });
					removeSelf();
				}}>close</Button
			>
		</div>
	</div>
{/if}
