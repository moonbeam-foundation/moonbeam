// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

// Contract that performs SSTORE operations
contract StorageWriter {
    mapping(uint256 => uint256) public storage_;
    
    function store(uint256 key, uint256 value) external {
        storage_[key] = value;
    }
    
    function load(uint256 key) external view returns (uint256) {
        return storage_[key];
    }
}

// Contract that uses transient storage (TSTORE/TLOAD)
contract TransientStorage {
    function storeTransient(uint256 slot, uint256 value) external {
        assembly {
            tstore(slot, value)
        }
    }
    
    function loadTransient(uint256 slot) external view returns (uint256 value) {
        assembly {
            value := tload(slot)
        }
    }
    
    function storeAndLoad(uint256 slot, uint256 value) external returns (uint256) {
        assembly {
            tstore(slot, value)
            value := tload(slot)
        }
        return value;
    }
}

// Contract that performs SELFDESTRUCT
contract SelfDestructor {
    function destruct(address payable recipient) external {
        selfdestruct(recipient);
    }
}

// Contract that creates other contracts
contract ContractCreator {
    event ContractCreated(address indexed addr);
    
    function createContract() external returns (address) {
        bytes memory bytecode = hex"6080604052348015600f57600080fd5b50603f80601d6000396000f3fe6080604052600080fdfea2646970667358221220";
        address addr;
        assembly {
            addr := create(0, add(bytecode, 0x20), mload(bytecode))
        }
        emit ContractCreated(addr);
        return addr;
    }
    
    function createContract2(bytes32 salt) external returns (address) {
        bytes memory bytecode = hex"6080604052348015600f57600080fd5b50603f80601d6000396000f3fe6080604052600080fdfea2646970667358221220";
        address addr;
        assembly {
            addr := create2(0, add(bytecode, 0x20), mload(bytecode), salt)
        }
        emit ContractCreated(addr);
        return addr;
    }
}

// Contract that performs various calls
contract Caller {
    event CallResult(bool success, bytes data);
    
    function callAddress(address target, bytes calldata data) external returns (bool, bytes memory) {
        (bool success, bytes memory result) = target.call(data);
        emit CallResult(success, result);
        return (success, result);
    }
    
    function delegatecallAddress(address target, bytes calldata data) external returns (bool, bytes memory) {
        (bool success, bytes memory result) = target.delegatecall(data);
        emit CallResult(success, result);
        return (success, result);
    }
    
    function staticcallAddress(address target, bytes calldata data) external view returns (bool, bytes memory) {
        (bool success, bytes memory result) = target.staticcall(data);
        return (success, result);
    }
}

// Contract that checks context (ADDRESS, BALANCE, etc.)
contract ContextChecker {
    function getAddress() external view returns (address) {
        return address(this);
    }
    
    function getBalance() external view returns (uint256) {
        return address(this).balance;
    }
    
    function getCodeSize() external view returns (uint256) {
        uint256 size;
        address addr = address(this);
        assembly {
            size := extcodesize(addr)
        }
        return size;
    }
    
    function getCodeHash() external view returns (bytes32) {
        bytes32 hash;
        address addr = address(this);
        assembly {
            hash := extcodehash(addr)
        }
        return hash;
    }
    
    function getCaller() external view returns (address) {
        return msg.sender;
    }
    
    function getOrigin() external view returns (address) {
        return tx.origin;
    }
}

// Contract for testing re-entrancy
contract ReentrantCaller {
    uint256 public depth;
    uint256 public maxDepth;
    
    event ReentryDepth(uint256 depth);
    
    function reenter(address target, uint256 targetDepth) external {
        maxDepth = targetDepth;
        depth = 0;
        _reenter(target);
    }
    
    function _reenter(address target) internal {
        depth++;
        emit ReentryDepth(depth);
        
        if (depth < maxDepth) {
            (bool success,) = target.call(abi.encodeWithSignature("_reenter(address)", target));
            require(success, "Reentry failed");
        }
        
        depth--;
    }
}

// Contract that modifies storage and can revert
contract StorageModifier {
    mapping(uint256 => uint256) public values;
    bool public shouldRevert;
    
    function setValue(uint256 key, uint256 value) external {
        values[key] = value;
        if (shouldRevert) {
            revert("Forced revert");
        }
    }
    
    function setShouldRevert(bool _shouldRevert) external {
        shouldRevert = _shouldRevert;
    }
}

// Simple counter contract
contract Counter {
    uint256 public count;
    
    function increment() external {
        count++;
    }
    
    function decrement() external {
        require(count > 0, "Counter underflow");
        count--;
    }
    
    function reset() external {
        count = 0;
    }
}

// Contract that accepts ETH
contract EthReceiver {
    mapping(address => uint256) public deposits;
    
    receive() external payable {
        deposits[msg.sender] += msg.value;
    }
    
    function withdraw() external {
        uint256 amount = deposits[msg.sender];
        deposits[msg.sender] = 0;
        payable(msg.sender).transfer(amount);
    }
}