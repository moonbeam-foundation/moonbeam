import solc from "solc";

export function compileSolidity(contractContent: string, contractName: string = "Test") {
  let result = JSON.parse(
    solc.compile(
      JSON.stringify({
        language: "Solidity",
        sources: {
          "main.sol": {
            content: contractContent,
          },
        },
        settings: {
          outputSelection: {
            "*": {
              "*": ["*"],
            },
          },
        },
      })
    )
  );

  const contract = result.contracts["main.sol"][contractName];
  return {
    bytecode: "0x" + contract.evm.bytecode.object,
    contract,
  };
}
