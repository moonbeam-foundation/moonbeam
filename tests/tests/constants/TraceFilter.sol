pragma solidity ^0.7.0;

contract TraceFilter {
    constructor(bool should_revert) {
        if (should_revert) {
            revert();
        }
    }

    function call_ok() public { }

    function call_revert() public {
        revert();        
    }

    function subcalls(address target0, address target1) public {
        try TraceFilter(target0).subsubcalls(target1) { } catch { }
        try TraceFilter(target0).subsubcalls(target1) { } catch { }
    }

    function subsubcalls(address target1) public {
        TraceFilter(target1).call_ok();
        TraceFilter(target1).call_revert();
    }
}