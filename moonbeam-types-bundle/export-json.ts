import { typesBundle } from ".";

console.log(
  JSON.stringify(
    {
      types: (typesBundle as any).spec.moonbeam.types[
        (typesBundle as any).spec.moonbeam.types.length - 1
      ].types,
      rpc: (typesBundle as any).spec.moonbeam.rpc,
    },
    null,
    2
  )
);

// console.log(JSON.stringify(typesBundle as any, null, 2));
