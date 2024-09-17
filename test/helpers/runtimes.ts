interface Runtime {
  specVersion: number;
  blockNumber: {
    moonbeam: bigint | null;
    moonriver: bigint | null;
    moonbase: bigint | null;
  };
}

export const runtimes: Runtime[] = [
  { specVersion: 40, blockNumber: { moonbeam: null, moonriver: null, moonbase: 0n } },
  { specVersion: 44, blockNumber: { moonbeam: null, moonriver: null, moonbase: 142863n } },
  { specVersion: 47, blockNumber: { moonbeam: null, moonriver: null, moonbase: 209144n } },
  { specVersion: 49, blockNumber: { moonbeam: null, moonriver: 0n, moonbase: null } },
  { specVersion: 52, blockNumber: { moonbeam: null, moonriver: null, moonbase: 238827n } },
  { specVersion: 53, blockNumber: { moonbeam: null, moonriver: 9696n, moonbase: null } },
  { specVersion: 155, blockNumber: { moonbeam: null, moonriver: 67938n, moonbase: 278703n } },
  { specVersion: 159, blockNumber: { moonbeam: null, moonriver: 166749n, moonbase: 383465n } },
  { specVersion: 200, blockNumber: { moonbeam: null, moonriver: 259002n, moonbase: 457614n } },
  { specVersion: 300, blockNumber: { moonbeam: null, moonriver: 344698n, moonbase: 485543n } },
  { specVersion: 400, blockNumber: { moonbeam: null, moonriver: 400458n, moonbase: 610935n } },
  { specVersion: 501, blockNumber: { moonbeam: null, moonriver: 430442n, moonbase: 653692n } },
  { specVersion: 600, blockNumber: { moonbeam: null, moonriver: 455107n, moonbase: 675176n } },
  { specVersion: 701, blockNumber: { moonbeam: null, moonriver: 581187n, moonbase: 797200n } },
  { specVersion: 800, blockNumber: { moonbeam: null, moonriver: 684728n, moonbase: 915684n } },
  { specVersion: 900, blockNumber: { moonbeam: 0n, moonriver: 923864n, moonbase: 1075626n } },
  { specVersion: 901, blockNumber: { moonbeam: null, moonriver: null, moonbase: 1130271n } },
  { specVersion: 902, blockNumber: { moonbeam: null, moonriver: null, moonbase: 1175311n } },
  { specVersion: 1001, blockNumber: { moonbeam: 5165n, moonriver: 1052242n, moonbase: 1285916n } },
  { specVersion: 1002, blockNumber: { moonbeam: 32532n, moonriver: 1141593n, moonbase: 1396972n } },
  {
    specVersion: 1101,
    blockNumber: { moonbeam: 171061n, moonriver: 1188000n, moonbase: 1426319n },
  },
  {
    specVersion: 1102,
    blockNumber: { moonbeam: 214641n, moonriver: 1295420n, moonbase: 1517440n },
  },
  {
    specVersion: 1103,
    blockNumber: { moonbeam: 312036n, moonriver: 1389122n, moonbase: 1591913n },
  },
  { specVersion: 1200, blockNumber: { moonbeam: null, moonriver: null, moonbase: 1648994n } },
  {
    specVersion: 1201,
    blockNumber: { moonbeam: 415946n, moonriver: 1471037n, moonbase: 1679619n },
  },
  {
    specVersion: 1300,
    blockNumber: { moonbeam: 524762n, moonriver: 1541735n, moonbase: 1761128n },
  },
  { specVersion: 1400, blockNumber: { moonbeam: null, moonriver: null, moonbase: 1962557n } },
  {
    specVersion: 1401,
    blockNumber: { moonbeam: 915320n, moonriver: 1705939n, moonbase: 1967358n },
  },
  {
    specVersion: 1502,
    blockNumber: { moonbeam: 1107285n, moonriver: 1814458n, moonbase: 2112058n },
  },
  {
    specVersion: 1503,
    blockNumber: { moonbeam: 1115896n, moonriver: 1909326n, moonbase: 2220736n },
  },
  {
    specVersion: 1504,
    blockNumber: { moonbeam: 1117310n, moonriver: 1910640n, moonbase: 2221773n },
  },
  { specVersion: 1603, blockNumber: { moonbeam: null, moonriver: null, moonbase: 2285347n } },
  { specVersion: 1605, blockNumber: { moonbeam: null, moonriver: 2077599n, moonbase: 2318567n } },
  {
    specVersion: 1606,
    blockNumber: { moonbeam: 1326697n, moonriver: 2105127n, moonbase: 2379759n },
  },
  { specVersion: 1700, blockNumber: { moonbeam: null, moonriver: null, moonbase: 2529736n } },
  {
    specVersion: 1701,
    blockNumber: { moonbeam: 1581457n, moonriver: 2281723n, moonbase: 2534200n },
  },
  { specVersion: 1702, blockNumber: { moonbeam: 1821212n, moonriver: 2524247n, moonbase: null } },
  { specVersion: 1800, blockNumber: { moonbeam: null, moonriver: null, moonbase: 2748786n } },
  { specVersion: 1801, blockNumber: { moonbeam: null, moonriver: 2572556n, moonbase: 2830542n } },
  {
    specVersion: 1802,
    blockNumber: { moonbeam: 1919458n, moonriver: 2616190n, moonbase: 2879403n },
  },
  {
    specVersion: 1803,
    blockNumber: { moonbeam: 2073477n, moonriver: 2767174n, moonbase: 3004714n },
  },
  { specVersion: 1900, blockNumber: { moonbeam: null, moonriver: null, moonbase: 3069635n } },
  {
    specVersion: 1901,
    blockNumber: { moonbeam: 2317683n, moonriver: 2911863n, moonbase: 3073562n },
  },
  {
    specVersion: 2000,
    blockNumber: { moonbeam: 2673234n, moonriver: 3202604n, moonbase: 3310369n },
  },
  {
    specVersion: 2100,
    blockNumber: { moonbeam: 3011798n, moonriver: 3588831n, moonbase: 3609708n },
  },
  {
    specVersion: 2201,
    blockNumber: { moonbeam: 3290853n, moonriver: 3858885n, moonbase: 3842850n },
  },
  { specVersion: 2301, blockNumber: { moonbeam: null, moonriver: null, moonbase: 4172407n } },
  {
    specVersion: 2302,
    blockNumber: { moonbeam: 3456477n, moonriver: 4133065n, moonbase: 4193323n },
  },
  { specVersion: 2401, blockNumber: { moonbeam: null, moonriver: 4668844n, moonbase: 4591616n } },
  { specVersion: 2402, blockNumber: { moonbeam: null, moonriver: null, moonbase: 4772817n } },
  {
    specVersion: 2403,
    blockNumber: { moonbeam: 4163078n, moonriver: 4770488n, moonbase: 4804425n },
  },
  { specVersion: 2500, blockNumber: { moonbeam: null, moonriver: 5175574n, moonbase: 5053547n } },
  {
    specVersion: 2501,
    blockNumber: { moonbeam: 4543267n, moonriver: 5211264n, moonbase: 5194594n },
  },
  { specVersion: 2601, blockNumber: { moonbeam: null, moonriver: null, moonbase: 5474345n } },
  {
    specVersion: 2602,
    blockNumber: { moonbeam: 4977160n, moonriver: 5638536n, moonbase: 5576588n },
  },
  {
    specVersion: 2700,
    blockNumber: { moonbeam: 5504531n, moonriver: 6041969n, moonbase: 5860584n },
  },
  {
    specVersion: 2801,
    blockNumber: { moonbeam: 5899847n, moonriver: 6411588n, moonbase: 6209638n },
  },
  {
    specVersion: 2901,
    blockNumber: { moonbeam: 6197065n, moonriver: 6699589n, moonbase: 6710531n },
  },
  { specVersion: 2902, blockNumber: { moonbeam: null, moonriver: null, moonbase: 6732678n } },
  { specVersion: 3000, blockNumber: { moonbeam: null, moonriver: 7043011n, moonbase: 7299818n } },
  { specVersion: 3001, blockNumber: { moonbeam: 6593037n, moonriver: null, moonbase: null } },
  { specVersion: 3100, blockNumber: { moonbeam: null, moonriver: 7829527n, moonbase: 8034666n } },
];
