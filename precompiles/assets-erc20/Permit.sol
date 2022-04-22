pragma solidity ^0.8.0;

/**
 * @title Extension of the ERC20 interface that allows users to
 * sign permit messages to interact with contracts without needing to
 * make a first approve transaction.
 */
interface Permit {
    function permit(
        address owner,
        address spender,
        uint value,
        uint deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external;

    function nonces(address owner) external view returns (uint);

    function DOMAIN_SEPARATOR() external view returns (bytes32);
}