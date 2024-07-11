import { DevModeContext } from "@moonwall/cli";
import { RUNTIME_CONSTANTS } from "@moonwall/util";

const Runtimes = Object.keys(RUNTIME_CONSTANTS).map(name => name.toLowerCase());
type Runtime = typeof Runtimes[number];

type MultiRuntimeValue = {
    [key in Runtime]: any;
};

export function valueFromRuntime(context: DevModeContext, multiRuntimeValue: MultiRuntimeValue) : bigint{
    let runtimeChain = context.pjsApi.runtimeChain.toLowerCase();
    let runtime = runtimeChain.split(" ").filter(v => Runtimes.includes(v as Runtime)).join();
    return multiRuntimeValue[runtime];
}

export const gasLimit = (context: DevModeContext) => gasPerSecond(context) * 3n / 4n;

export const gasPerSecond = (context: DevModeContext) => valueFromRuntime(context, {
    moonbeam: 20_000_000n,
    moonriver: 40_000_000n,
    moonbase: 80_000_000n,
});

export const weightPerSecond = (context: DevModeContext) => valueFromRuntime(context, {
    moonbeam: 500_000_000_000n,
    moonriver: 1_000_000_000_000n,
    moonbase: 2_000_000_000_000n,
});

export const gasPerPovBytes = (context: DevModeContext) => valueFromRuntime(context, {
    moonbeam: 4n,
    moonriver: 8n,
    moonbase: 16n,
});

export const gasLimitPovRatio = (context: DevModeContext) => valueFromRuntime(context, {
    moonbeam: 4n,
    moonriver: 8n,
    moonbase: 16n,
});

export const deadlineMiliSeconds = (context: DevModeContext) => valueFromRuntime(context, {
    moonbeam: 1000n,
    moonriver: 1000n,
    moonbase: 1000n,
});

export const gasPerWeight = (context: DevModeContext) => weightPerSecond(context) / gasPerSecond(context);
  
export const extrinsicGasLimit = (context: DevModeContext) => 
  innerExtrinsicGasLimit(weightPerSecond(context), gasPerSecond(context), deadlineMiliSeconds(context));

const innerExtrinsicGasLimit = (weightPerSecond: bigint, gasPerSecond: bigint, deadlineMiliSeconds: bigint) => {
    const gasPerWeight = weightPerSecond / gasPerSecond;
    const blockWeightLimit = weightPerSecond * deadlineMiliSeconds / 1000n;
    const blockGasLimit = blockWeightLimit / gasPerWeight;
    return (blockGasLimit * 3n) / 4n - blockGasLimit / 10n;
}