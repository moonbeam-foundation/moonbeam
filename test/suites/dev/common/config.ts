import { DevModeContext } from "@moonwall/cli";
import { RUNTIME_CONSTANTS } from "@moonwall/util";

const Runtimes = Object.keys(RUNTIME_CONSTANTS).map(name => name.toLowerCase());
type Runtime = typeof Runtimes[number];

type MultiRuntimeValue = {
    [key in Runtime]: any;
};

export function valueFromRuntime(context: DevModeContext, multiRuntimeValue: MultiRuntimeValue) {
    let runtimeChain = context.pjsApi.runtimeChain.toLowerCase();
    let runtime = runtimeChain.split(" ").filter(v => Runtimes.includes(v as Runtime)).join();
    return multiRuntimeValue[runtime];
}

export const gasLimit = (context) => valueFromRuntime(context, {
    moonbeam: 15000000n,
    moonriver: 30000000n,
    moonbase: 60000000n,
});
  