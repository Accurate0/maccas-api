<script lang="ts">
	export let deals: string[] | undefined;
	export let uuid: string;
	export let remove: (uuid: string, id: string) => void;
</script>

{#if (deals ?? []).length > 0}
	<md-list class="rounded pt-0 pb-0">
		{#each deals ?? [] as id, index}
			<md-list-item type="text">
				<div class="flex flex-row gap-8 justify-between items-center">
					<div class="w-full">
						<md-linear-progress class="w-full" indeterminate></md-linear-progress>
					</div>
					<div class="flex flex-row">
						<md-icon-button>
							<md-icon>refresh</md-icon>
						</md-icon-button>
						<!-- svelte-ignore a11y-no-static-element-interactions -->
						<!-- svelte-ignore a11y-click-events-have-key-events -->
						<md-icon-button on:click={() => remove(uuid, id)}>
							<md-icon>close</md-icon>
						</md-icon-button>
					</div>
				</div>
			</md-list-item>
			{#if index != (deals ?? []).length - 1}
				<md-divider inset-start inset-end></md-divider>
			{/if}
		{/each}
	</md-list>
{/if}
