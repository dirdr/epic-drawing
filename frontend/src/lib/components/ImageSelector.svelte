<script lang="ts">
	import { onMount } from 'svelte';
	import { getAllImages, type ImageData } from '$lib/backend';

	export let onImageSelect: (imageData: ImageData) => void;

	let images: ImageData[] = [];
	let selectedImageName: string | null = null;
	let loading = true;
	let error: string | null = null;

	onMount(async () => {
		try {
			const data = await getAllImages();
			images = data.images;
			if (images.length > 0) {
				selectedImageName = images[0].name;
				onImageSelect(images[0]);
			}
			loading = false;
		} catch (err) {
			console.error('Error loading images:', err);
			error = err instanceof Error ? err.message : 'Failed to load images';
			loading = false;
		}
	});

	function handleImageSelect(image: ImageData) {
		selectedImageName = image.name;
		onImageSelect(image);
	}
</script>

{#if !loading}
	<div
		class="flex h-[min(900px,90vw)] flex-col gap-4 rounded-2xl bg-bg-soft p-6 shadow-[-20px_20px_60px_-15px_rgba(0,0,0,0.5)]"
	>
		{#if error}
			<div class="flex flex-1 items-center justify-center">
				<div class="text-center text-red">
					<p>{error}</p>
				</div>
			</div>
		{:else}
			<div class="flex flex-col gap-2 overflow-y-auto pr-2">
				{#each images as image}
					<button
						class="group relative rounded-lg px-4 py-3.5 text-left transition-all duration-200 hover:bg-bg1 {selectedImageName ===
						image.name
							? 'bg-bg1'
							: ''}"
						on:click={() => handleImageSelect(image)}
					>
						<div class="flex items-center gap-3">
							<div
								class="h-2 w-2 rounded-full transition-colors {selectedImageName === image.name
									? 'bg-aqua'
									: 'bg-bg3'}"
							></div>
							<h3
								class="inline-block font-medium transition-colors {selectedImageName === image.name
									? 'border-b-2 border-aqua text-aqua'
									: 'text-fg2 group-hover:text-fg1'}"
							>
								{image.name}
							</h3>
						</div>
					</button>
				{/each}
			</div>
		{/if}
	</div>
{/if}
