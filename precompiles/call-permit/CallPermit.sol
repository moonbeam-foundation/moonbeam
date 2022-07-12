// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

interface CallPermit {
    /// @dev Dispatch a call on the behalf of an other user with a EIP712 permit.
    /// Will revert if the permit is not valid or if the dispatched call reverts or errors (such as
    /// out of gas).
    /// If successful the EIP712 nonce is increased to prevent this permit to be replayed.
    /// @param from Who made the permit and want its call to be dispatched on their behalf.
    /// @param to Which address the call is made to.
    /// @param value Value being transfered from the "from" account.
    /// @param data Call data
    /// @param gaslimit Gaslimit the dispatched call requires.
    ///     Providing it prevents the dispatcher to manipulate the gaslimit.
    /// @param deadline Deadline in UNIX seconds after which the permit will no longer be valid.
    /// @param v V part of the signature.
    /// @param r R part of the signature.
    /// @param s S part of the signature.
    /// @return output Output of the call.
    /// Selector: b5ea0966
    function dispatch(
        address from,
        address to,
        uint256 value,
        bytes memory data,
        uint64 gaslimit,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external returns (bytes memory output);

    /**
     * @dev Returns the current nonce for given owner.
     * A permit must have this nonce to be consumed, which will
     * increase the nonce by one.
     * Selector: 0x7ecebe00
     */
    function nonces(address owner) external view returns (uint256);

    /**
     * @dev Returns the EIP712 domain separator. It is used to avoid replay
     * attacks accross assets or other similar EIP712 message structures.
     * Selector: 0x3644e515
     */
    function DOMAIN_SEPARATOR() external view returns (bytes32);
}
