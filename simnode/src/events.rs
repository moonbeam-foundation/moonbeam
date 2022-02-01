// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

/// Concrete event type for verbose event asserts in tests.
#[derive(derive_more::From)]
pub enum AllRuntimeEvents {
	/// Moonbase runtime events
	Moonbase(moonbase_runtime::Event),
}

/// Convenience method to match on [`AllRuntimeEvents`]
#[macro_export]
macro_rules! match_event {
	($ev:expr, $event:ident, $sub_ev:pat) => {{
		matches!(
			$ev,
			AllRuntimeEvents::Moonbase(moonbase_runtime::Event::$event($sub_ev))
		)
	}};
}
