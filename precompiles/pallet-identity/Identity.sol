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
     *
     * @param subs The identity (new) sub-accounts.
     */
    function setSubs(Sub[] calldata subs) external;

    /**
     * Clear an account's identity info and all sub-accounts, and return all deposits.
     * Payment: All reserved balances on the account are returned.
     * The sender must have a registered identity.
     */
    function clearIdentity() external;

    /**
     * Request a judgement from a registar.
     * Payment: At most maxFee will be reserved for payment to the registar if
     * judgement given.
     * The sender must have a registered identity.
     * @param registarIndex The index of the registar whose judgement is requested.
     * @param maxFee The maximum fee that may be paid.
     */
    function requestJudgement(uint256 registarIndex, uint256 maxFee) external;

    /**
     * Cancel a previous request.
     * Payment: A previously reserved deposit is returned on success.
     * The sender must have a registered identity.
     * @param registarIndex The index of the registar whose judgement is
     * no longer requested.    
     */
    function cancelRequest(uint256 registarIndex) external;

    /**
     * Set the fee requiored for a judgement to be requested from a registar.
     * The sender must be the account of the registar whose index is
     * registarIndex.
     * @param registarIndex Index of the registar whose fee is to be set.
     * @param fee The new fee.
     */
    function setFee(uint256 registarIndex, uint256 fee) external;

    /**
     * Change the account associated with a registar.
     * The sender must be the account of the registar whose index is     
     * registarIndex.
     * @param registarIndex Index of the registar whose account is to be set.
     * @param newAddress New account address.
     */
    function setAccountId(uint256 registarIndex, address newAddress) external;

    /**
     * Set the field information for a registar.
     * The sender must be the account of the registar whose index is registarIndex.
     * @param registarIndex Index of the registar whose fee is to be set.
     * @param fields Fields that the registar concerns themselves with.
     * Is a bitmask that can be computed using the IDENTITY_ constants.
     */
    function setFields(uint256 registarIndex, uint256 fields) external;

    /**
     * Provide a judgement for an account's identity.
     * The sender must be the account of the reigstar whose index is registarIndex.
     * @param registarIndex The index of the registar whose judgement is being made.
     * @param target The account whose identity the judgement is upon. This must be
     * an account with a registered identity.
     * @param judgement The judgement of the registar of registarIndex about target.
     * Any value not starting with 0xff is interpreted as the FeePaid variant with
     * numeric value provided. Other values are:
     * - 0xff00... = Unknown
     * - 0xff01... = Reasonable
     * - 0xff02... = KnownGood
     * - 0xff03... = OutOfDate
     * - 0xff04... = LowQuality
     * - 0xff05... = Erroneous 
     * See Rust docs for more information :
     * https://paritytech.github.io/substrate/master/pallet_identity/enum.Judgement.html
     */
    function provideJudgement(uint256 registarIndex, address target, uint256 judgement) external;

    /**
     * Add the given account to the sender's subs.
     * Payment: Balance reserved by a previous setSubs call for one sub will be repatriated
     * to the sender.
     * The sender must have a registered sub identity of sub.
     * @param sub Sub-account.
     * @param data Data for this sub-account.
     */
    function addSub(address sub, bytes calldata data) external;

    /**
     * Alter the associated data of the given sub-account.
     * The sender must have a registered sub identity of sub.
     * @param sub Sub-account.
     * @param data Data for this sub-account.
     */
    function renameSub(address sub, bytes calldata data) external;

    /**
     * Remove the given account from the sender's subs.
     * Payment: Balance reserved by a previous setSubs call for one sub will be
     * repatriated to the sender.
     * The sender must have a registered sub identity of sub.
     * @param sub Sub-account.
     */
    function removeSub(address sub) external;

    /**
     * Remove the sender as a sub-account.
     * Payment: Balance reserved by a previous setSubs call for one sub will be
     * repatriated to the sender (not the original depositor).
     * The sender must have a registered super-identity.
     * NOTE: This should not normally be used, but is provided in the case that the non-controller
     * of an account is maliciously registered as a sub-account. 
     */
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