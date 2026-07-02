import type { WasmCoefficient } from '$lib/types/wasm';
import type { Point } from '$lib/types/canvas';

export function computePeriod(_coeffs: WasmCoefficient[]): number {
	// For Fourier series visualization, we use 2*pi as the period.
	// The mathematical period (LCM of all frequencies) can be astronomically large
	// with many coefficients, making it impractical for animation.
	return 2 * Math.PI;
}

export function computeEpicycloidPoint(coeffs: WasmCoefficient[], time: number): Point {
	let x = 0;
	let y = 0;

	for (let i = 0; i < coeffs.length; i++) {
		const coeff = coeffs[i];
		const radius = Math.sqrt(coeff.real * coeff.real + coeff.imag * coeff.imag);
		const phase = Math.atan2(coeff.imag, coeff.real);
		const angle = coeff.n * time + phase;

		x += radius * Math.cos(angle);
		y += radius * Math.sin(angle);
	}

	return { x, y };
}

export function precomputeEpicycloid(coeffs: WasmCoefficient[], numPoints: number = 2000): Point[] {
	const period = computePeriod(coeffs);
	const points: Point[] = [];

	for (let i = 0; i < numPoints; i++) {
		const t = (i / numPoints) * period;
		points.push(computeEpicycloidPoint(coeffs, t));
	}

	return points;
}
