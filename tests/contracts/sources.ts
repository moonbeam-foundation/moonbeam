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
    
        /// Check whether the specified address is currently a staking nominator
        function is_nominator(address) external view returns (bool);
    
        // Now the dispatchables
    
        /// Join the set of collator candidates
        function join_candidates(uint256 amount) external;
    
        /// Request to leave the set of candidates. If successful, the account is immediately
        /// removed from the candidate pool to prevent selection as a collator, but unbonding is
        /// executed with a delay of BondDuration rounds.
        function leave_candidates() external;
    
        /// Temporarily leave the set of collator candidates without unbonding
        function go_offline() external;
    
        /// Rejoin the set of collator candidates if previously had called go_offline
        function go_online() external;
    
        /// Bond more for collator candidates
        function candidate_bond_more(uint256 more) external;
    
        /// Bond less for collator candidates
        function candidate_bond_less(uint256 less) external;
    
        /// If caller is not a nominator, then join the set of nominators
        /// If caller is a nominator, then makes nomination to change their nomination state
        function nominate(address collator, uint256 amount) external;
    
        /// Leave the set of nominators and, by implication, revoke all ongoing nominations
        function leave_nominators() external;
    
        /// Revoke an existing nomination
        function revoke_nomination(address collator) external;
    
        /// Bond more for nominators with respect to a specific collator candidate
        function nominator_bond_more(address candidate, uint256 more) external;
    
        /// Bond less for nominators with respect to a specific nominator candidate
        function nominator_bond_less(address candidate, uint256 less) external;
    }

    /// An even more dead simple example to call the precompile
    contract JoinCandidatesWrapper {
        /// The ParachainStaking wrapper at the known pre-compile address. This will be used to 
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
  StakingNominationAttaker: `
    pragma solidity >=0.8.0;
    

    interface ParachainStaking {
        // First some simple accessors
    
        /// Check whether the specified address is currently a staking nominator
        function is_nominator(address) external view returns (bool);
    
        // Now the dispatchables
    
        /// Join the set of collator candidates
        function join_candidates(uint256 amount) external;
    
        /// Request to leave the set of candidates. If successful, the account is immediately
        /// removed from the candidate pool to prevent selection as a collator, but unbonding is
        /// executed with a delay of BondDuration rounds.
        function leave_candidates() external;
    
        /// Temporarily leave the set of collator candidates without unbonding
        function go_offline() external;
    
        /// Rejoin the set of collator candidates if previously had called go_offline
        function go_online() external;
    
        /// Bond more for collator candidates
        function candidate_bond_more(uint256 more) external;
    
        /// Bond less for collator candidates
        function candidate_bond_less(uint256 less) external;
    
        /// If caller is not a nominator, then join the set of nominators
        /// If caller is a nominator, then makes nomination to change their nomination state
        function nominate(address collator, uint256 amount) external;
    
        /// Leave the set of nominators and, by implication, revoke all ongoing nominations
        function leave_nominators() external;
    
        /// Revoke an existing nomination
        function revoke_nomination(address collator) external;
    
        /// Bond more for nominators with respect to a specific collator candidate
        function nominator_bond_more(address candidate, uint256 more) external;
    
        /// Bond less for nominators with respect to a specific nominator candidate
        function nominator_bond_less(address candidate, uint256 less) external;
    }

    contract StakingNominationAttaker {
        /// The collator (ALITH) that this contract will benefit with nominations
        address public target = 0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac; 

        /// The ParachainStaking wrapper at the known pre-compile address.
    ParachainStaking public staking = ParachainStaking(0x0000000000000000000000000000000000000800);

        /// Take advantage of the EVMs reversion logic and the fact that it doesn't extend to
        /// Substrate storage to score free nominations for a collator condidate of our choosing
        function score_a_free_nomination() public payable{
            
            // We nominate our target collator with all the tokens provided
            staking.nominate(target, msg.value);
            revert("By reverting this transaction, we return the eth to the caller");
        }
    }`,

    RelayStakeEncoder: `
    // SPDX-License-Identifier: GPL-3.0-only
    pragma solidity >=0.8.0;

    /// @author The Moonbeam Team
    /// @title The interface through which solidity contracts will interact with Relay Encoder
    /// We follow this same interface including four-byte function selectors, in the precompile that
    /// wraps the pallet
    interface RelayEncoder {
        // dev Encode 'bond' relay call
        // @param controller_address: Address of the controller
        // @param amount: The amount to bond
        // @param reward_destination: uint8 selecting one of RewardDestination
        // @param specified_account: In case 'Account' is chosen in the previous parameter, this is the address of such account. Else can be 0
        // @returns The bytes associated with the encoded call
        function encode_bond(uint256 controller_address, uint256 amount, bytes memory reward_destination) external view returns (bytes memory result);

        // dev Encode 'bond_extra' relay call
        // @param amount: The extra amount to bond
        // @returns The bytes associated with the encoded call
        function encode_bond_extra(uint256 amount) external view returns (bytes memory result);

        // dev Encode 'unbond' relay call
        // @param amount: The amount to unbond
        // @returns The bytes associated with the encoded call
        function encode_unbond(uint256 amount) external view returns (bytes memory result);

        // dev Encode 'withdraw_unbonded' relay call
        // @param slashes: Weight hint, number of slashing spans
        // @returns The bytes associated with the encoded call
        function encode_withdraw_unbonded(uint32 slashes) external view returns (bytes memory result);

        // dev Encode 'validate' relay call
        // @param comission: Comission of the validator as parts_per_billion
        // @param blocked: Whether or not the validator is accepting more nominations
        // @returns The bytes associated with the encoded call
        function encode_validate(uint256 comission, bool blocked) external view returns (bytes memory result);

        // dev Encode 'nominate' relay call
        // @param nominees: An array of AccountIds corresponding to the accounts we will nominate
        // @param blocked: Whether or not the validator is accepting more nominations
        // @returns The bytes associated with the encoded call
        function encode_nominate(uint256 [] memory nominees) external view returns (bytes memory result);

        // dev Encode 'chill' relay call
        // @returns The bytes associated with the encoded call
        function encode_chill() external view returns (bytes memory result);

        // dev Encode 'set_payee' relay call
        // @param reward_destination: uint8 selecting one of RewardDestination
        // @param specified_account: In case 'Account' is chosen in the previous parameter, this is the address of such account. Else can be 0
        // @returns The bytes associated with the encoded call
        function encode_set_payee(bytes memory reward_destination) external view returns (bytes memory result);

        // dev Encode 'set_controller' relay call
        // @param controller: The controller address
        // @returns The bytes associated with the encoded call
        function encode_set_controller(uint256 controller) external view returns (bytes memory result);

        // dev Encode 'rebond' relay call
        // @param amount: The amount to rebond
        // @returns The bytes associated with the encoded call
        function encode_rebond(uint256 amount) external view returns (bytes memory result);

    }`,
};
