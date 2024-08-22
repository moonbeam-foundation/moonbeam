// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

contract P256Verify {
    function verify(
        bytes32 msg_hash,
        bytes32[2] memory signature,
        bytes32[2] memory public_key
    ) public view returns (bool) {
        bool output;

        bytes memory args = abi.encodePacked(
            msg_hash,
            signature[0],
            signature[1],
            public_key[0],
            public_key[1]
        );

        bool success;
        assembly {
            success := staticcall(not(0), 0x100, add(args, 32), mload(args), output, 0x20)
        }
        require(success, "p256verify precompile call failed");

        return output;
    }

    function test() public {
        bytes32[2] memory msg_hashes;
        bytes32[2][2] memory signatures;
        bytes32[2][2] memory public_keys;
        bool[2] memory expected_result;

        // Case 1 (valid)
        msg_hashes[0] = hex"b5a77e7a90aa14e0bf5f337f06f597148676424fae26e175c6e5621c34351955";
        signatures[0][0] = hex"289f319789da424845c9eac935245fcddd805950e2f02506d09be7e411199556";
        signatures[0][1] = hex"d262144475b1fa46ad85250728c600c53dfd10f8b3f4adf140e27241aec3c2da";
        public_keys[0][0] = hex"3a81046703fccf468b48b145f939efdbb96c3786db712b3113bb2488ef286cdc";
        public_keys[0][1] = hex"ef8afe82d200a5bb36b5462166e8ce77f2d831a52ef2135b2af188110beaefb1";
        expected_result[0] = true;

        // Case 2 (invalid)
        msg_hashes[1] = hex"d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b";
        signatures[1][0] = hex"6162630000000000000000000000000000000000000000000000000000000000";
        signatures[1][1] = hex"6162630000000000000000000000000000000000000000000000000000000000";
        public_keys[1][0] = hex"6162630000000000000000000000000000000000000000000000000000000000";
        public_keys[1][1] = hex"6162630000000000000000000000000000000000000000000000000000000000";
        expected_result[0] = false;

        for (uint256 i = 0; i < expected_result.length; i++) {
            bool result = verify(msg_hashes[i], signatures[i], public_keys[i]);
            if (expected_result[i]) {
                require(result, "Expected success");
            } else {
                require(!result, "Expected failure");
            }
        }
    }
}