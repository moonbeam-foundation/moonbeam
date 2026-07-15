// Copyright 2025 Moonbeam foundation
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

//! Persistent, content-addressed cache for lazy-loading fork state.
//!
//! Lazy loading forks a remote chain and fetches state on demand from a public
//! RPC endpoint. That endpoint can be slow or rate-limited, which makes CI runs
//! flaky (state fetches time out). Because the state at a *specific block hash*
//! is immutable, every response for a given `(method, block, key)` can be cached
//! on disk and safely reused across process restarts — in particular across the
//! CI retry attempts that share the same working directory.
//!
//! The cache is intentionally simple: each entry is a file named after the
//! blake2 hash of its logical key, storing the JSON-encoded RPC response. Misses
//! and decode errors fall through to the network, so a corrupt or partial entry
//! is never fatal.

use sp_core::blake2_256;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Name of the marker file used to pin the auto-resolved fork block so that
/// subsequent runs/retries fork from the exact same block (and therefore reuse
/// the cached state instead of re-fetching a moved `latest`).
const PINNED_FORK_BLOCK_FILE: &str = "fork-block";

/// A persistent, content-addressed cache stored on the local filesystem.
#[derive(Debug)]
pub struct StateCache {
	root: PathBuf,
	counter: AtomicU64,
}

impl StateCache {
	/// Open (creating if necessary) a cache rooted at `root`.
	pub fn new(root: PathBuf) -> std::io::Result<Self> {
		fs::create_dir_all(&root)?;
		Ok(Self {
			root,
			counter: AtomicU64::new(0),
		})
	}

	/// Compute the on-disk path for a logical cache key. Entries are sharded into
	/// 256 sub-directories (by the first hash byte) to avoid a single directory
	/// with a huge number of files.
	fn path_for(&self, namespace: &str, key_bytes: &[u8]) -> PathBuf {
		let mut input = Vec::with_capacity(namespace.len() + 1 + key_bytes.len());
		input.extend_from_slice(namespace.as_bytes());
		input.push(b':');
		input.extend_from_slice(key_bytes);
		let digest = blake2_256(&input);
		let hex = hex::encode(digest);
		self.root.join(&hex[0..2]).join(hex)
	}

	/// Read a raw cache entry, returning `None` on a miss or any I/O error.
	pub fn get(&self, namespace: &str, key_bytes: &[u8]) -> Option<Vec<u8>> {
		fs::read(self.path_for(namespace, key_bytes)).ok()
	}

	/// Write a raw cache entry. Best-effort: failures are ignored so caching can
	/// never break an otherwise-successful request. Writes go through a uniquely
	/// named temporary file and an atomic rename so concurrent writers (the RPC
	/// client is shared across threads) never observe a partially written entry.
	pub fn put(&self, namespace: &str, key_bytes: &[u8], value: &[u8]) {
		let path = self.path_for(namespace, key_bytes);
		if let Some(parent) = path.parent() {
			if fs::create_dir_all(parent).is_err() {
				return;
			}
		}
		let id = self.counter.fetch_add(1, Ordering::Relaxed);
		let tmp = path.with_extension(format!("tmp-{}-{}", std::process::id(), id));
		if fs::write(&tmp, value).is_ok() {
			// If the rename fails (e.g. another writer won the race), drop the temp.
			if fs::rename(&tmp, &path).is_err() {
				let _ = fs::remove_file(&tmp);
			}
		} else {
			let _ = fs::remove_file(&tmp);
		}
	}

	/// Read the pinned fork block hash (as raw bytes) previously stored by
	/// [`Self::set_pinned_fork_block`], if any.
	///
	/// The pin is scoped to `fingerprint` (an identifier of the remote
	/// endpoint/chain that produced it). If the stored fingerprint does not match
	/// — e.g. the same cache directory is reused against a different network — the
	/// pin is ignored so a foreign block hash is never forked from.
	pub fn pinned_fork_block(&self, fingerprint: &str) -> Option<Vec<u8>> {
		let raw = fs::read_to_string(self.pinned_fork_block_path()).ok()?;
		let mut lines = raw.lines();
		let stored_fingerprint = lines.next()?.trim();
		if stored_fingerprint != fingerprint {
			return None;
		}
		let hash = lines.next()?.trim();
		hex::decode(hash.strip_prefix("0x").unwrap_or(hash)).ok()
	}

	/// Persist the resolved fork block hash (scoped to `fingerprint`) so later
	/// runs against the same endpoint/chain fork from the exact same block.
	pub fn set_pinned_fork_block(&self, fingerprint: &str, hash_bytes: &[u8]) {
		let _ = fs::write(
			self.pinned_fork_block_path(),
			format!("{}\n0x{}\n", fingerprint, hex::encode(hash_bytes)),
		);
	}

	fn pinned_fork_block_path(&self) -> PathBuf {
		self.root.join(PINNED_FORK_BLOCK_FILE)
	}
}

/// Derive a stable, non-sensitive fingerprint for a remote endpoint.
///
/// The fingerprint is used to scope the pinned fork block to the endpoint that
/// produced it. It is a blake2 hash of the URL rather than the URL itself, so a
/// credential- or API-key-carrying endpoint is never written to disk in clear
/// text.
pub fn endpoint_fingerprint(url: &str) -> String {
	hex::encode(blake2_256(url.as_bytes()))
}

/// Redact a remote endpoint URL for logging, keeping only `scheme://host[:port]`.
///
/// Userinfo (`user:pass@`), the path, and the query string are dropped, since
/// any of them may embed credentials or API keys.
pub fn redact_url(url: &str) -> String {
	let (scheme, rest) = match url.split_once("://") {
		Some(parts) => parts,
		None => return "<redacted>".to_string(),
	};
	// Drop userinfo ("user:pass@") if present.
	let authority = rest.rsplit_once('@').map(|(_, a)| a).unwrap_or(rest);
	// The authority (host[:port]) ends at the first '/', '?' or '#'.
	let host = authority
		.split(|c| c == '/' || c == '?' || c == '#')
		.next()
		.unwrap_or(authority);
	if host.is_empty() {
		"<redacted>".to_string()
	} else {
		format!("{scheme}://{host}")
	}
}
