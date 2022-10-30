use codec::Encode;
use safe_mix::TripletMix;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};
use std::fmt::Write;
use std::str::FromStr;

const RANDOM_MATERIAL_LEN: u32 = 81;

fn main() {
	let block_number: u32 = 3085876;
	let relay_block_number: u32 = 7452214;

	let index: usize = ((block_number - 1) % RANDOM_MATERIAL_LEN) as usize;
	let i: u32 = 0;
	let seed = relay_block_number;
	let mut first_two_bytes_of_index = &i.to_be_bytes()[..2];
	let mut first_four_bytes_of_seed = &seed.to_be_bytes()[..4];
	let mut constant_string: [u8; 6] = [b'f', b'i', b'l', b't', b'e', b'r'];
	let mut subject: [u8; 12] = [0u8; 12];
	subject[..6].copy_from_slice(&mut constant_string);
	subject[6..8].copy_from_slice(&mut first_two_bytes_of_index);
	subject[8..].copy_from_slice(&mut first_four_bytes_of_seed);

	let expect = "0x11984d5d5ac59cb747a3e76d1c11614f2cff3de1f9851b30149e12ba6673cc5a";

	// 0x9acdd8c2f25e0fe876441d86c91f133a80c8dcfbb0a3997b3a2ba50580ba23ff, (0, [102, 105, 108, 116, 101, 114, 0, 21], 0x6c6776eca9a28ae89c2ccc1a576e12d3706e909d31fdda1f5c09f22de4a3137c)
	let hash_series: Vec<H256> = vec![
		H256::from_str("5648591534a80444ebe0a96a4fc46f45a6d1b27a0e2822b96d2c53ffdca97665").unwrap(),
		H256::from_str("163e9683a11f5482c4e24fdfd7bf24627167b1bc1370e1cc8a3421601cfa39c1").unwrap(),
		H256::from_str("6f5fdb1d5ff5e8a6091d4183824796d813cd207c8891b865a482876eb8e8e694").unwrap(),
		H256::from_str("c57fa3978939b15e061271650deaeaff7cab21db7a756b7c78b549e76d96c0c6").unwrap(),
		H256::from_str("a752e1f1d3985612427835ea17513ee38acea9a49ce421af0e210daacd7ed675").unwrap(),
		H256::from_str("c4fb4ca856c5171e8ce0d2059b7775135984b635908126cc509a9b4f80fbba9d").unwrap(),
		H256::from_str("01d8b42d694a96fd3420724fc7195aad277b526d9537664ef5f463a839256784").unwrap(),
		H256::from_str("8d2d4357de7055c6b6ad217cbbbf85a9f3e507f764d5e785d6a353f774e9ed3c").unwrap(),
		H256::from_str("36de9cc7ff130b919777a9ecd91df9f94950bd832d607ee4a5fbf447e3394851").unwrap(),
		H256::from_str("2529b26c77ea3fd32ec8e2e68682d8c038e6562dff19c9bcbbf2466a2ea21b4c").unwrap(),
		H256::from_str("c3d26e6bd6ea15886bd0484f42050eaddfa448c5859da2bb703bd5327976d9fc").unwrap(),
		H256::from_str("1ca54256ae39fd43f73a6b3ac65aa67c6dbd71b99881f69d5e30abcd0e5e7446").unwrap(),
		H256::from_str("fc074f104f67f48e4305bb635c8059d0291c4734b38063bfda8c20130ba839df").unwrap(),
		H256::from_str("a013889681e38e55da7142e3c0d2985af296090b81dcd06641f20edc5cf9ff80").unwrap(),
		H256::from_str("76b93045dfec384db99f643846cafc1f68ffa71982cb41cd022037e00e13aa5f").unwrap(),
		H256::from_str("f6d7926bc339d85ba1f69d4f58fec1c45c568979aa3091d80623e093fe5eb364").unwrap(),
		H256::from_str("9c023210f197964951177ccb060c99bffb392e89a2bf892efb4df853a7a918f4").unwrap(),
		H256::from_str("ac8227ef26b6edf4b2b37b97efe585cb56b0c8f3ccb76a1e123410a63f411301").unwrap(),
		H256::from_str("742c9f268dfb8f8df26047dd7ce24150ac06a43bed60c45c30b93009fa052e71").unwrap(),
		H256::from_str("3030f5996c374e191ee794e263098a947740dbac65cc28cacba66677ce03b2ee").unwrap(),
		H256::from_str("32dc38c041704fa64f95bee03fc2470a899d9ef948f1bf351fca5ff36d3239a4").unwrap(),
		H256::from_str("a1f24f982fc8b36c4fea0093ba40848ac76e28dbcf25d1fb197fbbf34150d44a").unwrap(),
		H256::from_str("e2018301fbd33a85eecdf2f78e1c3963f951fe2fa45f1ca4884f633abc9fb17d").unwrap(),
		H256::from_str("470cb33ddd96a803d63a72ec38ee147a0647cf5d9807524a9bb0c3564799a2e2").unwrap(),
		H256::from_str("438eba13f863dc7285e94e4bc78fc2ce72d44aeaa92919332ed9aab81e86933a").unwrap(),
		H256::from_str("54223c579d4e31e568a55a17d58f65009110b3209eb523b9ef5abc31e54f134b").unwrap(),
		H256::from_str("e36b0b65356562981f1f3b5b498607200e9165e8596d8cfe68806e1971d8b4bd").unwrap(),
		H256::from_str("d9a2c9ed95b4e7cac13e8bb35224f0925f083d30da0e909849fde32d12a667f4").unwrap(),
		H256::from_str("2a4bb20ca747bfde18675ee7c6ca61e25ab912a9df3cf9ad998c1dd7073773fb").unwrap(),
		H256::from_str("9ea4a77874f33734b11395dc6942cb5de075c04bb429cf93ce9e9e53a93c6f36").unwrap(),
		H256::from_str("c8df3deec0f886f9c3972fff67ce5c9fb7a2a64d9c5e427d5c63c64721cdd324").unwrap(),
		H256::from_str("477286a3869bbd1087a0a7ecc7707bd2e719b3cad1ed69e757e619f3aed674d1").unwrap(),
		H256::from_str("b7915259604c20245bde082147e9b283e86e43feb696b21c5bf4ddd3159225b7").unwrap(),
		H256::from_str("885a6312426cf07c58c8b9738fbdcbb0e27df8790f1553c155951b54f931cc87").unwrap(),
		H256::from_str("8263f4641bdf6a351683bdec21aa49bce5e9859b19f27d483907bedfe339d3eb").unwrap(),
		H256::from_str("4582a9cf777eca3af8ad0857657fdf3dbee35df84ea8ad3449b3f70a6c28e845").unwrap(),
		H256::from_str("6177aa630d8cf226103091183f8d24faa619f38ccf3f1af98425108555c6f49a").unwrap(),
		H256::from_str("694f21dbbe5ba81262b2f51f92b1dbb59f36999073b949c5da6013c0cd94505d").unwrap(),
		H256::from_str("78b8c4cf94ee6ec3e78746597eeea08d4c2502885c65ab5747e6506f0013ecc6").unwrap(),
		H256::from_str("0e0cd374ccf55a1fbd7ce743fa3a54a7b05518c59cb3a30f4d3e121ba845cc24").unwrap(),
		H256::from_str("5777ff6794d1bad3d0fdd100310db5ee88d22d2046838e5e7d8ec52bd856f527").unwrap(),
		H256::from_str("889858f2ae24c246931a81abe26638fab7ae32a5fbd17b021131ca257e241f43").unwrap(),
		H256::from_str("f2f2aad2e8871ff7bad38d7e2a815a03e9f37e70e9f68257f44719a776a5de01").unwrap(),
		H256::from_str("8999e6c831133162b61465cdd4915314dadeb5edc77e4325934de88d390a6602").unwrap(),
		H256::from_str("8ff0bd5b5ec5d6e286dd866ad9def5167a270c42e017c333eb1d6bb910f36fdd").unwrap(),
		H256::from_str("a6d43eec530185181df6f5d239c8a67b6094c81bd88c87c3331105a10bc9238e").unwrap(),
		H256::from_str("4abca279e26bb2f8b31392fb938599860194cb57cdca652e78e47f0a5835d584").unwrap(),
		H256::from_str("e80c50b7c2168c22c57614fec981b07fb9aebd5900eaa27a017f85751566033f").unwrap(),
		H256::from_str("4826d62a8ba47f474049badf50a8511bb4015b221467e53ade187bc0b7be6040").unwrap(),
		H256::from_str("1e97b824925dba045509489d6ec1eecfef3dfe4aca47ab452564798c0a5443f6").unwrap(),
		H256::from_str("9a29c62306d64a30f342d64bed2a8bfd2ddcbf7ad1a22c738684bc8292e89025").unwrap(),
		H256::from_str("78ee93b778e38c301b88ee6aeb16765a97bcbd4ca3f382970168bc76512ca334").unwrap(),
		H256::from_str("3386f6092fc6c8e7e3af14d5a19aac351422c162df67e6bdf070888c21fa5a9a").unwrap(),
		H256::from_str("848bf63f888b7d9357792ec2db0b96fd746d298a744360658f9c9e24b05c5f97").unwrap(),
		H256::from_str("d4e88d4d96d2b5d1d4b6e7b480b8010f25b24d6d3a59629c72763977c1b1ba74").unwrap(),
		H256::from_str("17c34535f9679d3fd3a8c0199a9c513adaee59766d3c693b6a8e92b5c085e6f9").unwrap(),
		H256::from_str("602bb19434f8198c9dd970cfdc577e2c88724bcb90ac9f0ec2c145baff4a9cb1").unwrap(),
		H256::from_str("4e12edd016ff84e5afa9ab487e46745f9e34fb0e3464509a321aab462f292dab").unwrap(),
		H256::from_str("ab05ca0b71cbf00b3ab6d337287b1cd4e3397616d05de8be5602fb5bb22f3626").unwrap(),
		H256::from_str("e4262c29cb38ce1df6574a8fd64e72a180f7218a222f3b4579c92933f815d38e").unwrap(),
		H256::from_str("b73d879b0c1f95e30ac99f819411014ccec0e6cefe93d567e44268fc648ff897").unwrap(),
		H256::from_str("4465f712867142190966a7e9d8e06edf3fa90ad1c5735c4ab6bf656df4c49452").unwrap(),
		H256::from_str("0a80bac8d0c5b34bafe02bb2fcebe103cc94e4f494059e232d700cf5701b1555").unwrap(),
		H256::from_str("af8c748f19f1689d41738450af13a56bba27cd5207cfbd26cce5f0d6f2f00f2b").unwrap(),
		H256::from_str("6234006c68694ae7c5ea5147c050505ecbd9571da54646364c82c8940dfb3030").unwrap(),
		H256::from_str("f7b02db0c8c496fc3a5a0c48c1a6db07e70e76ebc748fd6eca402d8cea2add42").unwrap(),
		H256::from_str("db6524804883c19a0b80a45ad8c67edcc18c796107350763bf4b4b9c57535921").unwrap(),
		H256::from_str("5ce87c6db5b9ad91d7b47741c8e4946d8c4bbd53ad11231cca6db80ba9677bcf").unwrap(),
		H256::from_str("f03ad13a7e3e59f743979843ab5d010f126bdef2df9f5aab6f3c4acda8e86ea3").unwrap(),
		H256::from_str("587e7fd8b99bf7340171ea28950375b0843c829789dce43ba5736a6519da8cda").unwrap(),
		H256::from_str("58746b339543014ce811e9da6f5b4ea08bb6c49153c03025e96f2a27c61b3e2b").unwrap(),
		H256::from_str("036f2cc3df395cc83d1a25b3ddd462aa187d08a96a50c1de97250c14d259f569").unwrap(),
		H256::from_str("5801f53a5f5792474de0c026c559b3f480c6e6f225bfc4de9399bd0868ba86c1").unwrap(),
		H256::from_str("cd53c084605187e4bb2ffe8f0dcf865bfc2721d9ea8735f3d02263fb8732e570").unwrap(),
		H256::from_str("169dca460ba70c034ec85ce35de36a7a65514a70bbee5af4f95186230fe96e2b").unwrap(),
		H256::from_str("4b89e52de93e18ef36b48b8a146cb8a080cfc053bcaa077d18bb7dae1dd79c76").unwrap(),
		H256::from_str("4bb7779a0338506ed29843ebee17a86abca4c686a47f6eca81592d7fd8d30562").unwrap(),
		H256::from_str("b79a74e078fe7291ca7d6c4e374edc7b1b65d8fa4792e3718c84fbc8ed92d9a9").unwrap(),
		H256::from_str("08fe4abe31674525da96dba020f8baa47975677ed7eb094cdd75b730fbe1df0d").unwrap(),
		H256::from_str("3fb8b53fb61227fc53ace3397aa6c317c7424a37696dee384841319de2c8133d").unwrap(),
		H256::from_str("164a35ff25faa25bf6554c6115dd1106da05a20f4edb9eb26a6e4a1c2385d970").unwrap(),
	];

	println!(
		"{}",
		(16 as i8, 32 as i8, subject, hash_series[15]).using_encoded(|e| format!("{:?}", e))
	);

	let seed = if !hash_series.is_empty() {
		println!(
			"hash_series {:?}",
			hash_series
				.iter()
				.enumerate()
				.map(|(i, h)| (i as i8, 32 as i8, subject, h)
					.using_encoded(sp_io::hashing::blake2_256)
					.into())
				.collect::<Vec<H256>>()
		);
		println!(
			"index {}, RANDOM_MATERIAL_LEN: {}, subject: {:?}",
			index, RANDOM_MATERIAL_LEN, subject
		);
		let series = match hash_series
			.iter()
			.cycle()
			.skip(index)
			.take(RANDOM_MATERIAL_LEN as usize)
			.enumerate()
			.fold(None, |a: Option<String>, (i, h)| {
				Some(match a {
					None => format!(
						"  {:?}, {:?}",
						H256::from(
							(i as i8, 32 as i8, subject, h)
								.using_encoded(sp_io::hashing::blake2_256)
						),
						(i as i8, subject, h)
					),
					Some(mut a) => {
						write!(
							a,
							"\n  {:?}, {:?}",
							H256::from(
								(i as i8, 32 as i8, subject, h)
									.using_encoded(sp_io::hashing::blake2_256)
							),
							(i as i8, subject, h)
						)
						.unwrap();
						a
					}
				})
			}) {
			None => String::new(),
			Some(rval) => rval,
		};
		println!("{}", series);
		println!(
			"===   Result: {:?}",
			hash_series
				.iter()
				.cycle()
				.skip(index)
				.take(RANDOM_MATERIAL_LEN as usize)
				.enumerate()
				.map(|(i, h)| H256::from(
					(i as i8, (subject.len() * 4) as i8, subject, h)
						.using_encoded(sp_io::hashing::blake2_256)
				))
				.triplet_mix()
		);
		// Always the case after block 1 is initialized.
		hash_series
			.iter()
			.cycle()
			.skip(index)
			.take(RANDOM_MATERIAL_LEN as usize)
			.enumerate()
			.map(|(i, h)| {
				(i as i8, (subject.len() * 4) as i8, subject, h)
					.using_encoded(sp_io::hashing::blake2_256)
					.into()
			})
			.triplet_mix()
	} else {
		H256::default()
	};

	println!("=== Expected: {}", expect);
	// (seed, block_number.saturating_sub(RANDOM_MATERIAL_LEN.into()))
}
