<script lang="ts">
	import { EpicycloidCanvas } from '$lib/components';
	import ImageSelector from '$lib/components/ImageSelector.svelte';
	import type { WasmCoefficient } from '$lib/types/wasm';
	import type { ImageData } from '$lib/backend';

	let currentCoeffs: WasmCoefficient[] = [];

	function handleImageSelect(imageData: ImageData) {
		currentCoeffs = imageData.equation_data.coefficients;
	}
</script>

<div class="flex min-h-screen flex-col items-center justify-center gap-6 p-4">
	<h1 class="text-center">Epic<span class="text-gray">(ycloid)</span> Drawing</h1>

	<div class="relative flex w-full items-center justify-center">
		<div class="absolute top-0 right-[calc(50%+470px)] hidden w-80 lg:block">
			<ImageSelector onImageSelect={handleImageSelect} />
		</div>
		<div class="flex items-center justify-center">
			<EpicycloidCanvas coeffs={currentCoeffs} />
		</div>
	</div>

	<div class="w-full lg:hidden">
		<ImageSelector onImageSelect={handleImageSelect} />
	</div>

	<div class="flex gap-3">
		<a
			href="/about/maths"
			class="rounded-lg bg-bg px-6 py-2 text-sm font-semibold transition-colors hover:bg-bg1"
		>
			About Maths
		</a>
		<a
			href="/about/code"
			class="rounded-lg bg-bg px-6 py-2 text-sm font-semibold transition-colors hover:bg-bg1"
		>
			About code
		</a>
	</div>
</div>
