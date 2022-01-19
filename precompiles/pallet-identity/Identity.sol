pragma solidity ^0.8.0;

/**
 * @title Identity interface.
 *
 * Interface through which Solidity contract can interact with Substrate pallet_identity.
 */
interface Identity {
    struct Sub {
        address subAccount;
        bytes data;
    }

    /**
     * Set an account's identity information and reserve the appropriate deposit.
     * If the account already has identity information, the deposit is taken as part
     * payment for the new deposit.
     *
     * @param info IdentityInfo, encoded using "encodeSize".
     */
    function setIdentity(bytes calldata info) external;

    /**
     * Set the sub-accounts of the sender.
     * Payment: Any aggregate balance reserved by previous setSubs calls will be returned and
     * an amount will be reserved for each item in "subs".
     * The sender must have a registered identity.
     */
    function setSubs(Sub[] calldata subs) external;

    function clearIdentity() external;

    function requestJudgement(uint256 registarIndex, uint256 maxFee) external;

    function cancelRequest(uint256 registarIndex) external;

    function setFee(uint256 registarIndex, uint256 fee) external;

    function setAccountId(uint256 registarIndex, address newAddress) external;

    function setFields(uint256 registarIndex, uint256 fields) external;

    function provideJudgement(uint256 registarIndex, address target, uint256 judgement) external;

    function addSub(address sub, bytes calldata data) external;

    function renameSub(address sub, bytes calldata data) external;

    function removeSub(address sub) external;

    function quitSub() external;
}

/**
 * Encoding library for IdentityInfo and IdentityData.
 * The Solidity interface wraps a pallet whose types were not designed with Ethereum
 * compatibility in mind. Particularily, IdentityInfo and IdentityData are not 32 bytes aligned.
 * To avoid a lot of padding (and thus reduce cost), this library allows to encode the data in a
 * packed format.
 */
library Encoding {
    struct Entry {
        bytes key;
        bytes value;
    }

    /**
     * Mirror of the [Substrate struct]
     * "pgp_fingerprint" must be filled with the result of either "pgpNode" or "pgpSome".
     * Other bytes fields must be filled with the result of any "dataXXX" function.
     * Functions expecting an IdentityInfo as bytes must be provided the result of "encodeInfo".
     *
     * [Substrate struct]: https://paritytech.github.io/substrate/master/pallet_identity/struct.IdentityInfo.html
     */
    struct IdentityInfo {
        Entry[] additional;
        bytes display;
        bytes legal;
        bytes web;
        bytes riot;
        bytes email;
        bytes pgp_fingerprint;
        bytes image;
        bytes twitter;
    }

    uint public constant IDENTITY_DISPLAY = 1;
    uint public constant IDENTITY_LEGAL = 2;
    uint public constant IDENTITY_WEB = 4;
    uint public constant IDENTITY_RIOT = 8;
    uint public constant IDENTITY_EMAIL = 16;
    uint public constant IDENTITY_PGP = 32;
    uint public constant IDENTITY_IMAGE = 64;
    uint public constant IDENTITY_TWITTER = 128;

    function memcpy(uint _dest, uint _src, uint _len) private pure {
        uint dest = _dest;
        uint src = _src;
        uint len = _len;

        for(; len >= 32; len -= 32) {
            assembly {
                mstore(dest, mload(src))
            }
            dest += 32;
            src += 32;
        }

        uint mask = 256 ** (32 - len) - 1;
        assembly {
            let srcpart := and(mload(src), not(mask))
            let destpart := and(mload(dest), mask)
            mstore(dest, or(destpart, srcpart))
        }
    }

    function append(uint target, bytes memory data) private pure returns (uint) {
        uint dataPtr;
        assembly { dataPtr := add(data, 0x20) }

        memcpy(target, dataPtr, data.length);        

        return target + data.length;
    }

    function newIdentityInfo() public pure returns (IdentityInfo memory) {
        Entry[] memory entries = new Entry[](0);

        IdentityInfo memory info = IdentityInfo(
            entries,
            dataNone(),
            dataNone(),
            dataNone(),
            dataNone(),
            dataNone(),
            pgpNone(),
            dataNone(),
            dataNone()
        );
        return info;
    }

    function encodeInfo(IdentityInfo memory info) public pure returns (bytes memory) {
        // 1. Compute size.
        uint totalSize = 0;

        uint i;
        for(i = 0; i < info.additional.length; i++) {
            totalSize += info.additional[i].key.length;
            totalSize += info.additional[i].value.length;
        }

        totalSize += info.display.length;
        totalSize += info.legal.length;
        totalSize += info.web.length;
        totalSize += info.riot.length;
        totalSize += info.email.length;
        totalSize += info.pgp_fingerprint.length;
        totalSize += info.image.length;
        totalSize += info.twitter.length;

        // 2. Allocate and copy.
        bytes memory output = new bytes(totalSize);
        uint offset;
        assembly { offset := add(output, 0x20) }

        for(i = 0; i < info.additional.length; i++) {
            offset = append(offset, info.additional[i].key);
            offset = append(offset, info.additional[i].value);
        }

        offset = append(offset, info.display);
        offset = append(offset, info.legal);
        offset = append(offset, info.web);
        offset = append(offset, info.riot);
        offset = append(offset, info.email);
        offset = append(offset, info.pgp_fingerprint);
        offset = append(offset, info.image);
        offset = append(offset, info.twitter);

        return output;
    }

    function pgpNone() public pure returns (bytes memory) {
        bytes memory b = new bytes(1);
        b[0] = 0x00;
        return b;
    }

    function pgpSome(bytes20 data) public pure returns (bytes memory) {
        bytes memory b = bytes.concat(bytes1(0x00), data);
        return b;
    }

    function dataNone() public pure returns (bytes memory) {
        bytes memory b = new bytes(1);
        b[0] = 0xff;
        return b;
    }

    function dataRaw(bytes memory data) public pure returns (bytes memory) {
        require(data.length <= 32, "length must be at most 32");
        bytes memory b = bytes.concat(bytes1(uint8(data.length)), data);
        return b;
    }

    function dataBlakeTwo256(bytes32 data) public pure returns (bytes memory) {
        bytes memory b = bytes.concat(bytes1(0xfe), data);
        return b;
    }

    function dataSha256(bytes32 data) public pure returns (bytes memory) {
        bytes memory b = bytes.concat(bytes1(0xfd), data);
        return b;
    }

    function dataKeccak256(bytes32 data) public pure returns (bytes memory) {
        bytes memory b = bytes.concat(bytes1(0xfc), data);
        return b;
    }

    function dataShaThree256(bytes32 data) public pure returns (bytes memory) {
        bytes memory b = bytes.concat(bytes1(0xfb), data);
        return b;
    }
}