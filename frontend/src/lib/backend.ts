import init, { get_all_images, get_image_names } from './wasm/epic_drawing_backend.js';
import type { WasmEquationData } from './types';

export interface ImageData {
	name: string;
	equation_data: WasmEquationData;
}

export interface AllImagesResponse {
	images: ImageData[];
}

let initialized = false;

/**
 * Initialize the WASM backend. Must be called before using any other functions.
 */
export async function initBackend(): Promise<void> {
	if (!initialized) {
		await init();
		initialized = true;
	}
}

/**
 * Get all available images with their pre-computed Fourier coefficients.
 * Automatically initializes the backend if not already initialized.
 */
export async function getAllImages(): Promise<AllImagesResponse> {
	await initBackend();
	return get_all_images() as AllImagesResponse;
}

/**
 * Get only the names of all available images.
 * Automatically initializes the backend if not already initialized.
 */
export async function getImageNames(): Promise<string[]> {
	await initBackend();
	return get_image_names() as string[];
}

/**
 * Get a specific image by name.
 * Returns undefined if the image is not found.
 */
export async function getImageByName(name: string): Promise<ImageData | undefined> {
	const data = await getAllImages();
	return data.images.find((img) => img.name === name);
}
