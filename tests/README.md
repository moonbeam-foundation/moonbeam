# Functional testing for Moonbeam

This folder contains a set of functional tests desgined for Moonbeam network.

It is written in typescript, using Mocha/Chai as Test framework.

## Test flow

Each group will start a dev service with the
[development spec](../node/src/chain_spec.rs) before executing the tests.

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

## Verbose mode

You can also add the node's logs to the output using the `MOONBEAM_LOG` env variable. Ex:

```
MOONBEAM_LOG="warn,rpc=trace" npm run test
```

The test script will find available ports above 20000 in order to ensure that it doesn't conflict
with any other running services.

# Debugging a Moonbeam node

The repository contains a pre-configured debugger configuration for VSCode with the **CodeLLDB**
(`vadimcn.vscode-lldb`) extension. In the Debug tab of VSCode the dropdown at the top list 2
configurations :

- **Build & Launch Moonbeam Node (Linux)** : cargo build the node with debug profile and launch it.
- **Launch Moonbeam Node (Linux)** : directly launch the node without calling cargo build. Usefull
  if the node is already build with the other profile, or if you compiled it manually.

> Jérémy : I had issues with CodeLLDB not being able to call cargo, which was resolved without
> explanation. If the first profile doesn't work for you, call manually `cargo build` then use the
> second profile.

When launching the debug session it will take some time before the node starts, but the terminal
containing the node output will appear when it is really starting.

You can explore the code and place a breakpoint on a line by left clicking on the left of the line
number. The execution will pause the next time this line is reached. The debug toolbar contains the
following buttons :

- Resume/Pause : Resume the execution if paused, pause the execution at the current location
  (pretty random) if running.
- Step over : Resume the execution until next line, or go one level up if the end of the current
  scope is reached.
- Step into : Resume the execution to go inside the immediatly next function call if any, otherwise
  step to next line.
- Step out : Resume the execution until the end of the scope is reached.
- Restart : Kill the program and start a new debuging session.
- Stop : Kill the program and end debugin session.

Breakpoints stay between debugging sessions. When multiple function calls are made on the same line,
multiple step into, step out, step into, ... can be requiered to go inside one of the chained
calls.

When paused, content of variables is showed in the debuging tab of VSCode. Some basic types are
displayed correctly (primitive types, Vec, Arc) but more complex types such as HashMap/BTreeMap
are not "smartly" displayed (content of the struct is shown by mapping is hidden in the complexity
of the implementation).

## Running Typescript tests with a debug node

By setting the environement variable `DEBUG_MODE=true`, the Typescript tests will not spawn its
own node and instead will connect to an external node running on ports 19931/19932/19933, which
are the ports used by the debug node.