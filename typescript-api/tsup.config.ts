import { defineConfig } from 'tsup'
import { execSync } from 'child_process'

export default defineConfig([
  {
    entry: ['src/moonbeam'],
    outDir: 'dist/moonbeam',
    format: ['esm', 'cjs'],
    splitting: false,
    clean: true,
    onSuccess: async () => {
      console.log('Running tsc for moonbeam...')
      execSync('pnpm tsc -p src/moonbeam/tsconfig.json --emitDeclarationOnly', { stdio: 'inherit' })
    }
  },
  {
    entry: ['src/moonriver'],
    outDir: 'dist/moonriver',
    format: ['esm', 'cjs'],
    splitting: false,
    clean: true,
    onSuccess: async () => {
      console.log('Running tsc for moonriver...')
      execSync('pnpm tsc -p src/moonriver/tsconfig.json --emitDeclarationOnly', { stdio: 'inherit' })
    }
  },
  {
    entry: ['src/moonbase'],
    outDir: 'dist/moonbase',
    format: ['esm', 'cjs'],
    splitting: false,
    clean: true,
    onSuccess: async () => {
      console.log('Running tsc for moonbase...')
      execSync('pnpm tsc -p src/moonbase/tsconfig.json --emitDeclarationOnly', { stdio: 'inherit' })
    }
  }
])
