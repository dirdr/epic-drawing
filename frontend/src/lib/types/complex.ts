export interface Complex {
	re: number;
	im: number;
}

export function c(re: number, im = 0): Complex {
	return { re, im };
}

export function mul(a: Complex, b: Complex): Complex {
	return {
		re: a.re * b.re - a.im * b.im,
		im: a.re * b.im + a.im * b.re
	};
}

export function expi(theta: number): Complex {
	return {
		re: Math.cos(theta),
		im: Math.sin(theta)
	};
}
