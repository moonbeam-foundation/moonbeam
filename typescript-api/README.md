## Description

<a href="http://www.typescriptlang.org" target="_blank">TypeScript</a> type definitions that can be used to decorate the <a href="https://www.npmjs.com/package/@polkadot/api" target="_blank">@polkadot/api</a>.

## Installation

```bash
npm i @moonbeam/api-augment
```

## Usage

Add to your codebase entry point before any imports from the API itself.

- `import '@moonbeam/api-augment'` - applies Moonbeam types and endpoint augmentation
- `import '@moonbeam/api-augment/moonriver'` - applies Moonriver types and endpoint augmentation
- `import '@moonbeam/api-augment/moonbase'` - applies Moonbase Alpha types and endpoint augmentation

## Docs

- <a href="https://polkadot.js.org/docs/api/examples/promise/typegen/" target="_blank">@polkadot/api</a> - TS type generation
- <a href="https://polkadot.js.org/docs/api/FAQ/#since-upgrading-to-the-7x-series-typescript-augmentation-is-missing" target="_blank">@polkadot/api</a> - Since upgrading to the 7.x series, TypeScript augmentation is missing
- <a href="https://polkadot.js.org/docs/api/start/typescript" target="_blank">@polkadot/api</a> - TypeScript interfaces
