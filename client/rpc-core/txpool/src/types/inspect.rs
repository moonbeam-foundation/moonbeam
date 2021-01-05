use ethereum_types::{H160, U256};
use serde::{Serialize, Serializer};

#[derive(Clone)]
pub struct Summary {
	pub to: Option<H160>,
	pub value: U256,
	pub gas: U256,
	pub gas_price: U256,
}

impl Serialize for Summary {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let res = format!(
			"{:?}: {} wei + {} gas x {} wei",
			self.to, self.value, self.gas, self.gas_price
		);
		serializer.serialize_str(&res)
	}
}
