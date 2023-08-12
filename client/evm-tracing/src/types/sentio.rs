use crate::types::serialization::*;

use ethereum_types::{H160, H256, U256};
use parity_scale_codec::{Decode, Encode};
use serde::{Serialize, Deserialize, Serializer};

#[derive(Clone, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SentioTracerConfig {
	#[serde(default)]
	pub functions: std::collections::HashMap<String, Vec<FunctionInfo>>,

	#[serde(default)]
	pub calls: std::collections::HashMap<String, Vec<u64>>,

	#[serde(default)]
	pub debug: bool,

	#[serde(default)]
	pub with_internal_calls: bool,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct FunctionInfo {
	pub address: H160,
	pub name: String,
	pub signature_hash: String,
	pub pc: u64,
	pub input_size: u64,
	pub input_memory: bool,
	pub output_size: u64,
	pub output_memory: bool,
}

#[derive(Clone, Eq, PartialEq, Default, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SentioBaseTrace {
	// Only in debug mode, TODO p3, make it vec<u8> and have it serialize to json
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tracer_config: Option<String>,

	#[serde(rename = "type", serialize_with = "opcode_serialize")]
	pub op: Vec<u8>,
	pub pc: u64,
	pub start_index: i32,
	pub end_index: i32,

	#[serde(serialize_with = "u64_serialize")]
	pub gas: u64,
	#[serde(serialize_with = "u64_serialize")]
	pub gas_used: u64,
	#[serde(skip)]
	#[codec(skip)]
	pub gas_cost: u64,

	#[serde(serialize_with = "string_serialize", skip_serializing_if = "Vec::is_empty")]
	pub error: Vec<u8>,

	#[serde(serialize_with = "string_serialize", skip_serializing_if = "Vec::is_empty")]
	pub revert_reason: Vec<u8>,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum SentioTrace {
	EventTrace(SentioEventTrace),
	CallTrace(SentioCallTrace),
	OtherTrace(SentioBaseTrace),
	// InternalCallTrace(SentioInternalCallTrace)
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
pub struct Log {
	pub address: H160,
	pub topics: Vec<H256>,
	#[serde(serialize_with = "bytes_0x_serialize")]
	pub data: Vec<u8>,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SentioEventTrace {
	#[serde(flatten)]
	pub base: SentioBaseTrace,

	#[serde(flatten)]
	pub log: Log,
}

#[derive(Clone, Eq, PartialEq, Default, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SentioCallTrace {
	#[serde(flatten)]
	pub base: SentioBaseTrace,
	pub traces: Vec<SentioTrace>,
	pub from: H160,
	#[serde(serialize_with = "bytes_0x_serialize")]
	pub output: Vec<u8>,

	// for external call
	pub to: H160,
	#[serde(serialize_with = "bytes_0x_serialize")]
	pub input: Vec<u8>,
	pub value: U256,// TODO use some

	// for internal trace
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub input_stack: Vec<H256>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub input_memory: Option<Vec<H256>>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub output_stack: Vec<H256>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub output_memory: Option<Vec<H256>>,
	pub function_pc: u64,
	#[serde(skip)]
	#[codec(skip)]
	pub exit_pc: u64,
	#[codec(skip)]
	#[serde(skip)]
	pub function: Option<FunctionInfo>,
}
//
// #[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct SentioInternalCallTrace {
//   #[serde(skip_serializing_if = "Option::is_none")]
//   pub name: Option<String>,
//
//   #[serde(flatten)]
//   pub base: BaseSentioTrace,
//   pub traces: Vec<SentioTrace>,
//   pub from: H160,
//   #[serde(serialize_with = "bytes_0x_serialize")]
//   pub output: Vec<u8>,
//
//
// }

pub fn u64_serialize<S>(data: &u64, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
{
	serializer.serialize_str(&format!("0x{:x}", *data))
}
