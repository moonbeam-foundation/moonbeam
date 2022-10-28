
use safe_mix::TripletMix;
use sp_core::{H256};
use sp_runtime::traits::{BlakeTwo256, Hash};
use codec::Encode;
use std::str::FromStr;
use std::fmt::Write;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
	#[arg(long, default_value_t = 1)]
	block: u32, 
}

const RANDOM_MATERIAL_LEN: u32 = 81;

fn compute(args: &Args) {
	// index should be: block number - 1 % index
    let index = ((args.block - 1) % RANDOM_MATERIAL_LEN) as usize;
    let subject:[u8; 8] = [102, 105, 108, 116, 101, 114, 0, 146];

    let expect = "0x5f363262c183e1fd09886432b05255b935f8cd4399b13a562f8e238177e2f88a";
    // 0x9acdd8c2f25e0fe876441d86c91f133a80c8dcfbb0a3997b3a2ba50580ba23ff, (0, [102, 105, 108, 116, 101, 114, 0, 21], 0x6c6776eca9a28ae89c2ccc1a576e12d3706e909d31fdda1f5c09f22de4a3137c)
    let hash_series: Vec<H256> = vec![
	H256::from_str("6feaee6e3762885495213a21cc814b3b25119e765d95fa634189b5bf5df04968").unwrap(),
    H256::from_str("9598326651ed57a7773b3803e80399f964f9f510fc1b6f07e5a693aa169f7792").unwrap(),
    H256::from_str("c52827a52d0ed88cff1d019443b0e4795cef28b1c9796d17483734b10f37d905").unwrap(),
    H256::from_str("6b4633d5fe70e565aaf4dc558bb24d2d909e82dab6b05487d92ee73ecb348987").unwrap(),
    H256::from_str("8f9455e2d9f422d6387a6d29939c824ba92b113772bd9ce9b977409573b9a015").unwrap(),
    H256::from_str("ac9c66050ffb51c4dd21e92fede1d698556652ea920098f32a503ccc69410aae").unwrap(),
    H256::from_str("c94036441c42003107b546fb1d405354d415ef72ce66ea0b4ffc9cefb408fa4e").unwrap(),
    H256::from_str("f7c336d7de9c19562419f311f041b139e138db6273efd4a31b659aa614d2740d").unwrap(),
    H256::from_str("12d707e610015d95ab89f59420c05b1109af52758ad9a5c1f147928d4e8d07b4").unwrap(),
    H256::from_str("8e119cc30e5c12e5cc4b01876b8b5a2654bd304cfb4900932cf1c664586417d9").unwrap(),
    H256::from_str("8a3632d7619893319b7616e301277fbd1a072d4ea6c519cb5809218a87febad2").unwrap(),
    H256::from_str("e4aef6f001e1378369d8f6d304ca61630f4b0321865a2eba5a69c54542315832").unwrap(),
    H256::from_str("30451c44a5acece3f0437722e251415fdb4cf33281d75dd3b403923061716db2").unwrap(),
    H256::from_str("8f8610065011f6d935e38e04ef285881ef4b34d40f698d021123d0fab8346af5").unwrap(),
    H256::from_str("616cc32b7517f44762959397916b5a6e475edc5c062d7aacbfdfd98416b84171").unwrap(),
    H256::from_str("68a7447937d2c416a532a4463c553beefbd87791704d1913eb860c73dcf03993").unwrap(),
    H256::from_str("1bbc7d52349662478f0b2abd03e078949024b903bae84715d87422115a5697b2").unwrap(),
    H256::from_str("f6ca936b7c5cd5ae2e72af2285685e1506e0946f5f5f1840d19e1f55e80b2402").unwrap(),
    H256::from_str("9aa371e07bafae7fbd690d47666c6d77c35e9cfa8487255914b4e28c9b943dbd").unwrap(),
    H256::from_str("4b0fcf79cbc9c7d58b48eb0293cb5f44e060b31886a7ffd9bb79c7e93bdf3947").unwrap(),
    H256::from_str("78932ac4e7828d14522ac77accebbab442d6fa9c679001452774fdeb9c76e3e9").unwrap(),
    H256::from_str("a1b07f9233576546bf41fac4e36f9cea358d3be59618772652545bc68f5de915").unwrap(),
    H256::from_str("9ad25a551e62eea440f1984c181ae39eeca442b5868b9e11446a6f4d1776a26e").unwrap(),
    H256::from_str("ccf8c18ca01b35a95e491b30f538df5a3c5c9ce348b2a980d9cab032785302cd").unwrap(),
    H256::from_str("be27413a2183f5905badeb6a510f11f063b8e156a431288496d68d980af8b5a3").unwrap(),
    H256::from_str("f475d31a7f33ec29e74c32459565aa5094e628f7534adc8fb4c4d894a715a9ef").unwrap(),
    H256::from_str("124664bc1e53759810d51b031aa68f729200bd56aebba251b7580d27e130a05c").unwrap(),
    H256::from_str("d58a1d12f2d943f2982d08d69123c201c30045d9bf36a8ef024a40823015ecd6").unwrap(),
    H256::from_str("643b1b3e657a3eb3bdb5e3307221a2e260f748b155eeb9179188d1f103350837").unwrap(),
    H256::from_str("b89d68ab2ad03f77f8936d5418481b19af4babbd77eb9fa15834d1a4e0da3c73").unwrap(),
    H256::from_str("111680878e4830e63b7a0baa8136e38aa6213b2db86d17c9be67beb9d7fd27cb").unwrap(),
    H256::from_str("43a46fb831077c8b953aef073b4098c4438e8ae319654c34580340434ba1f9b6").unwrap(),
    H256::from_str("b04564da75c571eb79f097681d8f4d2c63c45c67c18dd128c184bcccd5acbef0").unwrap(),
    H256::from_str("40f37d650002545325c44df03c0c7ddb0dc496918fada08333ee2f5580b512c2").unwrap(),
    H256::from_str("08c57025fdd90ed0c86f2abd7bd60b8f192eb025ad56fda3377cc20d276b72e7").unwrap(),
    H256::from_str("2ff11cd3d3a9ac17b88cb5a76e5769cbbeec85fa9892d07b51700c1c3a581e9f").unwrap(),
    H256::from_str("4c50c67b0e12225cbfc39c07860c77934e62d099ae2bff4aa0d8593d9fc706e9").unwrap(),
    H256::from_str("04a82c6b4bb59d07e9ad391f364413bab3fa3a2043a517c873e6e912a72bccef").unwrap(),
    H256::from_str("90a9b872c72ea3273dfdb729aa4605cee303a59fff89b24e6459ed67b6fba5a7").unwrap(),
    H256::from_str("9821437e44f7031aae06160ee2720a700e4d9c9b933961abceb33ee1ea167a95").unwrap(),
    H256::from_str("5c29d916d98e1e4338b598f443061c1c1f3899602d0a7540b1c6951b7afa14e7").unwrap(),
    H256::from_str("bb8daeb39b7b57ac0390b0eaa1ec33442c0fc2ba95210378ed1f1eafd7c2b476").unwrap(),
    H256::from_str("dad57d0866a4faddc3fded42b321ea60417ea8fab523ab4aee6c1223fd2bead9").unwrap(),
    H256::from_str("8ca305bd210f6e4b58dfd0b6b66d55f3dc60e95abd1d079ec0ee4556b0c1eab4").unwrap(),
    H256::from_str("c578ac42446a8ec4c96c0aec0e4a75e6fd91e2ebb2c8b3b30df093f72327abed").unwrap(),
    H256::from_str("b991d17037b45259d54ad54248aa8e38f66673c15d59a184dd3aef1db7d5dda8").unwrap(),
    H256::from_str("6b9ed6ddf4e1233666593f686c21b4adcef200053b845196fc4a3bdcf3920849").unwrap(),
    H256::from_str("07f7fe04e9a283d48041e3efd77dc4b9735a78344d6381a184cd49f763ea48e8").unwrap(),
    H256::from_str("842c3ebfd5e1f657a5eb2a7bebf9fca83bc364c63d26eeee2302ece709ae9170").unwrap(),
    H256::from_str("37ac10030d068dc54751d4c10a7b9c227459e78f776c9bfd98e83252b1bdd011").unwrap(),
    H256::from_str("733effe8889f3f9c315f56b7755a82ff252aabd6f46cdcdb08153dfc22ab8b5b").unwrap(),
    H256::from_str("6dfd05e655f7b06b3e6ec287f2b729fbe03de49e0e63cdb2984cd864fb4a4cc4").unwrap(),
    H256::from_str("657d06131ff530f58b509220d41fd559d141bbb84ba103ed2a2b210a15d333ed").unwrap(),
    H256::from_str("12043157f7c64e3fe61a492126f5d8e46b99e81094052d1a40432a0ccbed8fda").unwrap(),
    H256::from_str("8c25d39572e8f282dd11fbf2a1928b5fc1a75c45b067dbdbf43c134122bb9491").unwrap(),
    H256::from_str("84b67803711fccc5a28554a427cf107eb54ef90cd8f65201b390e6f6894f2c67").unwrap(),
    H256::from_str("4fe9e411b11626c5c74ffba174b1f5e8ba0c0d231866f1c8e79b86df403da8ab").unwrap(),
    H256::from_str("fe1c482beca7f3ce49e8187295450af59cfc829fff03fa938c4fcb43edd494e6").unwrap(),
    H256::from_str("0ccb300f13991b7632d6f2da5199c8e1ca8f67ad0847df8e0e70b97b359a3e3c").unwrap(),
    H256::from_str("d026a5674fd682f29efb3c408a1c3022d9ccfb8c37100b7d6b111f7904c329c9").unwrap(),
    H256::from_str("31ebcff9ff2b1d82965a4482b9ffc83adb5148dc3147e9cef01a8aa9d4741bc7").unwrap(),
    H256::from_str("057c0fe8ee1052d9ea5879e71bcf3b0f63b04c9a31d387e0b48481cc1529521f").unwrap(),
    H256::from_str("07c6e3e86ad550e0db98537ffd1fc40e626dda31fc06395055e8cb9f2f612bba").unwrap(),
    H256::from_str("42814c1d6dd67f7b77c50b16f581f8e2cbd74aaed074760b5ae50b607589061a").unwrap(),
    H256::from_str("52bc6b31d3aaf59f3e0e453049df08a36a92aaf6319f5c2b9314e369afcfa70b").unwrap(),
    H256::from_str("2c1cd6d06ef3d5f2d97921756db301b54edff106e73fdec2ed628b884ad585cb").unwrap(),
    H256::from_str("88cda1185231d86b13dcfc0037f56f68d6fc9664172a57a36f7b7dc351c7092b").unwrap(),
    H256::from_str("5a872018c0c98dcece9cefc79787885bc83a561d4f04377aff4ab3a8122296d6").unwrap(),
    H256::from_str("f9911e9efb62995b7cfbb6c886438ec0983271d5bcd01bcead162f4c460bb926").unwrap(),
    H256::from_str("4aef51af702608d1f3bdd137e12d112efbd7e94b3800b3608f6f88c3dc1880f8").unwrap(),
    H256::from_str("01287b836180c1f40190571c3187c8be5f4ab60deda09949c023242b333d915b").unwrap(),
    H256::from_str("0d11db790649546859d1bc30e6ca5299ecd7c7b493d22fbc1a7e1e15273217da").unwrap(),
    H256::from_str("b0b31b286c15850b380580c1859e72ab0b6d073269b587dc0af7c1c3aaba4a57").unwrap(),
    H256::from_str("05567f0865db1c5d5ca98eed512efcc5b2c629ea0a4f5397c2e251955d1b4efd").unwrap(),
    H256::from_str("7be4283f70254ceb806392647d847c1dd9b0554cd4897f28ebfaf4117d58fddc").unwrap(),
    H256::from_str("5c32fa9ce2f99d0ea4ba72e961a63a471ec75c99816e41dfe331d4359e4570d4").unwrap(),
    H256::from_str("9f7809e0a02b3b406c9e9dc2c4b663b6f75e4ffa9c0c82029066cc6ed90c08c0").unwrap(),
    H256::from_str("4156e64d984bee0019b6e72e12457ad50abc01069bd7a2be5f8d6e1601db44a4").unwrap(),
    H256::from_str("60bb6d278e2552939c8d51833b5d5057596d8ab8dae0348afcb0f2e73e927264").unwrap(),
    H256::from_str("eaabb54a8c87358f3659e94cc3714adb71416fa60847e89d5c02f691e2497870").unwrap(),
    H256::from_str("a2bf2b628bc31ca24c09c1c80db858be8a105e5a15fad520fb017c702d03b7b1").unwrap(),
    ];
    let seed = if !hash_series.is_empty() {
        println!("hash_series {:?}", hash_series.iter().enumerate().map(|(i, h)| (i as i8, 32 as i8, subject, h).using_encoded(sp_io::hashing::blake2_256).into()).collect::<Vec<H256>>());
        println!("index {}, RANDOM_MATERIAL_LEN: {}, subject: {:?}", index,RANDOM_MATERIAL_LEN, subject);
        let series = match hash_series
        .iter()
        .cycle()
        .skip(index)
        .take(RANDOM_MATERIAL_LEN as usize)
        .enumerate().fold(None, |a:Option<String>, (i, h)| {
            Some(match a {
                None => format!("  {:?}, {:?}", H256::from((i as i8, 32 as i8, subject, h).using_encoded(sp_io::hashing::blake2_256)), (i as i8, subject, h)),
                Some(mut a) => {
                    write!(a, "\n  {:?}, {:?}", H256::from((i as i8, 32 as i8, subject, h).using_encoded(sp_io::hashing::blake2_256)), (i as i8, subject, h)).unwrap();
                    a
                }
            })
        }) {
            None => String::new(),
            Some(rval) => rval,
        };
        println!("{}", series);
        println!("result {:?}", hash_series
        .iter()
        .cycle()
        .skip(index)
        .take(RANDOM_MATERIAL_LEN as usize)
        .enumerate()
        .map(|(i, h)| H256::from((i as i8, 32 as i8, subject, h).using_encoded(sp_io::hashing::blake2_256)))
        .triplet_mix());
        // Always the case after block 1 is initialized.
        hash_series
            .iter()
            .cycle()
            .skip(index)
            .take(RANDOM_MATERIAL_LEN as usize)
            .enumerate()
            .map(|(i, h)| (i as i8, 32 as i8, subject, h).using_encoded(sp_io::hashing::blake2_256).into())
            .triplet_mix()
    } else {
        H256::default()
    };

    println!("{}", (16 as i8, 32 as i8, subject, hash_series[15]).using_encoded(|e| format!("{:?}", e)));
    println!("Expected: {}", expect);
    // (seed, block_number.saturating_sub(RANDOM_MATERIAL_LEN.into()))
}

fn main() {
	let args = Args::parse();

	println!("using args: {:?}", args);

	compute(&args);
	
}
