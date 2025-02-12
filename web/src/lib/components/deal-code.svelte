<script lang="ts">
	import { useMutation } from '@sveltestack/svelte-query';
	import { Skeleton } from './ui/skeleton';
	import type { AddOfferResponse } from '../../routes/api/offers/[offerId]/+server';
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { writable } from 'svelte/store';

	interface Props {
		offerId: number;
		id: string;
		removeSelf: () => void;
	}

	let { offerId, id, removeSelf }: Props = $props();

	const offerCode = writable('');
	const addOffer = useMutation(
		`add-${id}`,
		async () =>
			await fetch(`/api/offers/${offerId}`, { method: 'POST' }).then(
				(r) => r.json() as Promise<AddOfferResponse>
			),
		{
			onError: () => {
				offerCode.update(() => 'Something went wrong');
			},
			onSuccess: (data) => {
				offerCode.update(() => data.code);
			}
		}
	);

	const removeOffer = useMutation(
		`remove-${id}`,
		async ({ id }: { id: string }) => await fetch(`/api/offers/${id}`, { method: 'DELETE' })
	);

	const refreshOffer = useMutation(
		`code-${id}`,
		async () =>
			await fetch(`/api/offers/${$addOffer.data?.id}`, { method: 'GET' }).then(
				(r) => r.json() as Promise<Omit<AddOfferResponse, 'id'>>
			),
		{
			onError: () => {
				offerCode.update(() => 'Something went wrong');
			},
			onSuccess: (data) => {
				offerCode.update(() => data.code);
			}
		}
	);

	onMount(() => {
		$addOffer.mutate();
		return () => {
			if ($addOffer.data?.id) {
				$removeOffer.mutate({ id: $addOffer.data.id });
			}
		};
	});
</script>

{#if $addOffer.isLoading}
	<Skeleton class="h-[54px] w-full rounded-sm bg-primary/50" />
{:else}
	<div
		class="flex flex-grow flex-row items-center justify-between rounded-sm bg-slate-700/10 p-3 text-center"
	>
		<div class="font-mono">{$offerCode}</div>
		<div>
			<Button
				class="material-symbols-outlined cursor-pointer"
				disabled={$refreshOffer.isLoading || $removeOffer.isLoading}
				on:click={async (e) => {
					e.stopPropagation();
					await $refreshOffer.mutateAsync();
				}}>refresh</Button
			>
			<Button
				class="material-symbols-outlined cursor-pointer"
				disabled={$removeOffer.isLoading || $refreshOffer.isLoading}
				on:click={async (e) => {
					e.stopPropagation();
					removeSelf();
				}}>close</Button
			>
		</div>
	</div>
{/if}
