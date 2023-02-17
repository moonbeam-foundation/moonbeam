// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/referenda/Referenda.sol";
import "../../../precompiles/preimage/Preimage.sol";

/// @notice Smart contract to demonstrate how to use Referenda Precompile to self-upgrade
abstract contract ReferendaAutoUpgradeDemo {
    /// @notice The id of the track used for root execution
    uint16 public rootTrackId;

    /// @notice SetStorageCall index (pallet Index concatenated with call index, usually "\x00\x04")
    bytes2 public setStorageCallIndex;

    /// @notice The id of the track used for root execution
    uint16 public proposalId;

    /// @notice construct the smart contract with the track id to send the proposal to
    constructor(string memory trackName, bytes2 pSetStorageCallIndex) {
        setStorageCallIndex = pSetStorageCallIndex;
        uint16[] memory trackIds = REFERENDA_CONTRACT.trackIds();

        // This is obviously NOT THE RIGHT WAY to find/store the track id.
        // The track id should be passed as an argument of the constructor instead of the
        // track name.
        // In this demo, it is using the track name to show how to loop through the trackIds.
        for (uint256 i = 0; i < trackIds.length; i++) {
            Referenda.TrackInfo memory info = REFERENDA_CONTRACT.trackInfo(
                trackIds[i]
            );

            if (
                keccak256(abi.encodePacked((info.name))) ==
                keccak256(abi.encodePacked((trackName))) // Compare the 2 strings
            ) {
                rootTrackId = trackIds[i];
                return;
            }
        }

        revert("Couldn't find track");
    }

    /// TODO Add check for deposit value
    function autoUpgrade(
        bytes memory contractCode,
        bytes memory contractStorageKey
    ) public {
        bytes memory codeStorageKey = contractStorageKey;

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
            setStorageCallIndex,
            itemCountBytes,
            key,
            value
        );

        bytes32 preimageHash = PREIMAGE_CONTRACT.notePreimage(setStorageCall);

        // /// If the block count is lower than the minimum allowed, it will pick the minimum
        // /// allowed automatically.
        uint32 blockCount = 1;
        uint256 referendumId = REFERENDA_CONTRACT.submitAfter(
            rootTrackId,
            preimageHash,
            uint32(setStorageCall.length),
            blockCount
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
    constructor(string memory trackName, bytes2 pSetStorageCallIndex)
        ReferendaAutoUpgradeDemo(trackName, pSetStorageCallIndex)
    {}

    function version() external pure returns (uint256) {
        return 1;
    }
}

contract ReferendaAutoUpgradeDemoV2 is ReferendaAutoUpgradeDemo {
    constructor(string memory trackName, bytes2 pSetStorageCallIndex)
        ReferendaAutoUpgradeDemo(trackName, pSetStorageCallIndex)
    {}

    function version() external pure returns (uint256) {
        return 2;
    }
}
