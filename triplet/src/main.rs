
use safe_mix::TripletMix;
use sp_core::{H256};
use sp_runtime::traits::{BlakeTwo256, Hash};
use codec::Encode;
use std::str::FromStr;
use std::fmt::Write;

const RANDOM_MATERIAL_LEN: u32 = 81;

fn main() {
    let index = 66;
    let subject:[u8; 8] = [102, 105, 108, 116, 101, 114, 0, 227];

    let expect = "0x5fe7029d8bcf64f7e05f286ad772eea521d4e66cb6cc93d6bfa11ac8e45f2145";
    // 0x9acdd8c2f25e0fe876441d86c91f133a80c8dcfbb0a3997b3a2ba50580ba23ff, (0, [102, 105, 108, 116, 101, 114, 0, 21], 0x6c6776eca9a28ae89c2ccc1a576e12d3706e909d31fdda1f5c09f22de4a3137c)
    let hash_series: Vec<H256> = vec![
    H256::from_str("f00bdf0731b15ea9961c2188bf142a6d06fd80536149326c8d6a64c08f621ede").unwrap(),
    H256::from_str("f55b5fafba7785fe817625e3f4746f80532c9724b6c08bc3ff870f3eab9c548d").unwrap(),
    H256::from_str("1aa389975db07b57d6cded66eef9a76b4b6d3443e08e23e1d57fb7f4bdffd145").unwrap(),
    H256::from_str("450fc6cf176eace177dd654276e4751ef3028654461c8c96f3b83fccd63e90d1").unwrap(),
    H256::from_str("6ec567185a016df312eaa99a83446adb9a94255e7dd64ed27eaf0871cb60a35d").unwrap(),
    H256::from_str("cc30cdb5024f8d2bd9c51d9623cec9d9959467dd4a8e00162629eab36d355ebd").unwrap(),
    H256::from_str("6440ae56588f7063583ab106aa7a3d9657378f9bcc73476514c8961fb3f1a6af").unwrap(),
    H256::from_str("aef14341a5c2f34f189259f9585bc13205d3e88e1a35080014656cbd1e3ca025").unwrap(),
    H256::from_str("8b705f14320069fec2ec2e779a8f3fa802e128c46ef5e43aa2160fd71095311d").unwrap(),
    H256::from_str("2a77411397b16bfb0255a896fa52e2747ae77208f4365e7526416a18833a0815").unwrap(),
    H256::from_str("59c7f012f626027ecb23b532dd52c0d45ef6d907531222f982bb058d5c542f3a").unwrap(),
    H256::from_str("28f02c70a005e1ae77a04bd122b18f16355d7daa26f545021c1762ad66cc2bdc").unwrap(),
    H256::from_str("83fc07ec211a9a7bd295939acfae81296a394113d8eab77e0cd78d753fb990eb").unwrap(),
    H256::from_str("d359b6dfbf4c270abc55cd5dd4d9c0acef4d43f742ae7e1a24b20ecc3975ee60").unwrap(),
    H256::from_str("1687952c9450b32d83ec94ad0d2ed035be927986e5c4b6d34137e86b17a9d3fa").unwrap(),
    H256::from_str("01d6fc7895a55c57d2a15993731c78788ced017869b0ff830ff5c685a44e2754").unwrap(),
    H256::from_str("dfe8b621f541b01e8a2aa9218ac97ae9fac0a3b3281d42b1fc16474c075cb12e").unwrap(),
    H256::from_str("9854276482a01a510ad0cd3d5c10e54bf3f9de49431435e3ed5176323440c1a0").unwrap(),
    H256::from_str("3b310bf2b1e3e304cb138f3ca9814002d9c9fe6a7bed56d3cbd3b1098746ef80").unwrap(),
    H256::from_str("2655b69b4f90d630afc700b0caf69f0624bb8e7f44fd60bd19a362e2c017a6b1").unwrap(),
    H256::from_str("1aec2e9d50cf5e120372c53faec136725ce01901ce615d21ad1f05f5beadc3aa").unwrap(),
    H256::from_str("bfdd7042057b531237b037fd59930e9b3f617912adaf27d25b7df505862db932").unwrap(),
    H256::from_str("057ad0d872d02c6d961ecc36c270bb135e1a688d32da4a7716b9d1017ecde97a").unwrap(),
    H256::from_str("0d04739b485006e7058bfa10df3dce9776c6968a3a11301530a680a1a4818c4f").unwrap(),
    H256::from_str("060ef0bb8c17f738173314d82756f7becc749c30fa54bbc68f45a1469b7abac8").unwrap(),
    H256::from_str("1a8216e8e67820fd5352adaab04a22ef3f0aa744e70afb3da90531d06496bab2").unwrap(),
    H256::from_str("7f66a9706d0ac704fc71798876c552a551c5d659865876d24f91318eb421e7e0").unwrap(),
    H256::from_str("c22b8ea8a7fe2c5a2ee7af501458b4a11aca7dc5af8e3b3b4adf49056182d932").unwrap(),
    H256::from_str("e6690b8c32dfe117511d00bf9f623e62f829ae96eb44a5b633b3febf14df1ec4").unwrap(),
    H256::from_str("69e490498a38d0fae215754f66a46a22b179c18db2cb587ad40d1e70062658c3").unwrap(),
    H256::from_str("c004d6d51b1746c44ee90831f145b06e9c13c42eaa4117a8e99fefa89d458ed2").unwrap(),
    H256::from_str("3b95c042faa6572d21a72a65688433a1548530bda000ee4404e1ac4bde1e806a").unwrap(),
    H256::from_str("b900aa079d91a3a1facd971a62af7100cc3506e3a524ea616e1159d07b33b7bd").unwrap(),
    H256::from_str("ea24bd3fe77426837d2c15e63146a7c73eb9b704d305fcd1d4f1b9c5b2e94e97").unwrap(),
    H256::from_str("2c3cdde7c5d2cfa9771a3c38b8a7d07ce83fef470215d93419707df91dbbdc83").unwrap(),
    H256::from_str("e507e3467376e5e249ec794d01eb8241fcf5e21f6312cd830871bf41d1ae7a8f").unwrap(),
    H256::from_str("af046eafe2a0f7e4a04c9fb3ee8cf4681b21b25a7f98ea88c03c78e1983b310e").unwrap(),
    H256::from_str("4abff8268dbf73d4ed7b86de2a8b4969e1df5b113d850d1f33e7e508923d635e").unwrap(),
    H256::from_str("394085beda0d802c3d51102504509db28c5357237fb39225531e591599fa71e3").unwrap(),
    H256::from_str("8b80c127ccbfc6585f77bc2f0b85fbfdb69d89763857627b12c6657434b57d5c").unwrap(),
    H256::from_str("7811048302bbd296b2a602ef0074148ddfc49d9443368fa0bc9a2783b01a815c").unwrap(),
    H256::from_str("346f21db5893e131421e00d4e012e5f7c87b31868e16cd2d5a6a2f3b9a2b552a").unwrap(),
    H256::from_str("9f217779f44b6eff15d3fb8ed781a01099fa5a7805d6a7f1661a65da13a66879").unwrap(),
    H256::from_str("7cd93e82a9a565e4ba9ac8637d8ac18bcda2a3bd5b4e0a0c98ecae1b1b87a78e").unwrap(),
    H256::from_str("54b2c90288db85f6b7c09253c2868c99e5c81a78105cec900738bff12b223449").unwrap(),
    H256::from_str("5088ef6c687a3309a9446fdd85a686f1f8ef15c4f6787487fc1dd62fbd92163a").unwrap(),
    H256::from_str("13e5f4c8cff0b26e73632dcf17ae8f4154c2fb9e070e839f6457d6c93cc23a09").unwrap(),
    H256::from_str("dbe277bb346c40bf2a5936731524cb7e865669672a369e5b71040d61604257a9").unwrap(),
    H256::from_str("45f24040e98c65c67d86dcc0cad2c6cf1163b481918afed4120b095394784dfc").unwrap(),
    H256::from_str("cacc78b6616546dff82c3f43edd6e8e43602f4f4d12d84e16a1cc6482f5ef438").unwrap(),
    H256::from_str("bc2f0a7cab116be0a4692e52f0f09d1196862dd6365b5f07c459ad86fffbe6fe").unwrap(),
    H256::from_str("fb723abd858b44c3fc14cb98295e4fde92f0d172c7ab616b6e721c4620f91537").unwrap(),
    H256::from_str("63361b24df0994d85ae484aece8e7f05b209488bff3966c672ec21947d482f3f").unwrap(),
    H256::from_str("30755f12ebdf6b1a11ab67cad792f70035d8244aaefbb2415c53b1e0f04b1704").unwrap(),
    H256::from_str("7cf4983aa5575ebc49d7df0c720fa7d63a94efee3e407b7e7580bc573eb94536").unwrap(),
    H256::from_str("c4de394b8e6a469ba1242a0ede1e8e0541b4933331150897121fa900f46fde62").unwrap(),
    H256::from_str("4d0b8e5b93b160f2005c738dfbab03c09ef705e72fb550bb82cc1280987c6303").unwrap(),
    H256::from_str("9e389cb99fef7d079b5fc859f7d9c14e54017bfb1e85090f864f2129223351ff").unwrap(),
    H256::from_str("70bc33d6b56e5cad343aa7cb16f0609d48ec35499a8c0fdf481a789bc26e1498").unwrap(),
    H256::from_str("10a705ac58abd9f880e6cf2c2573c1742fb82646b19a07572c86c74654fbd30e").unwrap(),
    H256::from_str("0b25d0d6820f9ebf6af60048cd389872a69a02aea7cd1b24c1abc84a31e6f4ba").unwrap(),
    H256::from_str("37b1ece42fab52d470b706558c768a23ace55cff793003ee7662a2fb6bf85e7b").unwrap(),
    H256::from_str("25c9952f7c5c0e5b41d115db1d5f769384971c515c0b321b31ab62e9be8f9246").unwrap(),
    H256::from_str("e1424448042ce1dd2b727d23217443749ed44a890a825b67f853dcaa60df68ce").unwrap(),
    H256::from_str("9174cdb572abfa9766f19689bc87ce6320360cfe806e57a60e72d7b410612584").unwrap(),
    H256::from_str("ba8b274ed8f6637f34bcb095eebcca79c26ee28499d3501f5ea1b7b53c725a69").unwrap(),
    H256::from_str("85526547b5cc63117ea3b554ee6b5e2fd0fc6a1852e9d821b0285312c945b567").unwrap()
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