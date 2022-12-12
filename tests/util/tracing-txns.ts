// Hardcoded static data used for verifying Tracing works on fully synced tracing enabled nodes

export interface NetworkTestArtifact {
  chainId: number;
  networkLabel: String;
  WETH: String;
  testData: RuntimeTestArtifact[];
}

export interface RuntimeTestArtifact {
  runtime: number;
  blockNumber: number;
  txHash: string;
}

export const tracingTxns: NetworkTestArtifact[] = [
  {
    chainId: 1284,
    networkLabel: "Moonbeam",
    WETH: "0xAcc15dC74880C9944775448304B263D191c6077F",
    testData: [
      {
        runtime: 1101,
        blockNumber: 172073,
        txHash: "0x1f7584f08511803675e0bc514ef33117c1f4b8fcf87ed543889ee10ddc946c22",
      },
      {
        runtime: 1102,
        blockNumber: 215000,
        txHash: "0x1a316fc3a61752860489692f6484463ec569a5dce90044aaec73c4b458e22271",
      },
      {
        runtime: 1103,
        blockNumber: 330746,
        txHash: "0x259b05d9e2b5dc1c470258585f31e6a2b26bb6038f583f935492e065809c347f",
      },
      {
        runtime: 1201,
        blockNumber: 432029,
        txHash: "0x96befb68bcd3ef0cfbe86d50c6b905cea958077616a8b182b1f046ed30643472",
      },
      {
        runtime: 1300,
        blockNumber: 786498,
        txHash: "0xc1000fba44f8339b2160f6db948123676af31b08bc23329e9294998be0e70680",
      },
      {
        runtime: 1401,
        blockNumber: 963735,
        txHash: "0xfe9a902ddf0cbb9a2a6e283ea8dfcf7823de666c11639dd7252b9c70741b9a5f",
      },
      {
        runtime: 1502,
        blockNumber: 1107739,
        txHash: "0x17f88a61d4741eb8e96565d423d4f1119bbe2b370a4a10a6393064c355add40f",
      },
      {
        runtime: 1503,
        blockNumber: 1116052,
        txHash: "0x213e5afe9e712746081a35f73265233730f1383a24ad9acdbabc413f4988ff9a",
      },
      {
        runtime: 1504,
        blockNumber: 1128509,
        txHash: "0x0e859d43e2d567fa80059559e44adb81407aa8b46642153b81bb5e8c3a5fea20",
      },
      {
        runtime: 1606,
        blockNumber: 1326723,
        txHash: "0x2f6e7a3db8eee993b28138ec71f049ffac001e354d5b994c7c5ae51948762eab",
      },
      {
        runtime: 1701,
        blockNumber: 1688151,
        txHash: "0x007adce604dd483a0ab829fc4457160d92ca0585a332e58f9386064f124197b4",
      },
      {
        runtime: 1702,
        blockNumber: 1868857,
        txHash: "0x6bbf1d329ec28ec4eae31f290418b0542d0a19f25153aacc15be6c979c34e576",
      },
      {
        runtime: 1802,
        blockNumber: 1959212,
        txHash: "0x7723b80847b4c2e65e57782e9010360a92921f872d6fb5e76dcd6a8d3fa49d5c",
      },
    ],
  },
  {
    chainId: 1285,
    networkLabel: "Moonriver",
    WETH: "0xf50225a84382c74CbdeA10b0c176f71fc3DE0C4d",
    testData: [
      {
        runtime: 400,
        blockNumber: 425002,
        txHash: "0x000e9d72bad244ab6c27d3166bbcf182df0e40049bf3d6d945a61f68f36e6038",
      },
      {
        runtime: 501,
        blockNumber: 435000,
        txHash: "0x104d21f728b95f36d81c66f12b238b73b66c02b9c8389b77bb44e69f7fb670f3",
      },
      {
        runtime: 600,
        blockNumber: 460000,
        txHash: "0xd507f7c89e93013b4d4843bdd398e0900c8575a1e1190d8d2dad0df7fef7c9f3",
      },
      {
        runtime: 701,
        blockNumber: 585001,
        txHash: "0x28855e4fb4c10663f9a30e978777d2e4274f72a52cd3b016d0ec96a0f4152721",
      },
      {
        runtime: 800,
        blockNumber: 685000,
        txHash: "0xa23e61c53b653817ac84710bf8ad7e2ae0260230e130efb3cd8c3891b57550b9",
      },
      {
        runtime: 900,
        blockNumber: 925000,
        txHash: "0x740f7eea8f22c6f8cc09fc4d0d6f9897f20b212dfb5278f0c1cb768ebe924152",
      },
      {
        runtime: 1001,
        blockNumber: 1055001,
        txHash: "0x8695e9b222e2a8ca87d4f7b520e887cea61309aebe443e519ec92c54b7021059",
      },
      {
        runtime: 1002,
        blockNumber: 1145000,
        txHash: "0xdca6cbd70c8ae57f883947e02985c45f3c8729ba278af4dffd6725db2663a207",
      },
      {
        runtime: 1101,
        blockNumber: 1190000,
        txHash: "0x64a39c009f66bb48cb7b36e799c3e2ccf9793b40c7a255ad2a99e81054d8c020",
      },
      {
        runtime: 1102,
        blockNumber: 1300001,
        txHash: "0x2c8ee8f52e669b4def18e5d20a2d2a62b7c12549b8f1dfcb409d81ebb8cd6369",
      },
      {
        runtime: 1103,
        blockNumber: 1458104,
        txHash: "0x7e9ae6f800240a03b6defe949111de6faa0e95eacd668825f87cda4866e7190c",
      },
      {
        runtime: 1201,
        blockNumber: 1517390,
        txHash: "0x932b4d9db194acc121ef35061f8f36f263c5e805efc53a7c4f31e7999c5133fa",
      },
      {
        runtime: 1300,
        blockNumber: 1621145,
        txHash: "0xa4c29973eba4202a921dc08219b04bc5cdebba38cad7f6bf3aa3fd11fe8636f9",
      },
      {
        runtime: 1401,
        blockNumber: 1776780,
        txHash: "0x62d0aafb400305a620ca0981308ec158bb31e26932d7656beec5c0bdf084139f",
      },
      {
        runtime: 1502,
        blockNumber: 1854594,
        txHash: "0x41641b1e5d2f7039559cddb02f8c4822d2d99b347536d940108c00b0a1de70e9",
      },
      {
        runtime: 1503,
        blockNumber: 1910525,
        txHash: "0x244048753c0f794d3a4ed63802e0f2b68245bd4bdaaae782e9437ac2a46fcc81",
      },
      {
        runtime: 1504,
        blockNumber: 1921473,
        txHash: "0x1b8a2b9137f8b85b269b331a41f4b8f40ab51d8eb39d6ef6680b64233ca5f6cc",
      },
      {
        runtime: 1605,
        blockNumber: 2082924,
        txHash: "0x191e42667ad1a9fdcf95b75610701a3da5dff11d90b6145b1b2036f526be8416",
      },
      {
        runtime: 1606,
        blockNumber: 2105167,
        txHash: "0x24a6c89e71c5a905471cd7cd468ef162710197cb4affbdcba9fd274b5c957ea4",
      },
      {
        runtime: 1701,
        blockNumber: 2425113,
        txHash: "0xba1253e609697fcb50a98d9e768b41a7aa9beec0a8f026b166e3f8753cdbcf2d",
      },
      {
        runtime: 1702,
        blockNumber: 2545083,
        txHash: "0xffebd87d35a25b0b59608432095fca2167f56afccd4d73155d45c66247fe6912",
      },
      {
        runtime: 1801,
        blockNumber: 2595064,
        txHash: "0x370d8462f807ac36b656252a08f473073a699b014a6b3ee73ff801b3a860c10a",
      },
      {
        runtime: 1802,
        blockNumber: 2670050,
        txHash: "0xa7dc73c353f1d2a4f13b9294a279c5acc95c027dc82dd1b299326fae44c0b400",
      },
      {
        runtime: 1803,
        blockNumber: 2073478,
        txHash: "0x3f37192ee903acb033b83c34fdc96af2e40190bf31f4ad6fdd3b1dee7cd43cfc",
      },
      {
        runtime: 1901,
        blockNumber: 2951940,
        txHash: "0xfbc8098b67194f70be7ff8fbfe2bb0d72ddf1fb7ee6117b168d34ff3f49df429",
      },
    ],
  },
  {
    chainId: 1287,
    networkLabel: "Moonbase Alpha",
    WETH: "0xD909178CC99d318e4D46e7E66a972955859670E1",
    testData: [
      {
        runtime: 400,
        blockNumber: 615005,
        txHash: "0x05e666de74852a121144d5fc4e299c2d9168b55323d77bb05e4a527b094e2333",
      },
      {
        runtime: 501,
        blockNumber: 655002,
        txHash: "0xab00fc7c874dec858eb9948ec8ae2c0edf05391f2c79e3c2c0011aafb3909ae9",
      },
      {
        runtime: 600,
        blockNumber: 680000,
        txHash: "0xa4f69a0344da82bcd8da22f6b3bfa9ade4fd7d942d61f0e5990f3c2b319657c3",
      },
      {
        runtime: 701,
        blockNumber: 800000,
        txHash: "0xe07cdcd194c88850cc76e7d371588735ed4813f6d47dd575985c27e93e136a00",
      },
      {
        runtime: 800,
        blockNumber: 920000,
        txHash: "0xe53882a3eed1369a5ce90c98dce7183f27bfe2bd6680aac9578650e8c930cbb1",
      },
      {
        runtime: 900,
        blockNumber: 1080001,
        txHash: "0x9036ff517ad9ea04f3aef43e006f906c741c0d97ec7d9225f610cb2f1b485cc5",
      },
      {
        runtime: 901,
        blockNumber: 1135003,
        txHash: "0x4d2a09ef083fa842513d1ad319173db1473eb12281be49aa5ffce8e4656472ad",
      },
      {
        runtime: 902,
        blockNumber: 1180000,
        txHash: "0x2b95b75ea6fd3054708303f75b8850e3d714d20b832790a7f3195ebe6087bd72",
      },
      {
        runtime: 1001,
        blockNumber: 1290000,
        txHash: "0xccca7c000140d33c4e8f5d80c21994066442a3049e9402d2de1bbb4681aa1869",
      },
      {
        runtime: 1002,
        blockNumber: 1400000,
        txHash: "0x55b1feb92371660c48a0cefec523f8f7af5cc60cd10158e6a2dc12ed4be74ee2",
      },
      {
        runtime: 1101,
        blockNumber: 1430000,
        txHash: "0x86c123dbc1ca1a997ed44d8bfd8ef17cb355353424af94bc368af7e762c83772",
      },
      {
        runtime: 1102,
        blockNumber: 1520000,
        txHash: "0x749056a49ecfa5a7fa561206dbe0abf4565756ae5d214f4bfd599f718a9e6e61",
      },
      {
        runtime: 1103,
        blockNumber: 1609968,
        txHash: "0x3dee5079f37a28a0b116e06676db6125fc52788774a5c9ee5b9d70b8b3294814",
      },
      {
        runtime: 1200,
        blockNumber: 1649329,
        txHash: "0x48530c0544fa4a7fbfddf6980dde8d7fc1294455e3b2a2118c732136b798aa72",
      },
      {
        runtime: 1201,
        blockNumber: 1723131,
        txHash: "0x2c200474280274fa030fa64a4c5fbb7b7d6273476793e8e97f37e2e6522a26ff",
      },
      {
        runtime: 1300,
        blockNumber: 1852285,
        txHash: "0xcaca48e2fb54d00401f6291668e4035331c896317011bf9eb658611f52eb58d6",
      },
      {
        runtime: 1400,
        blockNumber: 1967313,
        txHash: "0x09a80a8072e671b29c1c19cd1abebc2a1eef7b7994e91e949b1d86ea0c28288a",
      },
      {
        runtime: 1401,
        blockNumber: 2103531,
        txHash: "0x82afd4d6c9cc2c0d7853c21fcec22b5f1ac19bccd87e17caa5487b74e3b3d364",
      },
      {
        runtime: 1502,
        blockNumber: 2171640,
        txHash: "0x1b582ea6fa157bf0e350be0c9e602d71168aed8eceaf74057a7c1a7546f57e5e",
      },
      {
        runtime: 1503,
        blockNumber: 2221657,
        txHash: "0x598abdfc983d5a3f538b01c762bd6cf84d4651b90c2d198a625cea874b73ffb6",
      },
      {
        runtime: 1504,
        blockNumber: 2230705,
        txHash: "0x25f98315de8e6d742b208cdefacff886af0fd22f05c91ef9b34e3b93975acfa7",
      },
      {
        runtime: 1603,
        blockNumber: 2290323,
        txHash: "0xc4353bf3e690b7f47b15f2b24b6b144cb12d7ae034b84f40e042c3908b292ccf",
      },
      {
        runtime: 1606,
        blockNumber: 2379851,
        txHash: "0x5ec239e623e792b65cb0aac22c2e852d689b60a7b8e9a4707d07dd54514e4c59",
      },
      {
        runtime: 1701,
        blockNumber: 2534246,
        txHash: "0x4b4068169501319795aa08b0315a2d43a6904f5c794bc72c39654c9525cf2f19",
      },
      {
        runtime: 1800,
        blockNumber: 2760849,
        txHash: "0x44c5dbb7ae703b63f635a7af6d3ac0b17fdaba6c5c1454027bde3454285b4cf3",
      },
      {
        runtime: 1801,
        blockNumber: 2874151,
        txHash: "0xbfa459831ec5c369fe3d217c3a01288a14492399aafdca59d4e8b68ecab4aa21",
      },
      {
        runtime: 1802,
        blockNumber: 2930802,
        txHash: "0x33e4492d14dceeee484ab37a0bf4da6c961cb9e4c64d285297b4058ef9a7ea5e",
      },
      {
        runtime: 1900,
        blockNumber: 3071020,
        txHash: "0x8e3f85d7c6a8630a7f1e1eec4d57b53b23b67dc083311e28d2802f882b885d43",
      },
      {
        runtime: 1901,
        blockNumber: 3147113,
        txHash: "0xbea07c678d9d995f80126ac1bbea12c434c386742160cae66449da99ca24e19e",
      },
    ],
  },
];
