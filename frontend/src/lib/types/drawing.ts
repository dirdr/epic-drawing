import type { Point } from './canvas';

export interface CircleData {
	center: Point;
	radius: number;
	color: string;
}

export interface VectorData {
	start: Point;
	end: Point;
	color: string;
}

export interface EpicycloidCircle {
	center: Point;
	radius: number;
	angle: number;
	frequency: number;
}

export interface DrawingState {
	circles: CircleData[];
	vectors: VectorData[];
	trace: Point[];
	currentPoint: Point | null;
}
