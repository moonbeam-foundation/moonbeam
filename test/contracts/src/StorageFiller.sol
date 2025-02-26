pragma solidity ^0.8.0;

contract StorageFiller {
    // Mapping to store large byte arrays
    mapping(uint256 => bytes) public largeStorage;

    // Fill a single storage slot with a large value
    function fillStorage(uint256 slot, uint256 size) public {
        bytes memory data = new bytes(size);
        // Fill with non-zero data to ensure it's stored
        for (uint i = 0; i < size; i++) {
            data[i] = bytes1(uint8((slot + i) % 256));
        }
        largeStorage[slot] = data;
    }

    // Fill multiple storage slots in one transaction
    function fillStorageBatch(
        uint256 startSlot,
        uint256 count,
        uint256 size
    ) public {
        for (uint256 i = 0; i < count; i++) {
            fillStorage(startSlot + i, size);
        }
    }

    // Read a single storage slot
    function readStorage(uint256 slot) public view returns (bytes memory) {
        return largeStorage[slot];
    }

    // Read multiple storage slots in one transaction
    function readStorageBatch(
        uint256 startSlot,
        uint256 count
    ) public view returns (uint256) {
        uint256 totalSize = 0;
        for (uint256 i = 0; i < count; i++) {
            totalSize += readStorage(startSlot + i).length;
        }
        return totalSize;
    }
}
