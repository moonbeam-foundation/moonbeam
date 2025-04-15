#!/bin/bash

# Default values
BUILD_LAST_TRACING_RUNTIME="no"
SKIP_AUGMENT_API="no"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    -f|--force-rebuild)
      BUILD_LAST_TRACING_RUNTIME="yes"
      shift
      ;;
    -s|--skip-augment-api)
      SKIP_AUGMENT_API="yes"
      shift
      ;;
    -h|--help)
      echo "Usage: $0 [options]"
      echo "Options:"
      echo "  -f, --force-rebuild         Force rebuild of the tracing runtime"
      echo "  -s, --skip-augment-api      Skip regenerating TypeScript API augmentations from runtime metadata"
      echo "  -h, --help                  Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use -h or --help for usage information"
      exit 1
      ;;
  esac
done

echo 'Make sure you have built moonbeam-types-bundle and run "npm install" in the test/ folder.'

if [ -e test/moonbase-overrides/moonbase-runtime-local-substitute-tracing.wasm ]; then
  if [[ "$BUILD_LAST_TRACING_RUNTIME" == "yes" ]]; then
    echo "Forcing rebuild of tracing runtime..."
  else
    echo "Using existing tracing runtime. Use -f to force rebuild."
  fi
else
  BUILD_LAST_TRACING_RUNTIME="yes"
fi

if [[ "$BUILD_LAST_TRACING_RUNTIME" == "yes" ]]; then
  ./scripts/build-last-tracing-runtime.sh
  mkdir -p test/moonbase-overrides/
  mv build/wasm/moonbase-runtime-local-substitute-tracing.wasm test/moonbase-overrides/
fi

echo "Preparing tests dependencies…"
(cd moonbeam-types-bundle && pnpm i && pnpm build)

if [[ "$SKIP_AUGMENT_API" == "no" ]]; then
  echo "Regenerating TypeScript API augmentations from runtime metadata..."
  (cd typescript-api && pnpm i && scripts/runtime-upgrade.sh)
else
  echo "Skipping TypeScript API augmentation regeneration..."
  (cd typescript-api && pnpm i)
fi

echo "Run tracing tests…"
(cd test && pnpm install && pnpm compile-solidity && pnpm moonwall test dev_moonbase_tracing)
