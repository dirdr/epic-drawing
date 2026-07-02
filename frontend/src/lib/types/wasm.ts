export interface WasmCoefficient {
	real: number;
	imag: number;
	n: number;
	phase?: number;
	radius?: number;
}

export interface WasmEquationData {
	coefficients: WasmCoefficient[];
	period: number;
}
