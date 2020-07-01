// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Module to process public crowdsale of DOTs.

use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
use frame_support::{decl_event, decl_storage, decl_module, decl_error, ensure};
use frame_support::traits::{EnsureOrigin, IsDeadAccount};

/// Configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type ValidityOrigin: EnsureOrigin<Self::Origin>;
}

/// The kind of a statement an account needs to make for a claim to be valid.
#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum AccountValidity {
	/// Account is not valid.
	Invalid,
	/// Account is pending validation.
	Pending,
	/// Account is valid with a low contribution amount.
	ValidLow,
	/// Account is valid with a high contribution amount.
	ValidHigh,
}

impl Default for AccountValidity {
	fn default() -> Self {
		AccountValidity::Invalid
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// Someone's account validity was updated
		ValidityUpdated(AccountId, AccountValidity),
		/// Someone's account validity statement was removed
		ValidityRemoved(AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Account used in the crowdsale already exists.
		ExistingAccount,
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Crowdsale {
		ValidityStatements: map hasher(blake2_128_concat) T::AccountId => AccountValidity;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		/// Deposit one of this module's events by using the default implementation.
		fn deposit_event() = default;

		/// Add a validity statement to a specified account.
		///
		/// Origin must match the `ValidityOrigin`.
		#[weight = 0]
		fn set_account_validity(origin, who: T::AccountId, validity: AccountValidity) {
			T::ValidityOrigin::ensure_origin(origin)?;
			ensure!(system::Module::<T>::is_dead_account(&who), Error::<T>::ExistingAccount);
			ValidityStatements::<T>::insert(&who, validity);
			Self::deposit_event(RawEvent::ValidityUpdated(who, validity));
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	use sp_core::H256;
	// The testing primitives are very useful for avoiding having to work with signatures
	// or public keys. `u64` is used as the `AccountId` and no `Signature`s are required.
	use sp_runtime::{Perbill, traits::{BlakeTwo256, IdentityLookup}, testing::Header};
	use frame_support::{
		impl_outer_origin, impl_outer_dispatch, assert_ok, assert_noop, parameter_types,
		ord_parameter_types, dispatch::DispatchError::BadOrigin,
	};
	use frame_support::traits::Currency;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	impl_outer_dispatch! {
		pub enum Call for Test where origin: Origin {
			crowdsale::Crowdsale,
		}
	}
	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u32 = 250;
		pub const MaximumBlockWeight: u32 = 4 * 1024 * 1024;
		pub const MaximumBlockLength: u32 = 4 * 1024 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type BaseCallFilter = ();
		type Origin = Origin;
		type Call = Call;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<u64>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type DbWeight = ();
		type BlockExecutionWeight = ();
		type ExtrinsicBaseWeight = ();
		type MaximumExtrinsicWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
		type ModuleToIndex = ();
		type AccountData = balances::AccountData<u64>;
		type OnNewAccount = ();
		type OnKilledAccount = Balances;
	}

	ord_parameter_types! {
		pub const Six: u64 = 6;
	}

	parameter_types! {
		pub const ExistentialDeposit: u64 = 1;
		pub const CreationFee: u64 = 0;
		pub const MinVestedTransfer: u64 = 0;
	}

	impl balances::Trait for Test {
		type Balance = u64;
		type Event = ();
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
	}

	impl Trait for Test {
		type Event = ();
		type ValidityOrigin = system::EnsureSignedBy<Six, u64>;
	}
	type System = system::Module<Test>;
	type Balances = balances::Module<Test>;
	type Crowdsale = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	pub fn new_test_ext() -> sp_io::TestExternalities {
		let t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
		t.into()
	}

	#[test]
	fn set_account_validity_works() {
		new_test_ext().execute_with(|| {
			// User initially has no validity statement, by default, they are `Invalid`.
			assert_eq!(ValidityStatements::<Test>::get(42), AccountValidity::Invalid);
			// Origin must be the `ValidityOrigin`
			assert_noop!(Crowdsale::set_account_validity(Origin::signed(1), 42, AccountValidity::ValidLow), BadOrigin);
			assert_ok!(Crowdsale::set_account_validity(Origin::signed(6), 42, AccountValidity::ValidLow));
			// Account is updated
			assert_eq!(ValidityStatements::<Test>::get(42), AccountValidity::ValidLow);
			// We update validity statement
			assert_ok!(Crowdsale::set_account_validity(Origin::signed(6), 42, AccountValidity::Invalid));
			assert_eq!(ValidityStatements::<Test>::get(42), AccountValidity::Invalid);
		});
	}

	#[test]
	fn set_account_validity_handles_basic_errors() {
		new_test_ext().execute_with(|| {
			// Create an "existing account"
			Balances::make_free_balance_be(&42, 500);
			// Origin must be the `ValidityOrigin`
			assert_noop!(Crowdsale::set_account_validity(Origin::signed(1), 42, AccountValidity::ValidLow), BadOrigin);
			// Account must be dead
			assert_noop!(Crowdsale::set_account_validity(Origin::signed(6), 42, AccountValidity::ValidLow), Error::<Test>::ExistingAccount);
		});
	}
}