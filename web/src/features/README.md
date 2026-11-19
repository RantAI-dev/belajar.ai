One folder per capability (`tanya/`, `belajar/`, `latihan/`, ...). A slice's `index.ts` is its
public surface; other slices import only from it. `scripts/check-boundaries.ts` enforces this.
