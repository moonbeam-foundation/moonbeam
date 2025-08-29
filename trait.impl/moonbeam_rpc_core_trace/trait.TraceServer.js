(function() {
    var implementors = Object.fromEntries([["moonbeam_rpc_trace",[["impl&lt;B, C&gt; <a class=\"trait\" href=\"moonbeam_rpc_trace/trait.TraceServer.html\" title=\"trait moonbeam_rpc_trace::TraceServer\">TraceServer</a> for <a class=\"struct\" href=\"moonbeam_rpc_trace/struct.Trace.html\" title=\"struct moonbeam_rpc_trace::Trace\">Trace</a>&lt;B, C&gt;<div class=\"where\">where\n    B: BlockT&lt;Hash = H256&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.84.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.84.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,\n    B::Header: HeaderT&lt;Number = <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.84.1/std/primitive.u32.html\">u32</a>&gt;,\n    C: HeaderMetadata&lt;B, Error = Error&gt; + HeaderBackend&lt;B&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.84.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.84.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,</div>"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[1151]}