<script>
	/**
	 * @typedef {Object} Props
	 * @property {import('svelte').Snippet} [children]
	 */

	/** @type {Props} */
	let { children } = $props();
</script>

<div class="flex items-center justify-center">
	<div class="absolute top-[40%]">
		{@render children?.()}
	</div>
</div>
