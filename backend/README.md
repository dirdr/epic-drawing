# Epic Drawing Backend

[![CI](https://github.com/dirdr/epic_drawing_backend/actions/workflows/ci.yml/badge.svg)](https://github.com/dirdr/epic_drawing_backend/actions/workflows/ci.yml)

WebAssembly backend for computing Fourier coefficients to draw images using epicycloids. Extracts contours from images and generates the mathematical data needed for animated epicycle visualizations.

## Building

### WASM (for web use)

```bash
wasm-pack build --target web
```

### Debug Mode (generate contour images)

```bash
cargo run --bin generate-debug-images --features cli
```
