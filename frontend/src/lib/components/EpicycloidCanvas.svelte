<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { browser } from '$app/environment';
	import type { WasmCoefficient } from '$lib/types/wasm';
	import type { Point } from '$lib/types/canvas';
	import type { CircleData, VectorData } from '$lib/types/drawing';
	import { precomputeEpicycloid, computePeriod } from '$lib/utils/epicycloid';

	let canvas: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null = null;

	let width = 0;
	let height = 0;
	let dpr = 1;

	let frameId: number | undefined;
	let lastTime = 0;
	let t = 0;

	let precomputedPath: Point[] = [];
	let period = 0;
	const numPrecomputedPoints = 3000;
	let scale = 1;
	let offsetX = 0;
	let offsetY = 0;

	const colors = {
		background: '#32302f',
		circle: ['#a8e6cf'],
		vector: ['#a8e6cf'],
		trace: '#ff6b6b',
		center: '#ffe66d',
		currentPoint: '#00ff88'
	};

	export let coeffs: WasmCoefficient[] = [
		{ real: 40, imag: 30, n: 2 },
		{ real: 60, imag: 56, n: -3 },
		{ real: 30, imag: 100, n: 2 }
	];

	function initializePrecomputedPath(coeffs: WasmCoefficient[]) {
		if (coeffs.length === 0) {
			precomputedPath = [];
			period = 0;
			scale = 1;
			offsetX = 0;
			offsetY = 0;
			return;
		}
		period = computePeriod(coeffs);
		precomputedPath = precomputeEpicycloid(coeffs, numPrecomputedPoints);
		updateScaleAndOffset();
		t = 0;
	}

	$: initializePrecomputedPath(coeffs);

	function updateScaleAndOffset() {
		if (precomputedPath.length === 0 || width === 0 || height === 0) return;

		let minX = Infinity,
			maxX = -Infinity,
			minY = Infinity,
			maxY = -Infinity;
		for (const p of precomputedPath) {
			minX = Math.min(minX, p.x);
			maxX = Math.max(maxX, p.x);
			minY = Math.min(minY, p.y);
			maxY = Math.max(maxY, p.y);
		}

		// Calculate bounding box dimensions
		const boundingWidth = maxX - minX;
		const boundingHeight = maxY - minY;
		const centerX = (minX + maxX) / 2;
		const centerY = (minY + maxY) / 2;

		// Calculate scale to fit with padding (85% of available space)
		const padding = 0.85;
		const scaleX = (width * padding) / boundingWidth;
		const scaleY = (height * padding) / boundingHeight;
		scale = Math.min(scaleX, scaleY);

		// Calculate offsets to center the drawing
		offsetX = -centerX;
		offsetY = -centerY;
	}

	function resize() {
		if (!canvas || !ctx) return;

		dpr = window.devicePixelRatio || 1;
		width = canvas.clientWidth;
		height = canvas.clientHeight;

		canvas.width = Math.floor(width * dpr);
		canvas.height = Math.floor(height * dpr);

		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

		updateScaleAndOffset();
	}

	function drawPoint(point: Point, radius: number, color: string) {
		if (!ctx) return;

		ctx.fillStyle = color;
		ctx.beginPath();
		ctx.arc(point.x, point.y, radius, 0, Math.PI * 2);
		ctx.fill();
	}

	function drawCircle(center: Point, radius: number, color: string, lineWidth = 2) {
		if (!ctx) return;

		ctx.strokeStyle = color;
		ctx.lineWidth = lineWidth;
		ctx.beginPath();
		ctx.arc(center.x, center.y, radius, 0, Math.PI * 2);
		ctx.stroke();
	}

	function drawVector(start: Point, end: Point, color: string, lineWidth = 2) {
		if (!ctx) return;

		ctx.strokeStyle = color;
		ctx.lineWidth = lineWidth;
		ctx.setLineDash([5, 3]);
		ctx.beginPath();
		ctx.moveTo(start.x, start.y);
		ctx.lineTo(end.x, end.y);
		ctx.stroke();
		ctx.setLineDash([]);
	}

	function drawTrace(trace: Point[]) {
		if (!ctx || trace.length < 2) {
			return;
		}

		ctx.strokeStyle = colors.trace;
		ctx.lineWidth = 3;
		ctx.beginPath();
		ctx.moveTo(trace[0].x, trace[0].y);
		for (let i = 1; i < trace.length; i++) {
			ctx.lineTo(trace[i].x, trace[i].y);
		}
		if (trace.length === precomputedPath.length) {
			ctx.closePath();
		}
		ctx.stroke();
	}

	function computeEpicycloidChain(
		coeffs: WasmCoefficient[],
		time: number
	): {
		circles: CircleData[];
		vectors: VectorData[];
		finalPoint: Point;
	} {
		const circles: CircleData[] = [];
		const vectors: VectorData[] = [];

		let currentPoint: Point = { x: 0, y: 0 };

		for (let i = 0; i < coeffs.length; i++) {
			const coeff = coeffs[i];
			const radius = Math.sqrt(coeff.real * coeff.real + coeff.imag * coeff.imag);
			const phase = Math.atan2(coeff.imag, coeff.real);
			const angle = coeff.n * time + phase;

			circles.push({
				center: { ...currentPoint },
				radius: radius,
				color: colors.circle[i % colors.circle.length]
			});

			const nextPoint: Point = {
				x: currentPoint.x + radius * Math.cos(angle),
				y: currentPoint.y + radius * Math.sin(angle)
			};

			vectors.push({
				start: { ...currentPoint },
				end: { ...nextPoint },
				color: colors.vector[i % colors.vector.length]
			});

			currentPoint = nextPoint;
		}

		return { circles, vectors, finalPoint: currentPoint };
	}

	function update(dt: number) {
		t += dt * 0.5;
	}

	function draw() {
		if (!ctx || width === 0 || height === 0 || precomputedPath.length === 0) return;

		ctx.fillStyle = colors.background;
		ctx.fillRect(0, 0, width, height);

		ctx.save();
		ctx.translate(width / 2, height / 2);
		ctx.scale(scale, -scale);
		ctx.translate(offsetX, offsetY);

		const normalizedTime = ((t % period) + period) % period;
		const currentIndex = Math.floor((normalizedTime / period) * precomputedPath.length);

		const endIndex = Math.min(currentIndex + 1, precomputedPath.length);
		const tracePoints = precomputedPath.slice(0, endIndex);
		drawTrace(tracePoints);

		const { circles, vectors, finalPoint } = computeEpicycloidChain(coeffs, t);

		circles.forEach((circle) => {
			drawCircle(circle.center, circle.radius, circle.color, 2);
			drawPoint(circle.center, 3, circle.color);
		});

		vectors.forEach((vector) => {
			drawVector(vector.start, vector.end, vector.color, 2);
		});

		drawPoint({ x: 0, y: 0 }, 5, colors.center);

		drawPoint(finalPoint, 6, colors.currentPoint);

		ctx.restore();
	}

	function loop(time: number) {
		const dt = (time - lastTime) / 1000;
		lastTime = time;

		update(dt);
		draw();

		frameId = requestAnimationFrame(loop);
	}

	onMount(() => {
		if (!browser) return;

		ctx = canvas.getContext('2d');
		resize();
		window.addEventListener('resize', resize);

		initializePrecomputedPath(coeffs);

		lastTime = performance.now();
		frameId = requestAnimationFrame(loop);
	});

	onDestroy(() => {
		if (!browser) return;

		if (frameId !== undefined) {
			cancelAnimationFrame(frameId);
		}

		window.removeEventListener('resize', resize);
	});
</script>

<div
	class="aspect-square w-[min(900px,90vw)] rounded-2xl p-4 shadow-[0_20px_60px_-15px_rgba(0,0,0,0.5)]"
>
	<canvas bind:this={canvas} class="block h-full w-full rounded-xl"></canvas>
</div>
