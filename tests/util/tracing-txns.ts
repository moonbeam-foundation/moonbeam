// Hardcoded static data used for verifying Tracing works on for DEV tests

export interface NetworkTestArtifact {
  chainId: number;
  testData: RuntimeTestArtifact[];
}

export interface RuntimeTestArtifact {
  runtime: number;
  blockNumber: number;
  txHash: string;
}

export const tracingTxns: NetworkTestArtifact[] = [
  {
    chainId: 1284, // Moonbeam
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
    chainId: 1285, // Moonriver
    testData: [
      {
        runtime: 400,
        blockNumber: 424998,
        txHash: "0x5eaa6c3e8186f98b3d8a1f61d94430af671251e40fb63bf62eb96c48ec8ef773",
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
        blockNumber: 585000,
        txHash: "0x2c88c8c2d3c476fb8e001264d47c0d3266b038dc1b08fd24678ca284e5ef68a9",
      },
      {
        runtime: 800,
        blockNumber: 685000,
        txHash: "0xfb9bbe158e03a9d329216ee8a9bcee59f1e73dc2a4524f4d53755a944a0bab6e",
      },
      {
        runtime: 900,
        blockNumber: 925000,
        txHash: "0x8c8a8d0a227409568fbf8e36db9ed29f3af0c7e2bff7e6c5c50a77e3412ad5c0",
      },
      {
        runtime: 1001,
        blockNumber: 1055000,
        txHash: "0x1c93a11b3aa6fc75cd256029c9e52fe65b0c1176b6d49622b45a91d480f25d5f",
      },
      {
        runtime: 1002,
        blockNumber: 1145000,
        txHash: "0xba1c1918ab2dbc4498287810b49081c98bc9fd1a600f1ebbbc0117469e717ccc",
      },
      {
        runtime: 1101,
        blockNumber: 1190000,
        txHash: "0x586143cace437c68ed090cbe625964393fb8d7eec4c4dc1e6cd5b0e131d621a3",
      },
      {
        runtime: 1102,
        blockNumber: 1300000,
        txHash: "0x0b0d1c5c79e216dcf83af98d16d9560c16d197ec976fe436c9d39259a5e92033",
      },
      {
        runtime: 1103,
        blockNumber: 1458102,
        txHash: "0xfa3ded338a953dd52d5494817ccbc8459bac251376f8300fbdef27e4c945a97d",
      },
      {
        runtime: 1201,
        blockNumber: 1517390,
        txHash: "0x932b4d9db194acc121ef35061f8f36f263c5e805efc53a7c4f31e7999c5133fa",
      },
      {
        runtime: 1300,
        blockNumber: 1621145,
        txHash: "0xddf073e07cdf3573f7c4653997e9984c9ea72c97522867063999e9d7230dcf91",
      },
      {
        runtime: 1401,
        blockNumber: 1776778,
        txHash: "0x0dcf365809fa210a6fac285c4657df134b5d66e1a3ee00045e13f093f6b4ba97",
      },
      {
        runtime: 1502,
        blockNumber: 1854594,
        txHash: "0x41641b1e5d2f7039559cddb02f8c4822d2d99b347536d940108c00b0a1de70e9",
      },
      {
        runtime: 1503,
        blockNumber: 1910524,
        txHash: "0x0c2e7571852a484ecaa800e75d23bd39f70acdaa12b4bba736be369d5c69ec53",
      },
      {
        runtime: 1504,
        blockNumber: 1921467,
        txHash: "0x56efe78fcdfcc0052d04fad5be8277347346e3e54169f956c1eea32824496e8b",
      },
      {
        runtime: 1605,
        blockNumber: 2082923,
        txHash: "0x6c1ca793ce298dea497943c6e093efc1e903f0c84e48561a502df4767e02a862",
      },
      {
        runtime: 1606,
        blockNumber: 2105167,
        txHash: "0xe4d0b82f008a7245d880351eaf718a27e64d8d686357d8a2f9bd514a19c4b69f",
      },
      {
        runtime: 1701,
        blockNumber: 2425099,
        txHash: "0x42e556ce2532b78c7be2262839ad8c0561705aea7dff9e3b76fd36a893ab95d6",
      },
      {
        runtime: 1702,
        blockNumber: 2545073,
        txHash: "0x8b6c7c7f7c02713028f394fbe7465e35f230952915b083e9d95ee37b66cc248a",
      },
      {
        runtime: 1801,
        blockNumber: 2595062,
        txHash: "0xd6bb6447f1bd9628d17dc59c4ddc9e776e099866a0d735765f32bba97507d6ca",
      },
      {
        runtime: 1802,
        blockNumber: 2670047,
        txHash: "0x5d8e54b355274d8aec817f251a0944af7b12f0409f893f6964ff5536831af46b",
      },
    ],
  },
  {
    chainId: 1287, // Moonbase
    testData: [
      {
        runtime: 400,
        blockNumber: 615001,
        txHash: "0x6d3be15e41c663a12f1b9886cfff084f5be9267055a55fc3a218f4a2bf9d07e4",
      },
      {
        runtime: 501,
        blockNumber: 655000,
        txHash: "0xec8d43aec5e0fcb4435bccadbf7f280524c4432ed5c33bd3782d2cc8da434a2d",
      },
      {
        runtime: 600,
        blockNumber: 680000,
        txHash: "0xb251488483b7ded41d639b09c17f8349357950a0d29088c53d2907ea5b4f881e",
      },
      {
        runtime: 701,
        blockNumber: 800000,
        txHash: "0x5d5910b986fc98a8333bca547d5eed274f1019a945a62838b5944b7f14fd7c9b",
      },
      {
        runtime: 800,
        blockNumber: 920000,
        txHash: "0x431592ad25db7a008c7c1762ee8f58afa04e3889805f07d43f41a310c30c8c83",
      },
      {
        runtime: 900,
        blockNumber: 1080000,
        txHash: "0x4c850e17c0046b2a8b7c00a9e6c9ce4f6c06766d2e98c2bafff724086a440d55",
      },
      {
        runtime: 901,
        blockNumber: 1135000,
        txHash: "0xf22c5c6f4b9c35d44158eb07664a36527c65dfa4a1d751cce1a69de900194028",
      },
      {
        runtime: 902,
        blockNumber: 1180000,
        txHash: "0xa148ed971adf65ee2d03e2cf372eaf94fc17fa53094856761b3e891fc745cb6c",
      },
      {
        runtime: 1001,
        blockNumber: 1290000,
        txHash: "0x6bd07a2a1394b049d6aa3cdb3c767a29b342d528bda27731e65a41d149a4ffb1",
      },
      {
        runtime: 1002,
        blockNumber: 1400000,
        txHash: "0xfcbcbad9a2e7fe22dcbf272a59873fed5098dcfb1fbb136412e5c536ca365c38",
      },
      {
        runtime: 1101,
        blockNumber: 1430000,
        txHash: "0x18d048c2c9dd83534f0a0ed2f873ec0ca98cb14a50bcf67d286cd9d104e54d7d",
      },
      {
        runtime: 1102,
        blockNumber: 1520000,
        txHash: "0xdfc09fed088c522a5d3f07cbe915d5dce666b8af98ea9baa3a9be9475d0bdea5",
      },
      {
        runtime: 1103,
        blockNumber: 1609968,
        txHash: "0xb04c978b4556cfeaaf15c7f4ecdd0dcd3f4c3f5b20912c174f92342760375764",
      },
      {
        runtime: 1200,
        blockNumber: 1649329,
        txHash: "0x955c3dfce09552dcfd7cbe3bd95619b98e2ea57206cda2cc5d8c75ae2d7aa7a1",
      },
      {
        runtime: 1201,
        blockNumber: 1723131,
        txHash: "0x6570d66251531f1a32fe8c73fc6c9ed56da29f82027d4bec07ca16428e3412c2",
      },
      {
        runtime: 1300,
        blockNumber: 1852285,
        txHash: "0x780a476b77daeae57e354c41d6da2b87bc7f25ced686d8dda48433627bb07f9f",
      },
      {
        runtime: 1400,
        blockNumber: 1967313,
        txHash: "0x0ea32bae1ea9f74c3186eeb709f0eb77fbdd26a36c39a4643e1e809c9d131422",
      },
      {
        runtime: 1401,
        blockNumber: 2103531,
        txHash: "0x2468a777d8b5bf935798a3f4b1c2a066d32588e49e8552765c373be77bbdd1ba",
      },
      {
        runtime: 1502,
        blockNumber: 2171640,
        txHash: "0x113d5c875111fb3fa16de27a731cc247375f2becfe5f9dabfa8b5d792d026380",
      },
      {
        runtime: 1503,
        blockNumber: 2221656,
        txHash: "0x47b8bb8f17f05eabc851a9edde292e76bc13408d82b2a6d797ca6c7e26e165b5",
      },
      {
        runtime: 1504,
        blockNumber: 2230702,
        txHash: "0x52b89d3a198c73df3ac977483a9fa538ac59f06b1fbd7ec8121b4ca627629533",
      },
      {
        runtime: 1603,
        blockNumber: 2290323,
        txHash: "0xfccbcecc811850924b7c3d115db8f1727467ae43dcccc7fe8ae3bd469b18520c",
      },
      {
        runtime: 1606,
        blockNumber: 2379851,
        txHash: "0x5ec239e623e792b65cb0aac22c2e852d689b60a7b8e9a4707d07dd54514e4c59",
      },
      {
        runtime: 1701,
        blockNumber: 2534246,
        txHash: "0xe11a3c209c2b4c74fa759c376484a7c29a037e4102420f123d6463d37ad72fb3",
      },
      {
        runtime: 1800,
        blockNumber: 2760849,
        txHash: "0x0fa2f70d9ca75a106ee2c4d4ce40d9c26b6e9993eabd059149f61a1faaff8bae",
      },
      {
        runtime: 1801,
        blockNumber: 2874151,
        txHash: "0x4a7703494b76e7eb161ec908ae722f9c19975d154c9efdb9d1af01fc321b1705",
      },
      {
        runtime: 1802,
        blockNumber: 2930802,
        txHash: "0x399f67c8101640d8ff7e873f3652e1c86d734db40c347dddad41a4a4535dbcc7",
      },
      {
        runtime: 1900,
        blockNumber: 3071020,
        txHash: "0x8bb752c82077b17bfd52118b6ee9ea69e9b1809481530d7ef7e836197138f4dd",
      },
    ],
  },
];