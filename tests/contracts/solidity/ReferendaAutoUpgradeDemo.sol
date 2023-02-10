// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/referenda/Referenda.sol";
import "../../../precompiles/preimage/Preimage.sol";

/// @notice Smart contract to demonstrate how to use Referenda Precompile to self-upgrade
abstract contract ReferendaAutoUpgradeDemo {
    /// @notice The id of the track used for root execution
    uint16 public rootTrackId;

    /// @notice The id of the track used for root execution
    uint16 public proposalId;

    /// @notice construct the smart contract with the track id to send the proposal to
    constructor(uint16 trackId) {
        rootTrackId = trackId;
    }

    /// TODO Add check for deposit value
    function autoUpgrade(
        bytes memory contractCode,
        bytes memory contractStorageKey
    ) public {
        bytes memory codeStorageKey = contractStorageKey;

        // Moonbase: Pallet index 00 , call index 05
        bytes memory systemSetStorageIndexes = bytes.concat(
            bytes("\x00"),
            bytes("\x05")
        );
        // 1 storage key to change, so 4 bytes
        bytes memory itemCountBytes = bytes("\x04");
        // Key value prefixed with key size in big endian (same for all the evm.accountStorage keys)
        // Add 1 for encodings
        uint16 keyLength = uint16(codeStorageKey.length * 4) + 1;
        // conversion to big endian and
        uint16 reversedkeyLength = ((keyLength >> 8) | (keyLength << 8));

        // Add 1 for encodings
        uint16 codeLength = uint16(contractCode.length * 4) + 1;
        // conversion to big endian and add 1 for encoding
        uint16 reversedCodeLength = ((codeLength >> 8) | (codeLength << 8));

        bytes memory key = bytes.concat(
            bytes2(reversedkeyLength),
            codeStorageKey
        );
        bytes memory value = bytes.concat(
            bytes2(reversedCodeLength),
            contractCode
        );
        bytes memory setStorageCall = bytes.concat(
            systemSetStorageIndexes,
            itemCountBytes,
            key,
            value
        );

        /// Size of the call + the transaction metadata (1 byte);
        uint16 txContentLength = uint16((setStorageCall.length) * 4 + 1);
        // Because of SCALE Compact encoding we need to have a dynamic size of the
        // transaction length
        bytes memory fullLength;

        if (txContentLength >= 64) {
            // 2 bytes
            fullLength = abi.encodePacked(
                ((txContentLength >> 8) | (txContentLength << 8))
            );
        } else {
            // 1 byte
            fullLength = abi.encodePacked(uint8(txContentLength));
        }

        bytes memory setStorageTx = bytes.concat(
            bytes32(uint256(txContentLength)),
            setStorageCall
        );

        // /// If the block count is lower than the minimum allowed, it will pick the minimum
        // /// allowed automatically.
        uint32 blockCount = 1;
        // /// Submit the proposal
        // REFERENDA_CONTRACT.submitAfter(rootTrackId, setStorageTx, blockCount);

        // uint256 referendumId = REFERENDA_CONTRACT.referendumCount();

        // /// TODO once the referendumInfo is available
        // /// Referenda.TrackInfo memory trackInfo = REFERENDA_CONTRACT.referendumInfo(referendumId);

        // /// Directly place the deposit
        // REFERENDA_CONTRACT.placeDecisionDeposit(uint32(referendumId));
    }
}

contract ReferendaAutoUpgradeDemoV1 is ReferendaAutoUpgradeDemo {
    constructor(uint16 trackId) ReferendaAutoUpgradeDemo(trackId) {}

    function version() external pure returns (uint256) {
        return 1;
    }
}

contract ReferendaAutoUpgradeDemoV2 is ReferendaAutoUpgradeDemo {
    constructor(uint16 trackId) ReferendaAutoUpgradeDemo(trackId) {}

    function version() external pure returns (uint256) {
        return 2;
    }
}
