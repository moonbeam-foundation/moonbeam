#![cfg_attr(not(feature = "std"), no_std)]
use sp_runtime_interface::runtime_interface;
#[cfg(feature = "std")]
use sp_externalities::ExternalitiesExt;
use sp_std::prelude::Vec;

pub struct RawTracer {
    items: Vec<u8>
}

impl RawTracer {
    pub fn new() -> Self {
        RawTracer {
            items: Vec::new()
        }
    }
    pub fn push(&mut self, step: u8) {
        self.items.push(step);
    }
    pub fn get(&self) -> Vec<u8> {
        self.items.clone()
    }
}

#[cfg(feature = "std")]
sp_externalities::decl_extension! {
	pub struct RawTracerExt(RawTracer);
}

#[runtime_interface]
pub trait MoonbeamExt {
    fn start_trace_raw(&mut self) {
        // TODO: pass RawTracer instance
        self.register_extension(RawTracerExt(RawTracer::new()))
			.expect("Failed to register required extension: `RawTracerExt`");
    }
    fn record_raw_step(&mut self, step: u8) {
        let ext = self.extension::<RawTracerExt>()
            .expect("Cannot find registered extension: `RawTracerExt`");
		
        ext.push(step);
        println!("Items: {:?}", ext.get());
	}
}