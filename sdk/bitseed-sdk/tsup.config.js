import { defineConfig } from 'tsup'

export default defineConfig({
  entry: ['src/index.ts', 'src/cli.ts'],
  format: ['esm', 'cjs', 'iife'],
  iife: {
    globalName: 'BitSeed',
  },
  splitting: false,
  minify: true,
  sourcemap: true,
  target: 'es2020',
})
