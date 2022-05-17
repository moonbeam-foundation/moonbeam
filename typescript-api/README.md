## Description

<a href="http://www.typescriptlang.org" target="_blank">TypeScript</a> type definitions that can be used to decorate the <a href="https://www.npmjs.com/package/@polkadot/api" target="_blank">@polkadot/api</a>.

## Installation

```bash
npm i @moonbeam-network/api-augment
```

> :warning: `@polkadot/api` should be installed in your project!

## Usage

Add to your codebase entry point before any imports from the API itself.

- `import '@moonbeam-network/api-augment'` - applies Moonbeam types and endpoint augmentation
- `import '@moonbeam-network/api-augment/moonriver'` - applies Moonriver types and endpoint augmentation
- `import '@moonbeam-network/api-augment/moonbase'` - applies Moonbase Alpha types and endpoint augmentation

## Docs

- <a href="https://polkadot.js.org/docs/api/examples/promise/typegen/" target="_blank">@polkadot/api</a> - TS type generation
- <a href="https://polkadot.js.org/docs/api/FAQ/#since-upgrading-to-the-7x-series-typescript-augmentation-is-missing" target="_blank">@polkadot/api</a> - Since upgrading to the 7.x series, TypeScript augmentation is missing
- <a href="https://polkadot.js.org/docs/api/start/typescript" target="_blank">@polkadot/api</a> - TypeScript interfaces

## Publish

Update package version.

```bash
npm version --no-git-tag-version 0.1500.0
```

Generate new types.

```bash
npm run generate
```

`The version change and new generated types should be merged to master.`

Build the package.

```bash
npm run build
```

`This will build the package and copy necessary files to the build folder.`

```bash
npm run publish
```

`This will publish content of the build folder.`
