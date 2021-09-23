use ethereum_types::H256;
use serde::{de::Error, Deserialize, Deserializer};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum RequestBlockId {
	Number(#[serde(deserialize_with = "deserialize_u32_0x")] u32),
	Hash(H256),
	Tag(RequestBlockTag),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RequestBlockTag {
	Earliest,
	Latest,
	Pending,
}

fn deserialize_u32_0x<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
	D: Deserializer<'de>,
{
	let buf = String::deserialize(deserializer)?;

	let parsed = match buf.strip_prefix("0x") {
		Some(buf) => u32::from_str_radix(&buf, 16),
		None => u32::from_str_radix(&buf, 10),
	};

	parsed.map_err(|e| Error::custom(format!("parsing error: {:?} from '{}'", e, buf)))
}
