<script>
	import '../app.pcss';

	import { onNavigate } from '$app/navigation';

	onNavigate((navigation) => {
		if (!document.startViewTransition) {
			return;
		}

		return new Promise((resolve) => {
			document.startViewTransition(async () => {
				resolve();
				await navigation.complete;
			});
		});
	});
</script>

<h2 class="m-4 mb-1 p-1 text-3xl font-bold tracking-tight">Maccas</h2>
<div class="flex h-full justify-center">
	<slot />
</div>
