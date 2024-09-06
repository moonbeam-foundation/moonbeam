// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

contract TraceFilter {
    constructor(bool should_revert) {
        if (should_revert) {
            revert();
        }
    }

    function call_ok() public pure {}

    function call_revert() public pure {
        revert();
    }

    function subcalls(address target0, address target1) public pure {
        try TraceFilter(target0).subsubcalls(target1) {} catch {}
        try TraceFilter(target0).subsubcalls(target1) {} catch {}
    }

    function subsubcalls(address target1) public pure {
        TraceFilter(target1).call_ok();
        TraceFilter(target1).call_revert();
    }

    function heavy_steps(uint256 store_steps, uint256 op_steps) external {
        while (store_steps != 0) {
            assembly {
                sstore(store_steps, store_steps)
            }
            store_steps--;
        }

        while (op_steps != 0) {
            op_steps--;
        }
    }

    // This part is to trace Wasm memory overflow
    uint256 public a;
    uint256 public b;
    uint256 public c;
    uint256 public d;
    uint256 public e;
    uint256 public f;
    uint256 public g;
    uint256 public h;
    uint256 public i;
    uint256 public j;

    function set_and_loop(uint256 loops) public returns (uint256 result) {
        a = 1;
        b = 1;
        c = 1;
        d = 1;
        e = 1;
        f = 1;
        g = 1;
        h = 1;
        i = 1;
        j = 1;
        uint256 count = 0;
        while (i < loops) {
            count += 1;
        }
        return 1;
    }
}

event EventArgs0();
event EventArgs1(string);

contract TraceCallee {
    uint256 public store;

    function addtwo(uint256 _value) external returns (uint256 result) {
        uint256 x = 7;
        store = _value;
        return _value + x;
    }

    function emitSomeLogs(address addr) public {
        emit EventArgs0();
        emit EventArgs1("SUBCALL_TEST");

        TraceCaller caller = TraceCaller(addr);
        caller.someAction(address(this), 1);
    }
}

contract TraceCaller {
    TraceCallee internal callee;
    uint256 public store;

    function someAction(address addr, uint256 number) public {
        callee = TraceCallee(addr);
        store = callee.addtwo(number);
    }

    function emitSomeLogs(address addr) public {
        emit EventArgs0();
        emit EventArgs1("TEST");
        callee = TraceCallee(addr);
        callee.emitSomeLogs(address(this));
    }
}
