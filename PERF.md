# Introduction

`perf` is a linux tool to profile cpu utilization within the system,
a process, or directly a thread. It can be used in multiple ways but this documentation only describe capturing with call-graph.

`perf` simply takes a snapshot of the cpu stack at the given frequency (usually 1000hz).  
It also provides a report computing how many times a snapshot stack contained each functions.
This roughly corresponds to how much cpu time is spent in each of them.

In addition to `perf` itself, some tools like [speedscope](https://github.com/jlfwong/speedscope) (or directly on https://www.speedscope.app/) allow to visualize the perf data under different dimensions (time-based, thread-based,...)

# Configuring moonbeam

Moonbeam node built with `--release`, contains enough symbols to have meanigful report for perf.
However it is suggest to also combine the build with additional frame pointers in order to capture the full stack:

```
RUSTFLAGS="-C force-frame-pointers=yes" cargo build --release
```

## Enabling WASM debugging

Because substrate nodes are designed to compile the wasm runtime when reading it (on-chain or locally), you can start your node with
`WASMTIME_PROFILING_STRATEGY=jitdump` so the wasm compiler will include symbols to the runtime. Ex:

```
WASMTIME_PROFILING_STRATEGY=jitdump moonbeam --dev --sealing 6000
```

# Capturing data

In most cases, we want to profile a specific feature of the node (producing a block, processing a request,...). However perf doesn't support this type of filtering.

`perf` requires to specify the `pid` of the running node. It is suggested to retrieve it
with a command. Ex:

```
# if you have only 1 node running
pgrep moonbeam

# if you have multiple nodes, use tricks to retrieve only the given node (port, command parameters...). Ex:
ps aux | grep moonbeam | grep -e 9944 | grep db-cache | tr -s " " | cut -d' ' -f 2
```

To start perf:

```
perf record -F 20000 -p $(pgrep moonbeam) --call-graph fp
```

To start perf with jitdump :

```
perf record -F 20000 -p $(pgrep moonbeam) -k mono --call-graph fp
```

(`-k mono` allows jit injection later)

To stop `perf`, use **<CTRL+C>**, it will stop recording and close after few seconds.

- **-F** is the **frequency** (20000hz), which allows to get more precise result. If you are tracking functions taking more than 1ms, 1000hz is good enough, otherwise you need to increase it. However the more you increase, the more likely you are to miss data. If perf can't profile fast enough, it will drop some snapshot to avoid impacting the process itself.  
  Suggested, based on function duration: 200ms: 50000, 1s: 20000, 10+s: 1000
- **-p** the **\<pid\>** of the node
- **--call-graph** enables the call-graph (the function stack) using the frame-pointer that we enabled during compilation
  (If you can't enable frame-pointer in the compilation, use `--call-graph dwarf` instead, and limit frequency to 2000 max)

# Generating script for speedscope

`perf` comes with a command `script` that allows to generate formatted data that can be used by other tools like speedscope. In order to produce a meaningful report.

If you want the whole report (with all the threads) simply run:

```
perf script --no-inline > perf.script.data

```

## Associating jitdump for wasm (optional)

If you enabled the JitDump profiler at the node compilation, you need to associate the data with the recording before running `perf script` with:

```
perf inject --jit --input perf.data --output perf.jit.data
```

(verify there is a file jit before with `ls jit*`)

## Only specific thread (optional)

perf script allows to specify the thread with `--tid <thread_id>` which will limit the output.  
One easy way to retrieve the thread id of the function you are looking for is to script in the script:

```
perf script --no-inline | less
```

then search (`/` key) for the function. Ex for `execute_in_transaction`:

```
tokio-runtime-w 16503 2925245.809827:     100010 cpu-clock:pppH:
        562394235bb6 wasmtime::func::Func::invoke+0x126 (...get/release/moonbeam)
        56239422a831 wasmtime::func::Func::new::_...87d04f+0x81 (...get/release/moonbeam)
        ...
        56239098dcff <sc_service::client::client::Clie...at+0xff (...get/release/moonbeam)
        5623900a6c0c sp_block_builder::runtime_d...all_api_at+0x28c (...get/release/moonbeam)
        56238fe47751 <moonbase_runtime::Runtim...>>::execute_in_transaction+0x201 (...get/release/moonbeam)
```

(The thread is `16503`, written on the first line of the block)

You can also use the following command to retrieve all the threads of execute_in_transaction using:

```
perf script --no-inline | awk -v RS='' '/execute_in_transaction/' | grep ' cycles:u:' | tr -s ' ' | sed 's/^[ \t]*//' | cut -d' ' -f2 | sort | uniq
```

(Each thread is a different execution, except in dev mode. So if for exemple your node imported 3 blocks during the recording,
it will have 3 threads. You can select the one you want for a specific block based on the order)

Finally, export the data (with my example):

```
perf script --no-inline --tid 16503 > perf.script.data
```

If you inserted jitdump, use:

```
perf script --no-inline --input perf.jit.data --tid 16503 > perf.script.data
```

You can also filter to remove noise generated by libc and other kernel parts:

```
perf script --no-inline --input perf.jit.data --tid 16503 | awk '
BEGIN { tokio_line = 0; }
{
    line[NR] = $0
    if ( length($0) == 0 && line[NR-1] ~ /thread_start/) {
        while ( tokio_line++ <= NR) { print line[tokio_line-1] }
    }
    if ( $0 ~ /^tokio-runtime-w/ ) { tokio_line = NR }
}
END { print $0 }' > perf.script.data
```

## Example of producing a block

The block production is not manual (at least in normal condition), so it requires to play a bit with perf to record only what we are looking for.

Either start the node with your synced chain, or launch a new one with `yarn run launch --chain local --port-prefix 12` (this will launch the node on port ws `12102`)

Open 3 terminals:

1.  with the node logs (`tail -f 12102.log` if using local parachain)
2.  with the command line ready to perf record (`perf record -F 9999 -p $(ps aux | grep moonbeam | grep 12102 | grep unsafe  | tr -s " " | cut -d' ' -f 2) --call-graph fp`)
3.  in the tools folder to generate some load

You can generate the load in many different way, ex: sending a bunch of request (`yarn ts-node scenarios/flood-evm-transfers.ts --url ws://localhost:12102 --eth-url http://localhost:12101 --amount 2 --count 1000`);

- Step 1: Generate the load
- Step 2: Look at the node logs until 1 block is produced
- Step 3: Start `perf record...` command very quickly after
- Step 4: Wait for the node logs to produce the next block (verify it contains the expected transactions)
- Step 5: Stop the `perf record...` command with \<Ctrl+C\>

At this point you should have a `perf.data` file (and a `jit-xxxx.dump` if you used jitdump), follow the `Generating script for speedscope` part.

## Exemple of debugging a wasm block production of a typescript-test

1. Compile the node

```
RUSTFLAGS="-C force-frame-pointers=yes" cargo build --release
```

2. Mark the test you want to execute with `it.only(...` (to avoid running other tests)
3. Run the perf with your node included:

```
WASMTIME_PROFILING_STRATEGY=jitdump perf record -F 10000 -k mono ./target/release/moonbeam --dev --sealing manual
```

4. (Another terminal) run your test:

```
TS_NODE_TRANSPILE_ONLY=true DEBUG_MODE=true npm run test-seq
```

5. Once the test passed (or failed), CTRL^C on the `perf` (first terminal) to stop the node.
6. Inject the JIT data inside the perf record data:

```
perf inject --jit --input perf.data --output perf.jit.data
```

7. Extract all the threads performing execute_in_transaction:

```
for i in $(perf script --no-inline | awk -v RS='' '/execute_in_transaction/' | grep ' cycles:u:' | tr -s ' ' | sed 's/^[ \t]*//' | cut -d' ' -f2 | sort | uniq); do echo $i; perf script --no-inline --tid $i --input perf.jit.data > perf-thread--$i.script.data; done
```

8. Open the `perf-thread--$i.script.data` files in https://www.speedscope.app/ and search (ctrl+f) for `execute_in_transaction`
