// Copyright 2019-2021 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

use evm::{ExitReason, Runtime, executor::{Hook, StackExecutor, StackState}};
use moonbeam_rpc_primitives_debug::single::TransactionTrace;

mod raw;
mod call_list;

pub struct TracingHook(TracingHookInner);

impl TracingHook {
    pub fn new_raw(
        disable_storage: bool,
        disable_memory: bool,
        disable_stack: bool,
    ) -> Self {
        Self(TracingHookInner::Raw(raw::State::new(
            disable_storage,
            disable_memory,
            disable_stack,
        )))
    }

    pub fn new_call_list() -> Self {
        Self(TracingHookInner::CallList(call_list::State::new()))
    }

    pub fn finish(self) -> TransactionTrace {
        match self.0 {
            TracingHookInner::Raw(state) => state.finish(),
            TracingHookInner::CallList(state) => state.finish(),
        }
    }
}

enum TracingHookInner {
    Raw (raw::State),
    CallList(call_list::State),
}

impl Hook for TracingHook {
    /// Called before the execution of a context.
    fn before_loop<'config, S: StackState<'config>, H: Hook>(
        &mut self,
        executor: &StackExecutor<'config, S, H>,
        runtime: &Runtime,
    ) {
        match &mut self.0 {
            TracingHookInner::Raw(state) => state.before_loop(executor, runtime),
            TracingHookInner::CallList(state) => state.before_loop(executor, runtime),
        }
    }

	/// Called before each step.
    fn before_step<'config, S: StackState<'config>, H: Hook>(
        &mut self,
        executor: &StackExecutor<'config, S, H>,
        runtime: &Runtime,
    ) {
        match &mut self.0 {
            TracingHookInner::Raw(state) => state.before_step(executor, runtime),
            TracingHookInner::CallList(state) => state.before_step(executor, runtime),
        }
    }

	/// Called after each step. Will not be called if runtime exited
    /// from the loop.
    fn after_step<'config, S: StackState<'config>, H: Hook>(
        &mut self,
        executor: &StackExecutor<'config, S, H>,
        runtime: &Runtime,
    ) {
        match &mut self.0 {
            TracingHookInner::Raw(state) => state.after_step(executor, runtime),
            TracingHookInner::CallList(state) => state.after_step(executor, runtime),
        }
    }

	/// Called after the execution of a context.
    fn after_loop<'config, S: StackState<'config>, H: Hook>(
        &mut self,
        executor: &StackExecutor<'config, S, H>,
        runtime: &Runtime,
        reason: &ExitReason,
    ) {
        match &mut self.0 {
            TracingHookInner::Raw(state) => state.after_loop(executor, runtime, reason),
            TracingHookInner::CallList(state) => state.after_loop(executor, runtime, reason),
        }
    }
}