import Web3 from "web3";
import { JsonRpcResponse } from "web3-core-helpers";
import { spawn, ChildProcess } from "child_process";

export const RPC_PORT = 19933;
export const SPECS_PATH = `./moonbeam-test-specs`;

export const DISPLAY_LOG = process.env.MOONBEAM_LOG || false;
export const MOONBEAM_LOG = process.env.MOONBEAM_LOG || "info";

export const BINARY_PATH = `../target/debug/node-moonbeam`;
export const SPAWNING_TIME = 30000;

export async function customRequest(web3: Web3, method: string, params: any[]) {
	return new Promise<JsonRpcResponse>((resolve, reject) => {
		(web3.currentProvider as any).send(
			{
				jsonrpc: "2.0",
				id: 1,
				method,
				params,
			},
			(error: Error | null, result?: JsonRpcResponse) => {
				if (error) {
					reject(
						`Failed to send custom request (${method} (${params.join(",")})): ${
							error.message || error.toString()
						}`
					);
				}
				resolve(result);
			}
		);
	});
}

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(web3: Web3) {
	const response = await customRequest(web3, "engine_createBlock", [true, true, null]);
	if (!response.result) {
		throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
	}
}

export async function startMoonbeamNode(specFilename: string): Promise<{ web3: Web3; binary: ChildProcess }> {
	const web3 = new Web3(`http://localhost:${RPC_PORT}`);

	const cmd = BINARY_PATH;
	const args = [
		`--chain=${SPECS_PATH}/${specFilename}`,
		`--validator`, // Required by manual sealing to author the blocks
		`--execution=Native`, // Faster execution using native
		`--no-telemetry`,
		`--no-prometheus`,
		`--manual-seal`,
		`--no-grandpa`,
		`--force-authoring`,
		`-l${MOONBEAM_LOG}`,
		`--rpc-port=${RPC_PORT}`,
		`--ws-port=19944`, // not used
		`--tmp`,
	];
	const binary = spawn(cmd, args);
	binary.on("error", (err) => {
		if ((err as any).errno == "ENOENT") {
			console.error(
				`\x1b[31mMissing Moonbeam binary (${BINARY_PATH}).\nPlease compile the Moonbeam project:\ncargo build --bin moonbeam-node\x1b[0m`
			);
		} else {
			console.error(err);
		}
		process.exit(1);
	});

	const binaryLogs = [];
	await new Promise((resolve) => {
		const timer = setTimeout(() => {
			console.error(`\x1b[31m Failed to start Moonbeam Test Node.\x1b[0m`);
			console.error(`Command: ${cmd} ${args.join(" ")}`);
			console.error(`Logs:`);
			console.error(binaryLogs.map((chunk) => chunk.toString()).join("\n"));
			process.exit(1);
		}, SPAWNING_TIME - 2000);

		const onData = async (chunk) => {
			if (DISPLAY_LOG) {
				console.log(chunk.toString());
			}
			binaryLogs.push(chunk);
			if (chunk.toString().match(/Manual Seal Ready/)) {
				// This is needed as the EVM runtime needs to warmup with a first call
				await web3.eth.getChainId();

				clearTimeout(timer);
				if (!DISPLAY_LOG) {
					binary.stderr.off("data", onData);
					binary.stdout.off("data", onData);
				}
				// console.log(`\x1b[31m Starting RPC\x1b[0m`);
				resolve();
			}
		};
		binary.stderr.on("data", onData);
		binary.stdout.on("data", onData);
	});

	return { web3, binary };
}

export function describeWithMoonbeam(title: string, specFilename: string, cb: (context: { web3: Web3 }) => void) {
	describe(title, () => {
		let context: { web3: Web3 } = { web3: null };
		let binary: ChildProcess;

		// Making sure the Moonbeam node has started
		before("Starting Moonbeam Test Node", async function () {
			this.timeout(SPAWNING_TIME);
			const init = await startMoonbeamNode(specFilename);
			context.web3 = init.web3;
			binary = init.binary;
		});

		after(async function () {
			//console.log(`\x1b[31m Killing RPC\x1b[0m`);
			binary.kill();
		});

		cb(context);
	});
}
