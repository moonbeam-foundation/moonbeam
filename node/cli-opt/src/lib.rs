use std::str::FromStr;

/// Block authoring scheme to be used by the dev service.
#[derive(Debug)]
pub enum Sealing {
	/// Author a block immediately upon receiving a transaction into the transaction pool
	Instant,
	/// Author a block upon receiving an RPC command
	Manual,
	/// Author blocks at a regular interval specified in milliseconds
	Interval(u64),
}

impl FromStr for Sealing {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"instant" => Self::Instant,
			"manual" => Self::Manual,
			s => {
				let millis =
					u64::from_str_radix(s, 10).map_err(|_| "couldn't decode sealing param")?;
				Self::Interval(millis)
			}
		})
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum EthApi {
	Txpool,
	Debug,
	Trace,
}

impl FromStr for EthApi {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"txpool" => Self::Txpool,
			"debug" => Self::Debug,
			"trace" => Self::Trace,
			_ => {
				return Err(format!(
					"`{}` is not recognized as a supported Ethereum Api",
					s
				))
			}
		})
	}
}

pub struct RpcParams {
    ethapi_max_permits: u32,
    ethapi_trace_max_count: u32,
    ethapi_trace_cache_duration: u64,
    max_past_logs: u32,
}
