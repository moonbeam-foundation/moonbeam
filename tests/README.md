# Functional testing for Moonbeam

This folder contains a set of functional tests desgined for Moonbeam network.

It is written in typescript, using Mocha/Chai as Test framework.

## Test flow

Tests are separated depending of their genesis requirements.
Each group will start a standalone moonbeam node with the [test spec](moonbeam-test-specs/templates/simple-specs-template.json) before executing the tests.

## Installation

```
npm install
```

## Run the tests

```
npm run test
```

and to print more information:

```
npm run test-with-logs
```

This command will generate the simple-specs.json file from the template and execute the tests.  
To only generate the specs use `npm run generate-specs`.  
To only run the tests use `npm run test-only`.

## Verbose mode

You can also add the Frontier Node logs to the output using the `MOONBEAM_LOG` env variable. Ex:

```
MOONBEAM_LOG="warn,rpc=trace" npm run test
```

(The frontier node be listening for RPC on port 19933, mostly to avoid conflict with already running substrate node)
