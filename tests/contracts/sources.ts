export const contractSources: { [key: string]: string } = {
  TestContract: `
    pragma solidity >=0.8.0;
    
    contract TestContract {
        function multiply(uint a) public pure returns(uint d) {
            return a *7;
        }
    }`,
  FailContract: `
    pragma solidity >=0.8.0;
    
    contract FailContract {
        constructor() public {
            require(false);
        }
    }`,
  // simple incremental count contract to test contract with state changes
  TestContractIncr: `
    pragma solidity >=0.8.0;
    
    contract TestContractIncr {
        uint public count;
    
        constructor() public {
            count = 0;
        }
    
        function incr() public {
            count=count+1;
        }
    }`,
  // infinite loop call
  InfiniteContract: `
    pragma solidity >=0.8.0;
    
    contract InfiniteContract {
        function infinite() public pure returns(uint d) {while (true) {}}
    }`,
  // infinite loop call with variable allocation
  InfiniteContractVar: `
    pragma solidity >=0.8.0;
    
    contract InfiniteContractVar {
        uint public count;
    
        constructor() public {
            count = 0;
        }
    
        function infinite() public {
            while (true) {
                count=count+1;
            }
        }
    }`,
  // definite loop call with variable allocation
  FiniteLoopContract: `
    pragma solidity >=0.8.0;
    
    contract FiniteLoopContract {
        uint public count;
    
        constructor() public {
            count = 0;
        }
    
        function incr(uint n) public {
            uint i=0;
            while (i<n) {
                count=count+1;
                i+=1;
            }
        }
    }`,
  SingleEventContract: `
    pragma solidity >=0.8.0;
        
    contract SingleEventContract {
        event Constructed(address indexed owner);

        constructor() {
            emit Constructed(msg.sender);
        }
    }`,
  HashRipmd160: `
  pragma solidity >=0.8.0;
    contract HashRipmd160 {
      constructor() {
        require(ripemd160(bytes ('Hello World!')) ==
          hex'8476ee4631b9b30ac2754b0ee0c47e161d3f724c');
      }
    }`,
  Bn128Addition: `
    pragma solidity >=0.8.0;
    contract Bn128Addition{
        constructor() {
            bool success;
            uint256[4] memory input = [
                0x2243525c5efd4b9c3d3c45ac0ca3fe4dd85e830a4ce6b65fa1eeaee202839703,
                0x301d1d33be6da8e509df21cc35964723180eed7532537db9ae5e7d48f195c915,
                0x18b18acfb4c2c30276db5411368e7185b311dd124691610c5d3b74034e093dc9,
                0x063c909c4720840cb5134cb9f59fa749755796819658d32efc0d288198f37266
            ];
            uint256[2] memory result;

            assembly {
                // 0x06     id of the bn256Add precompile
                // 0        number of ether to transfer
                // 128      size of call parameters, i.e. 128 bytes total
                // 64       size of return value, i.e. 64 bytes / 512 bit for a BN256 curve point
                success := call(not(0), 0x06, 0, input, 128, result, 64)
            }
            require(success, "elliptic curve addition failed");
            require(
                result[0] ==
                0x2bd3e6d0f3b142924f5ca7b49ce5b9d54c4703d7ae5648e61d02268b1a0a9fb7,
            "failed");
            require(
                result[1] ==
                0x21611ce0a6af85915e2f1d70300909ce2e49dfad4a4619c8390cae66cefdb204,
            "failed");
        }
    }`,
  Bn128Multiply: `
    pragma solidity >=0.8.0;
    contract Bn128Multiply{
        constructor() {
            bool success;
            uint256[3] memory input = [
            0x1a87b0584ce92f4593d161480614f2989035225609f08058ccfa3d0f940febe3,
            0x1a2f3c951f6dadcc7ee9007dff81504b0fcd6d7cf59996efdc33d92bf7f9f8f6,
            0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000
            ];
            uint256[2] memory result;

            assembly {
                // 0x07     id of the bn256Mul precompile
                // 0        number of ether to transfer
                // 96       size of call parameters, i.e. 128 bytes total
                // 64       size of return value, i.e. 64 bytes / 512 bit for a BN256 curve point
                success := call(not(0), 0x07, 0, input, 96, result, 64)
            }
            require(success, "elliptic curve addition failed");
            require(
                result[0] ==
                0x1a87b0584ce92f4593d161480614f2989035225609f08058ccfa3d0f940febe3,
            "failed");
            require(
                result[1] ==
                0x163511ddc1c3f25d396745388200081287b3fd1472d8339d5fecb2eae0830451,
            "failed");
        }
    }`,
  Bn128Pairing: `
    pragma solidity >=0.8.0;
    contract Bn128Pairing {
        constructor() {
        uint256[12] memory input = [
            0x2eca0c7238bf16e83e7a1e6c5d49540685ff51380f309842a98561558019fc02,
            0x03d3260361bb8451de5ff5ecd17f010ff22f5c31cdf184e9020b06fa5997db84,
            0x1213d2149b006137fcfb23036606f848d638d576a120ca981b5b1a5f9300b3ee,
            0x2276cf730cf493cd95d64677bbb75fc42db72513a4c1e387b476d056f80aa75f,
            0x21ee6226d31426322afcda621464d0611d226783262e21bb3bc86b537e986237,
            0x096df1f82dff337dd5972e32a8ad43e28a78a96a823ef1cd4debe12b6552ea5f,
            0x06967a1237ebfeca9aaae0d6d0bab8e28c198c5a339ef8a2407e31cdac516db9,
            0x22160fa257a5fd5b280642ff47b65eca77e626cb685c84fa6d3b6882a283ddd1,
            0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
            0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
            0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b,
            0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa
        ];
        uint256[1] memory result;
        bool success;
        assembly {
            // 0x08     id of the bn256CheckPairing precompile
            // 0        number of ether to transfer
            // 0        since we have an array of fixed length, our input starts in 0
            // 384      size of call parameters, i.e. 12*256 bits == 384 bytes
            // 32       size of result (one 32 byte boolean!)
        success := call(sub(gas(), 2000), 0x08, 0, input, 384, result, 32)
        }
        require(success, "elliptic curve pairing failed");
        require(result[0] == 1, "failed");
        }
    }`,
  ModularCheck: `
    pragma solidity >=0.8.0;
    contract ModularCheck {
            // Verify simple modular exponentiation
        constructor() {
            require(modExp(3, 5, 7) == 5);
            require(modExp(5, 7, 11) == 3);
        }
            // Wrapper function to use the precompile.
        // Taken from https://ethereum.stackexchange.com/a/71590/9963
        function modExp(uint256 _b, uint256 _e, uint256 _m) public returns (uint256 result) {
            assembly {
                // Free memory pointer
                let pointer := mload(0x40)
                    // Define length of base, exponent and modulus. 0x20 == 32 bytes
                mstore(pointer, 0x20)
                mstore(add(pointer, 0x20), 0x20)
                mstore(add(pointer, 0x40), 0x20)
                    // Define variables base, exponent and modulus
                mstore(add(pointer, 0x60), _b)
                mstore(add(pointer, 0x80), _e)

                mstore(add(pointer, 0xa0), _m)
                // Store the result
                let value := mload(0xc0)
                    // Call the precompiled contract 0x05 = bigModExp
                if iszero(call(not(0), 0x05, 0, pointer, 0xc0, value, 0x20)) {
                    revert(0, 0)
                }
                    result := mload(value)
            }
        }
    }`,
  TraceFilter: `
    pragma solidity >=0.8.0;
    contract TraceFilter {
        constructor(bool should_revert) {
            if (should_revert) {
                revert();
            }
        }
    
        function call_ok() public { }
    
        function call_revert() public {
            revert();        
        }
    
        function subcalls(address target0, address target1) public {
            try TraceFilter(target0).subsubcalls(target1) { } catch { }
            try TraceFilter(target0).subsubcalls(target1) { } catch { }
        }
    
        function subsubcalls(address target1) public {
            TraceFilter(target1).call_ok();
            TraceFilter(target1).call_revert();
        }
    }`,
  Callee: `
    pragma solidity >=0.8.0;
    contract Callee {
        uint public store;
        function addtwo(uint _value) external returns (uint result) {
            uint x = 7;
            store = _value;
            return _value + x;
        }
    }`,
  Caller: `
    pragma solidity >=0.8.0;
    interface Callee {
        function addtwo(uint _value) external returns (uint result);
    }    
    contract Caller {
        Callee internal callee;
        uint public store;
        function someAction(address _addr, uint _number) public {
            callee = Callee(_addr);
            store = callee.addtwo(_number);
        }
    }`,
  Incrementer: `
    pragma solidity >=0.8.0;
    contract Incrementer {
        uint256 number;
        function sum(uint256 num) public returns (uint256){
            number += num;
            return number;
        }
    }`,
  CheckBlockVariables: `
    pragma solidity >=0.8.0;
    contract CheckBlockVariables {
        uint public initialgaslimit;
        uint public initialchainid;
        uint public initialnumber;
        
        constructor() {
            initialgaslimit = block.gaslimit;
            initialchainid = block.chainid;
            initialnumber = block.number;
        }

        function getGasLimit() public view returns (uint) {
            return block.gaslimit;
        }

        function getChainId() public view returns (uint) {
            return block.chainid;
        }

        function getNumber() public view returns (uint) {
            return block.number;
        }
    }`,
  JoinCandidatesWrapper: `
    pragma solidity >=0.8.0;

    interface ParachainStaking {
        // First some simple accessors
    
        /// Check whether the specified address is currently a staking delegator
        function is_delegator(address) external view returns (bool);
    
        // Now the dispatchables
    
        /// Join the set of collator candidates
        function join_candidates(uint256 amount) external;
    
        /// Request to leave the set of candidates. If successful, the account is immediately
        /// removed from the candidate pool to prevent selection as a collator, but unbonding is
        /// executed with a delay of BondDuration rounds.
        function schedule_leave_candidates() external;
    
        /// Temporarily leave the set of collator candidates without unbonding
        function go_offline() external;
    
        /// Rejoin the set of collator candidates if previously had called go_offline
        function go_online() external;
    
        /// Bond more for collator candidates
        function candidate_bond_more(uint256 more) external;
    
        /// Bond less for collator candidates
        function candidate_bond_less(uint256 less) external;
    
        /// If caller is not a delegator, then join the set of delegators
        /// If caller is a delegator, then makes delegation to change their delegation state
        function delegate(address candidate, uint256 amount) external;
    
        /// Leave the set of delegators and, by implication, revoke all ongoing delegations
        function leave_delegators() external;
    
        /// Revoke an existing delegation
        function revoke_delegation(address candidate) external;
    
        /// Bond more for delegators with respect to a specific collator candidate
        function delegator_bond_more(address candidate, uint256 more) external;
    
        /// Bond less for delegators with respect to a specific collator candidate
        function delegator_bond_less(address candidate, uint256 less) external;
    }

    /// An even more dead simple example to call the precompile
    contract JoinCandidatesWrapper {
        /// The ParachainStaking wrapper at the known precompile address. This will be used to 
        /// make all calls to the underlying staking solution
        ParachainStaking public staking;

        /// Solely for debugging purposes
        event Trace(uint256);

        constructor(address _staking) {
            staking = ParachainStaking(_staking);
        }

        receive() external payable {}

        function join() public {
            emit Trace(1 << 250);
            staking.join_candidates(1234 ether);
            emit Trace(2 << 250);
        }
    }`,
  OverflowingTrace: `
      pragma solidity >=0.8.0;
      contract OverflowingTrace {
          uint public a;
          uint public b;
          uint public c;
          uint public d;
          uint public e;
          uint public f;
          uint public g;
          uint public h;
          uint public i;
          uint public j;
          function set_and_loop(uint loops) public returns (uint result) {
              a = 1;
              b = 1;
              c = 1;
              d = 1;
              e = 1;
              f = 1;
              g = 1;
              h = 1;
              i = 1;
              j = 1;
              uint count = 0;
              while (i < loops) {
                count += 1;
              }
              return 1;
          }
      }`,
  StorageBloater: `
    pragma solidity >=0.8.0;
    contract StorageBloater {
      mapping(uint => uint) public bloat;
      uint256 sum = 0;

      function bloat_storage(uint start, uint num_items, uint seed) public {
        for (uint i=0; i<num_items; i++) {
          bloat[start + i] = start + i * seed;
        }
      }

      function calculate_sum(uint start, uint num_items) public {
        uint256 tmp = 0;
        for (uint i=0; i<num_items; i++) {
          tmp += bloat[start+i];
        }
        sum += tmp;
      }
    }`,
  Fibonacci: `
    pragma solidity>= 0.8.0;
    contract Fibonacci {
      function fib2(uint n) public returns(uint b) {
        if (n == 0) {
          return 0;
        }
        uint a = 1;
        b = 1;
        for (uint i = 2; i < n; i++) {
          uint c = a + b;
          a = b;
          b = c;
        }
        return b;
      }
    }`,
  StakingDelegationAttaker: `
    pragma solidity >=0.8.0;
    

    interface ParachainStaking {
        // First some simple accessors
    
        /// Check whether the specified address is currently a staking delegator
        function is_delegator(address) external view returns (bool);
    
        // Now the dispatchables
    
        /// Join the set of collator candidates
        function join_candidates(uint256 amount) external;
    
        /// Request to leave the set of candidates. If successful, the account is immediately
        /// removed from the candidate pool to prevent selection as a collator, but unbonding is
        /// executed with a delay of BondDuration rounds.
        function schedule_leave_candidates() external;
    
        /// Temporarily leave the set of collator candidates without unbonding
        function go_offline() external;
    
        /// Rejoin the set of collator candidates if previously had called go_offline
        function go_online() external;
    
        /// Bond more for collator candidates
        function schedule_candidate_bond_more(uint256 more) external;
    
        /// Bond less for collator candidates
        function schedule_candidate_bond_less(uint256 less) external;
    
        /// If caller is not a delegator, then join the set of delegators
        /// If caller is a delegator, then makes delegation to change their delegation state
        function delegate(address collator, uint256 amount) external;
    
        /// Leave the set of delegators and, by implication, revoke all ongoing delegations
        function schedule_leave_delegators() external;
    
        /// Revoke an existing delegation
        function revoke_delegation(address candidate) external;
    
        /// Bond more for delegators with respect to a specific collator candidate
        function delegator_bond_more(address candidate, uint256 more) external;
    
        /// Bond less for delegators with respect to a specific collator candidate
        function delegator_bond_less(address candidate, uint256 less) external;
    }

    contract StakingDelegationAttaker {
        /// The collator (ALITH) that this contract will benefit with delegations
        address public target = 0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac; 

        /// The ParachainStaking wrapper at the known pre-compile address.
    ParachainStaking public staking = ParachainStaking(0x0000000000000000000000000000000000000800);

        /// Take advantage of the EVMs reversion logic and the fact that it doesn't extend to
        /// Substrate storage to score free delegations for a collator candidate of our choosing
        function score_a_free_delegation() public payable{
            
            // We delegate our target collator with all the tokens provided
            staking.delegate(target, msg.value);
            revert("By reverting this transaction, we return the eth to the caller");
        }
    }`,
  RelayEncoderInstance: `
    // SPDX-License-Identifier: GPL-3.0-only
    pragma solidity >=0.8.0;

    /// @author The Moonbeam Team
    /// @title The interface through which solidity contracts will interact with Relay Encoder
    /// We follow this same interface including four-byte function selectors, in the precompile that
    /// wraps the pallet
    interface RelayEncoder {
        
        // dev Encode 'bond' relay call
        // Selector: 31627376
        // @param controller_address: Address of the controller
        // @param amount: The amount to bond
        // @param reward_destination: the account that should receive the reward
        // @returns The bytes associated with the encoded call
        function encode_bond(
            uint256 controller_address,
            uint256 amount,
            bytes memory reward_destination
        ) external pure returns (bytes memory result);
    
        // dev Encode 'bond_extra' relay call
        // Selector: 49def326
        // @param amount: The extra amount to bond
        // @returns The bytes associated with the encoded call
        function encode_bond_extra(uint256 amount) external pure returns (bytes memory result);
    
        // dev Encode 'unbond' relay call
        // Selector: bc4b2187
        // @param amount: The amount to unbond
        // @returns The bytes associated with the encoded call
        function encode_unbond(uint256 amount) external pure returns (bytes memory result);
    
        // dev Encode 'withdraw_unbonded' relay call
        // Selector: 2d220331
        // @param slashes: Weight hint, number of slashing spans
        // @returns The bytes associated with the encoded call
        function encode_withdraw_unbonded(
            uint32 slashes
        ) external pure returns (bytes memory result);
    
        // dev Encode 'validate' relay call
        // Selector: 3a0d803a
        // @param comission: Comission of the validator as parts_per_billion
        // @param blocked: Whether or not the validator is accepting more nominations
        // @returns The bytes associated with the encoded call
        // selector: 3a0d803a
        function encode_validate(
            uint256 comission,
            bool blocked
        ) external pure returns (bytes memory result);
    
        // dev Encode 'nominate' relay call
        // Selector: a7cb124b
        // @param nominees: An array of AccountIds corresponding to the accounts we will nominate
        // @param blocked: Whether or not the validator is accepting more nominations
        // @returns The bytes associated with the encoded call
        function encode_nominate(
            uint256 [] memory nominees
        ) external pure returns (bytes memory result);
    
        // dev Encode 'chill' relay call
        // Selector: bc4b2187
        // @returns The bytes associated with the encoded call
        function encode_chill() external pure returns (bytes memory result);
    
        // dev Encode 'set_payee' relay call
        // Selector: 9801b147
        // @param reward_destination: the account that should receive the reward
        // @returns The bytes associated with the encoded call
        function encode_set_payee(
            bytes memory reward_destination
        ) external pure returns (bytes memory result);
    
        // dev Encode 'set_controller' relay call
        // Selector: 7a8f48c2
        // @param controller: The controller address
        // @returns The bytes associated with the encoded call
        function encode_set_controller(
            uint256 controller
        ) external pure returns (bytes memory result);
    
        // dev Encode 'rebond' relay call
        // Selector: add6b3bf
        // @param amount: The amount to rebond
        // @returns The bytes associated with the encoded call
        function encode_rebond(uint256 amount) external pure returns (bytes memory result);
    }

    // We only use this to be able to generate the input data, since we need a compiled instance
    contract RelayEncoderInstance is RelayEncoder {
        /// The Relay Encoder wrapper at the known pre-compile address.
        RelayEncoder public relayencoder = RelayEncoder(0x0000000000000000000000000000000000000805);
        function encode_bond(
            uint256 controller_address,
            uint256 amount, bytes
            memory reward_destination
        )  external pure override returns (bytes memory result){
            return "0x00";
        }
        function encode_bond_extra(
            uint256 amount
        ) external pure override returns (bytes memory result){
            return "0x00";
        }
        function encode_unbond(
            uint256 amount
        ) external pure override returns (bytes memory result) {
            return "0x00";
        }
        function encode_withdraw_unbonded(
            uint32 slashes
        ) external pure override returns (bytes memory result) {
            return "0x00";
        }
        function encode_validate(
            uint256 comission,
            bool blocked
        ) external pure override returns (bytes memory result) {
            return "0x00";
        }
        function encode_nominate(
            uint256 [] memory nominees
        ) external pure override returns (bytes memory result) {
            return "0x00";
        }
        function encode_chill() external pure override returns (bytes memory result) {
            return "0x00";
        }
        function encode_set_payee(
            bytes memory reward_destination
        ) external pure override returns (bytes memory result) {
            return "0x00";
        }
        function encode_set_controller(
            uint256 controller
        ) external pure override returns (bytes memory result){
            return "0x00";
        }
        function encode_rebond(
            uint256 amount
        ) external pure override returns (bytes memory result){
            return "0x00";
        }
    }`,
  XtokensInstance: `
    pragma solidity >=0.8.0;

    /**
     * @title Xtokens Interface
     *
     * The interface through which solidity contracts will interact with xtokens pallet
     *
     */
    interface Xtokens {
        // A multilocation is defined by its number of parents and the encoded junctions (interior)
        struct Multilocation {
            uint8 parents;
            bytes [] interior;
        }

        /** Transfer a token through XCM based on its currencyId
         *
         * @dev The token transfer burns/transfers the corresponding amount before sending
         * @param currency_address The ERC20 address of the currency we want to transfer
         * @param amount The amount of tokens we want to transfer
         * @param destination The Multilocation to which we want to send the tokens
         * @param destination The weight we want to buy in the destination chain
         */
        function transfer(
            address currency_address,
            uint256 amount,
            Multilocation memory destination,
            uint64 weight
        ) external;

        /** Transfer a token through XCM based on its currencyId specifying fee
         *
         * @dev The token transfer burns/transfers the corresponding amount before sending
         * @param currency_address The ERC20 address of the currency we want to transfer
         * @param amount The amount of tokens we want to transfer
         * @param destination The Multilocation to which we want to send the tokens
         * @param destination The weight we want to buy in the destination chain
         */
        function transfer_with_fee(
            address currency_address,
            uint256 amount,
            uint256 fee,
            Multilocation memory destination,
            uint64 weight
        ) external;

        /** Transfer a token through XCM based on its MultiLocation
         *
         * @dev The token transfer burns/transfers the corresponding amount before sending
         * @param asset The asset we want to transfer, defined by its multilocation. 
         * Currently only Concrete Fungible assets
         * @param amount The amount of tokens we want to transfer
         * @param destination The Multilocation to which we want to send the tokens
         * @param destination The weight we want to buy in the destination chain
         */
        function transfer_multiasset(
            Multilocation memory asset,
            uint256 amount,
            Multilocation memory destination, uint64 weight) external;
        
        /** Transfer a token through XCM based on its MultiLocation specifying fee
         *
         * @dev The token transfer burns/transfers the corresponding amount before sending
         * @param asset The asset we want to transfer, defined by its multilocation. 
         * Currently only Concrete Fungible assets
         * @param amount The amount of tokens we want to transfer
         * @param destination The Multilocation to which we want to send the tokens
         * @param destination The weight we want to buy in the destination chain
         */
        function transfer_multiasset_with_fee(
            Multilocation memory asset,
            uint256 amount,
            uint256 fee,
            Multilocation memory destination, uint64 weight) external;
    }

    // Function selector reference
    // {
    // "b9f813ff": "transfer(address,uint256,(uint8,bytes[]),uint64)",
    // "b38c60fa": "transfer_multiasset((uint8,bytes[]),uint256,(uint8,bytes[]),uint64)"
    //}

    contract XtokensInstance is Xtokens {

    /// The Xtokens wrapper at the known pre-compile address.
    Xtokens public xtokens = Xtokens(0x0000000000000000000000000000000000000804);

        function transfer(
            address currency_address,
            uint256 amount,
            Multilocation memory destination,
            uint64 weight
        ) override external {
            // We nominate our target collator with all the tokens provided
            xtokens.transfer(currency_address, amount, destination, weight);
        }
        function transfer_with_fee(
            address currency_address,
            uint256 amount,
            uint256 fee,
            Multilocation memory destination,
            uint64 weight
        ) override external {
            // We nominate our target collator with all the tokens provided
            xtokens.transfer_with_fee(currency_address, amount, fee, destination, weight);
        }
        function transfer_multiasset(
            Multilocation memory asset,
            uint256 amount,
            Multilocation memory destination,
            uint64 weight
        ) override external {
            xtokens.transfer_multiasset(asset, amount, destination, weight);
        }
        function transfer_multiasset_with_fee(
            Multilocation memory asset,
            uint256 amount,
            uint256 fee,
            Multilocation memory destination,
            uint64 weight
        ) override external {
            xtokens.transfer_multiasset_with_fee(asset, amount, fee, destination, weight);
        }
    }`,
  XcmTransactorInstance: `
    // SPDX-License-Identifier: GPL-3.0-only
    pragma solidity >=0.8.0;

    /**
     * @title Xcm Transactor Interface
     *
     * The interface through which solidity contracts will interact with xcm transactor pallet
     *
     */
    interface XcmTransactor {
        // A multilocation is defined by its number of parents and the encoded junctions (interior)
        struct Multilocation {
            uint8 parents;
            bytes [] interior;
        }

        /** Get index of an account in xcm transactor
         *
         * @param index The index of which we want to retrieve the account
         */
        function index_to_account(uint16 index) external view returns(address);

        /** Get transact info of a multilocation
         * Selector 71b0edfa
         * @param multilocation The location for which we want to retrieve transact info
         */
        function transact_info(
            Multilocation memory multilocation) 
        external view  returns(uint64, uint256, uint64, uint64, uint256);

        /** Transact through XCM using fee based on its multilocation
         *
         * @dev The token transfer burns/transfers the corresponding amount before sending
         * @param transactor The transactor to be used
         * @param index The index to be used
         * @param fee_asset The asset in which we want to pay fees. 
         * It has to be a reserve of the destination chain
         * @param weight The weight we want to buy in the destination chain
         * @param inner_call The inner call to be executed in the destination chain
         */
        function transact_through_derivative_multilocation(
            uint8 transactor,
            uint16 index,
            Multilocation memory fee_asset,
            uint64 weight,
            bytes memory inner_call
        ) external;
        
        /** Transact through XCM using fee based on its currency_id
         *
         * @dev The token transfer burns/transfers the corresponding amount before sending
         * @param transactor The transactor to be used
         * @param index The index to be used
         * @param currency_id Address of the currencyId of the asset to be used for fees
         * It has to be a reserve of the destination chain
         * @param weight The weight we want to buy in the destination chain
         * @param inner_call The inner call to be executed in the destination chain
         */
        function transact_through_derivative(
            uint8 transactor,
            uint16 index,
            address currency_id,
            uint64 weight,
            bytes memory inner_call
        ) external;
    }

    contract XcmTransactorInstance is XcmTransactor {

    /// The Xcm Transactor wrapper at the known pre-compile address.
    XcmTransactor public xcmtransactor = XcmTransactor(0x0000000000000000000000000000000000000806);

        function index_to_account(uint16 index) external view override returns(address) {
            // We nominate our target collator with all the tokens provided
            return xcmtransactor.index_to_account(index);
        }

        function transact_info(
            Multilocation memory multilocation
        ) external view override returns(uint64, uint256, uint64, uint64, uint256) {
            // We nominate our target collator with all the tokens provided
            return xcmtransactor.transact_info(multilocation);
        }

        function transact_through_derivative_multilocation(
            uint8 transactor,
            uint16 index,
            Multilocation memory fee_asset,
            uint64 weight,
            bytes memory inner_call
        ) override external {
            xcmtransactor.transact_through_derivative_multilocation(
                transactor,
                index,
                fee_asset,
                weight,
                inner_call
            );
        }
        
        function transact_through_derivative(
            uint8 transactor,
            uint16 index,
            address currency_id,
            uint64 weight,
            bytes memory inner_call
        ) override external {
            xcmtransactor.transact_through_derivative(
                transactor,
                index,
                currency_id,
                weight,
                inner_call
            );
        }
    }`,
  // Blake2Check contract used to test blake2 precompile at address 0x9
  // source: https://eips.ethereum.org/EIPS/eip-152#example-usage-in-solidity
  Blake2Check: `
    pragma solidity >=0.8.0;

    contract Blake2Check {

      function F(
        uint32 rounds,
        bytes32[2] memory h,
        bytes32[4] memory m,
        bytes8[2] memory t,
        bool f
      ) public view returns (bytes32[2] memory) {

        bytes32[2] memory output;

        bytes memory args =
          abi.encodePacked(rounds, h[0], h[1], m[0], m[1], m[2], m[3], t[0], t[1], f);

        assembly {
          if iszero(staticcall(not(0), 0x09, add(args, 32), 0xd5, output, 0x40)) {
            revert(0, 0)
          }
        }

        return output;
      }

      function callF() public view returns (bytes32[2] memory) {
        uint32 rounds = 12;

        bytes32[2] memory h;
        h[0] = hex"48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5";
        h[1] = hex"d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b";

        bytes32[4] memory m;
        m[0] = hex"6162630000000000000000000000000000000000000000000000000000000000";
        m[1] = hex"0000000000000000000000000000000000000000000000000000000000000000";
        m[2] = hex"0000000000000000000000000000000000000000000000000000000000000000";
        m[3] = hex"0000000000000000000000000000000000000000000000000000000000000000";

        bytes8[2] memory t;
        t[0] = hex"03000000";
        t[1] = hex"00000000";

        bool f = true;

        // Expected output:
        // ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d1
        // 7d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923
        return F(rounds, h, m, t, f);
      }
    }`,
  ERC20Instance: `
    // SPDX-License-Identifier: GPL-3.0-only
    pragma solidity ^0.8.0;

    /**
     * @title ERC20 interface
     * @dev see https://github.com/ethereum/EIPs/issues/20
     * @dev copied from https://github.com/OpenZeppelin/openzeppelin-contracts
     */
    interface IERC20 {
        
    /**
     * @dev Returns the name of the token.
     * Selector: 06fdde03
     */
    function name() external view returns (string memory);

    /**
     * @dev Returns the symbol of the token.
     * Selector: 95d89b41
     */
    function symbol() external view returns (string memory);

    /**
     * @dev Returns the decimals places of the token.
     * Selector: 313ce567
     */
    function decimals() external view returns (uint8);
    
    /**
     * @dev Total number of tokens in existence
     * Selector: 18160ddd
     */
    function totalSupply() external view returns (uint256);

    /**
     * @dev Gets the balance of the specified address.
     * Selector: 70a08231
     * @param who The address to query the balance of.
     * @return An uint256 representing the amount owned by the passed address.
     */
    function balanceOf(address who) external view returns (uint256);

    /**
     * @dev Function to check the amount of tokens that an owner allowed to a spender.
     * Selector: dd62ed3e
     * @param owner address The address which owns the funds.
     * @param spender address The address which will spend the funds.
     * @return A uint256 specifying the amount of tokens still available for the spender.
     */
    function allowance(address owner, address spender)
        external view returns (uint256);

    /**
     * @dev Transfer token for a specified address
     * Selector: a9059cbb
     * @param to The address to transfer to.
     * @param value The amount to be transferred.
     */
    function transfer(address to, uint256 value) external returns (bool);

    /**
     * @dev Approve the passed address to spend the specified amount of tokens on behalf
     * of msg.sender.
     * Beware that changing an allowance with this method brings the risk that someone may
     * use both the old
     * and the new allowance by unfortunate transaction ordering. One possible solution to
     * mitigate this race condition is to first reduce the spender's allowance to 0 and set
     * the desired value afterwards:
     * https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
     * Selector: 095ea7b3
     * @param spender The address which will spend the funds.
     * @param value The amount of tokens to be spent.
     */
    function approve(address spender, uint256 value)
        external returns (bool);

    /**
     * @dev Transfer tokens from one address to another
     * Selector: 23b872dd
     * @param from address The address which you want to send tokens from
     * @param to address The address which you want to transfer to
     * @param value uint256 the amount of delegated tokens to be transferred
     */
    function transferFrom(address from, address to, uint256 value)
        external returns (bool);

    /**
     * @dev Event emited when a transfer has been performed.
     * Selector: ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
     * @param from address The address sending the tokens
     * @param to address The address receiving the tokens.
     * @param value uint256 The amount of tokens transfered.
     */
    event Transfer(
        address indexed from,
        address indexed to,
        uint256 value
    );

    /**
     * @dev Event emited when an approval has been registered.
     * Selector: 8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925
     * @param owner address Owner of the tokens.
     * @param spender address Allowed spender.
     * @param value uint256 Amount of tokens approved.
     */
    event Approval(
        address indexed owner,
        address indexed spender,
        uint256 value
    );
    }

    contract ERC20Instance is IERC20 {

        /// The ierc20 at the known pre-compile address.
        IERC20 public erc20 = IERC20(0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080);
        address erc20address = 0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080;

            receive() external payable {
            // React to receiving ether
            }

            function name() override external view returns (string memory) {
                // We nominate our target collator with all the tokens provided
                return erc20.name();
            }
            
            function symbol() override external view returns (string memory) {
                // We nominate our target collator with all the tokens provided
                return erc20.symbol();
            }
            
            function decimals() override external view returns (uint8) {
                // We nominate our target collator with all the tokens provided
                return erc20.decimals();
            }

            function totalSupply() override external view returns (uint256){
                // We nominate our target collator with all the tokens provided
                return erc20.totalSupply();
            }
            
            function balanceOf(address who) override external view returns (uint256){
                // We nominate our target collator with all the tokens provided
                return erc20.balanceOf(who);
            }
            
            function allowance(
                address owner,
                address spender
            ) override external view returns (uint256){
                return erc20.allowance(owner, spender);
            }

            function transfer(address to, uint256 value) override external returns (bool) {
                return erc20.transfer(to, value);
            }
            
            function transfer_delegate(address to, uint256 value) external returns (bool) {
            (bool result, bytes memory data) = erc20address.delegatecall(
                abi.encodeWithSignature("transfer(address,uint256)", to, value));
            return result;
            }
            
            function approve(address spender, uint256 value) override external returns (bool) {
            return erc20.approve(spender, value);
            }

            function approve_delegate(address spender, uint256 value) external returns (bool) {
            (bool result, bytes memory data) = erc20address.delegatecall(
                abi.encodeWithSignature("approve(address,uint256)", spender, value));
            return result;
            }
            
            function transferFrom(
                address from,
                address to,
                uint256 value)
            override external returns (bool) {
                return erc20.transferFrom(from, to, value);
            }
            
            function transferFrom_delegate(
                address from,
                address to,
                uint256 value) external returns (bool) {
            (bool result, bytes memory data) = erc20address.delegatecall(
                abi.encodeWithSignature("transferFrom(address,address,uint256)", from, to, value));
            return result;
            }
    }`,
  Democracy: `
    pragma solidity >=0.8.0;
    interface Democracy {
        // First some simple accessors
    
        /**
         * Get The total number of public proposals past and present
         * Selector: 56fdf547
         *
         * @return The total number of public proposals past and present
         */
        function public_prop_count() external view returns (uint256);
    
        /**
         * Get details about all public porposals.
         * Selector:
         * @return (prop index, proposal hash, proposer)
         * TODO This is supposed to be a vec. Let's save this one for later.
         */
        // function public_props()
        //     external
        //     view
        //     returns (
        //         uint256,
        //         bytes32,
        //         address
        //     );
    
        /**
         * Get the total amount locked behind a proposal.
         * Selector: a30305e9
         *
         * @dev Unlike the similarly-named Rust function this one only returns the value, not the
         * complete list of backers.
         * @param prop_index The index of the proposal you are interested in
         * @return The amount of tokens locked behind the proposal
         */
        function deposit_of(uint256 prop_index) external view returns (uint256);
    
        /**
         * Get the index of the lowest unbaked referendum
         * Selector: 0388f282
         *
         * @return The lowest referendum index representing an unbaked referendum.
         */
        function lowest_unbaked() external view returns (uint256);
    
        /**
         * Get the details about an ongoing referendum.
         * Selector: 8b93d11a
         *
         * @dev This, along with "finished_referendum_info", wraps the pallet's "referendum_info"
    * function. It is necessary to split it into two here because Solidity only has c-style enums.
         * @param ref_index The index of the referendum you are interested in
         * @return A tuple including:
         * * The block in which the referendum ended
         * * The proposal hash
         * * The baising mechanism 0-SuperMajorityApprove, 1-SuperMajorityAgainst, 2-SimpleMajority
         * * The delay between passing and launching
         * * The total aye vote (including conviction)
         * * The total nay vote (including conviction)
         * * The total turnout (not including conviction)
         */
        function ongoing_referendum_info(uint256 ref_index)
            external
            view
            returns (
                uint256,
                bytes32,
                uint256,
                uint256,
                uint256,
                uint256,
                uint256
            );
    
        /**
         * Get the details about a finished referendum.
         * Selector: b1fd383f
         *
         * @dev This, along with "ongoing_referendum_info", wraps the pallet's "referendum_info"
    * function. It is necessary to split it into two here because Solidity only has c-style enums.
         * @param ref_index The index of the referendum you are interested in
    * @return A tuple including whether the referendum passed, and the block at which it finished.
         */
        function finished_referendum_info(uint256 ref_index)
            external
            view
            returns (bool, uint256);
    
        // Now the dispatchables
    
        /**
         * Make a new proposal
         * Selector: 7824e7d1
         *
         * @param proposal_hash The hash of the proposal you are making
         * @param value The number of tokens to be locked behind this proposal.
         */
        function propose(bytes32 proposal_hash, uint256 value) external;
    
        /**
         * Signal agreement with a proposal
         * Selector: c7a76601
         *
        * @dev No amount is necessary here. Seconds are always for the same amount that the original
         * proposer locked. You may second multiple times.
         *
         * @param prop_index index of the proposal you want to second
    * @param seconds_upper_bound A number greater than or equal to the current number of seconds.
         * This is necessary for calculating the weight of the call.
         */
        function second(uint256 prop_index, uint256 seconds_upper_bound) external;
    
    //TODO should we have an alternative "simple_second" where the upper bound is read from storage?
    
        /**
         * Vote in a referendum.
         * Selector: 3f3c21cc
         *
         * @param ref_index index of the referendum you want to vote in
    * @param aye "true" is a vote to enact the proposal; "false" is a vote to keep the status quo.
         * @param vote_amount The number of tokens you are willing to lock if you get your way
        * @param conviction How strongly you want to vote. Higher conviction means longer lock time.
         * This must be an interget in the range 0 to 6
         *
         * @dev This function only supposrts "Standard" votes where you either vote aye xor nay.
         * It does not support "Split" votes where you vote on both sides. If such a need
         * arises, we should add an additional function to this interface called "split_vote".
         */
        function standard_vote(
            uint256 ref_index,
            bool aye,
            uint256 vote_amount,
            uint256 conviction
        ) external;
    
        /** Remove a vote for a referendum.
         * Selector: 2042f50b
         *
         * @dev Locks get complex when votes are removed. See pallet-democracy's docs for details.
         * @param ref_index The index of the referendum you are interested in
         */
        function remove_vote(uint256 ref_index) external;
    
        /**
         * Delegate voting power to another account.
         * Selector: 0185921e
         *
    * @dev The balance delegated is locked for as long as it is delegated, and thereafter for the
         * time appropriate for the conviction's lock period.
         * @param representative The account to whom the vote shall be delegated.
    * @param conviction The conviction with which you are delegating. This conviction is used for
         * _all_ delegated votes.
         * @param amount The number of tokens whose voting power shall be delegated.
         */
        function delegate(
            address representative,
            uint256 conviction,
            uint256 amount
        ) external;
    
        /**
         * Undelegatehe voting power
         * Selector: cb37b8ea
         *
    * @dev Tokens may be unlocked once the lock period corresponding to the conviction with which
         * the delegation was issued has elapsed.
         */
        function un_delegate() external;
    
        /**
         * Unlock tokens that have an expired lock.
         * Selector: 2f6c493c
         *
         * @param target The account whose tokens should be unlocked. This may be any account.
         */
        function unlock(address target) external;
    
        /**
         * Register the preimage for an upcoming proposal. This doesn't require the proposal to be
         * in the dispatch queue but does require a deposit, returned once enacted.
         * Selector: 200881f5
         *
        * @param encoded_proposal The scale-encoded proposal whose hash has been submitted on-chain.
         */
        function note_preimage(bytes memory encoded_proposal) external;
    
        /**
         * Register the preimage for an upcoming proposal. This requires the proposal to be
         * in the dispatch queue. No deposit is needed. When this call is successful, i.e.
         * the preimage has not been uploaded before and matches some imminent proposal,
         * no fee is paid.
         * Selector: cf205f96
         *
        * @param encoded_proposal The scale-encoded proposal whose hash has been submitted on-chain.
         */
        function note_imminent_preimage(bytes memory encoded_proposal) external;
    }`,
};
