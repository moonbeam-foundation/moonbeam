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

    // Modify all existing slots with a small value
    function modifyStorage(uint256 slot) public returns (bytes memory) {
        for (uint256 i = 0; i < slot + 1; i++) {
            if (largeStorage[i].length > 0) {
                bytes memory data = largeStorage[i];
                // Modify each byte in the existing data
                for (uint256 j = 0; j < data.length; j++) {
                    data[j] = bytes1(uint8((i + j) % 256));
                }
                largeStorage[i] = data;
            }
        }
        return largeStorage[slot];
    }

    // Modify multiple storage slots in one transaction
    function modifyStorageBatch(
        uint256 startSlot,
        uint256 count
    ) external returns (uint256) {
        uint256 totalSize = 0;
        for (uint256 i = 0; i < count; i++) {
            totalSize += modifyStorage(startSlot + i).length;
        }
        return totalSize;
    }
}
