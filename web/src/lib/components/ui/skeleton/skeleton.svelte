<script lang="ts">
	import { cn } from '$lib/utils';
	import type { HTMLAttributes } from 'svelte/elements';

	type $$Props = HTMLAttributes<HTMLDivElement>;

	interface Props {
		class?: $$Props['class'];
		[key: string]: any;
	}

	let { class: className = undefined, ...rest }: Props = $props();
</script>

<div class={cn('animate-pulse rounded-md bg-primary/10', className)} {...rest}></div>
