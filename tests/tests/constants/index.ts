export * from "./testContracts";
export * from "./constants";
export * from "./transactionConfigs";

// TESTING NOTES
//
// BLOCK TESTING
// - block dont seem to have gas limit but it's usually around 1500 tx per block
// and the time it takes to construct the block increases until around 12s for 1500tx
// - after the first 1500tx block, following block have around 100-300 tx per block
// until all blocks are incuded. 10 blockds for 3000tx
// - between 7k and 10k, for some reason block creation doesnt work and we get
// one Pool(ImmediatelyDropped) error
// and  Pool(TooLowPriority { old: 0, new: 0 })': 819 for the contract creation

// 8192 is the number of tx that can be sent to the Pool before it throws an error and drops all tx
// from the pool (we can see in the logs that the ‘ready’ field goes from 8192 to zero)

// It does say however, 8182/20480kB ready, 819/2048kB future and I’m not sure what that means
//
// INFINITE LOOP
// - infinite loop contract should throw out of gas error, but they don't and
// they are included in the block.
// - there are some rpc errors sometimes
// - the state remains unchanged tho (test with infinite incremental contract)
//
// FINITE LOOP
// - making a 1000 loop incr on a smart contract doesnt pass but doesnt throw
// error either (although it does include the tx in a block)
// => is there a problem with out of gas error
// =>probably because we don't have the concept of gas?
// - posting a tx that goes over the gas limit/tx does throw an out of gas error
//  in the debug log but not in js

//NB: https://github.com/paritytech/frontier/blob/master/frame/ethereum/src/lib.rs
// show that root=0 when error is thrown,
//which is something we can see when fethcing receipt
// also the current block limit is zero
