// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Implements the dispute coordinator subsystem.
//!
//! This is the central subsystem of the node-side components which participate in disputes.
//! This subsystem wraps a database which tracks all statements observed by all validators over some window of sessions.
//! Votes older than this session window are pruned.
//!
//! This subsystem will be the point which produce dispute votes, eiuther positive or negative, based on locally-observed
//! validation results as well as a sink for votes received by other subsystems. When importing a dispute vote from
//! another node, this will trigger the dispute participation subsystem to recover and validate the block and call
//! back to this subsystem.

use std::collections::HashMap;
use std::sync::Arc;

use polkadot_node_primitives::CandidateVotes;
use polkadot_node_subsystem::{
	messages::{
		RuntimeApiRequest, DisputeCoordinatorMessage,
	},
	Subsystem, SubsystemContext, SubsystemResult, FromOverseer, OverseerSignal, SpawnedSubsystem,
	SubsystemError,
};
use polkadot_primitives::v1::{SessionIndex, CandidateHash};

use sc_keystore::LocalKeystore;

mod db;

struct State {
	keystore: Arc<LocalKeystore>,
	overlay: HashMap<(SessionIndex, CandidateHash), CandidateVotes>,
	highest_session: SessionIndex,
}
