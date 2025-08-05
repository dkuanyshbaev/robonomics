///////////////////////////////////////////////////////////////////////////////
//
//  Copyright 2018-2024 Robonomics Network <research@robonomics.network>
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
//
///////////////////////////////////////////////////////////////////////////////
// Benchmarks for Liability Pallet

#![cfg(feature = "runtime-benchmarks")]

use super::{Pallet as Liability, *};
use frame_benchmarking::v2::*;
use frame_support::pallet_prelude::{Get, MaxEncodedLen};
use frame_system::RawOrigin;
use parity_scale_codec::{Decode, Encode};
use sp_std::prelude::*;
use sp_core::Pair;

const SEED: u32 = 0;

use hex_literal::hex;
// IPFS raw hash (sha256)
const IPFS_HASH: [u8; 32] =
    hex!["30f3d649b3d140a6601e11a2cfbe3560e60dc5434f62d702ac8ceff4e1890015"];

// Define a test runtime for benchmarking
#[cfg(feature = "runtime-benchmarks")]
mod benchmark_runtime {
    use super::*;
    use crate::{self as liability};
    use frame_support::parameter_types;
    use sp_core::H256;
    use sp_runtime::{traits::IdentityLookup, AccountId32};

    // Define our own Block type for benchmarking
    use sp_runtime::generic;
    pub type Block = generic::Block<
        generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
        sp_runtime::generic::UncheckedExtrinsic<
            sp_runtime::MultiAddress<AccountId32, ()>,
            RuntimeCall,
            sp_runtime::MultiSignature,
            (),
        >,
    >;
    type Balance = u128;

    frame_support::construct_runtime!(
        pub enum BenchmarkRuntime {
            System: frame_system,
            Balances: pallet_balances,
            Liability: liability,
        }
    );

    parameter_types! {
        pub const BlockHashCount: u32 = 250;
    }

    impl frame_system::Config for BenchmarkRuntime {
        type RuntimeOrigin = RuntimeOrigin;
        type RuntimeCall = RuntimeCall;
        type Nonce = u32;
        type Block = Block;
        type Hash = H256;
        type Hashing = ::sp_runtime::traits::BlakeTwo256;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
        type RuntimeEvent = RuntimeEvent;
        type BlockHashCount = BlockHashCount;
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = pallet_balances::AccountData<Balance>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type DbWeight = ();
        type BaseCallFilter = frame_support::traits::Everything;
        type SystemWeightInfo = ();
        type BlockWeights = ();
        type BlockLength = ();
        type SS58Prefix = ();
        type OnSetCode = ();
        type MaxConsumers = frame_support::traits::ConstU32<16>;
        type RuntimeTask = RuntimeTask;
        type ExtensionsWeightInfo = ();
        type SingleBlockMigrations = ();
        type MultiBlockMigrator = ();
        type PreInherents = ();
        type PostInherents = ();
        type PostTransactions = ();
    }

    parameter_types! {
        pub const MaxLocks: u32 = 50;
        pub const MaxReserves: u32 = 50;
        pub const ExistentialDeposit: Balance = 10;
    }

    impl pallet_balances::Config for BenchmarkRuntime {
        type MaxLocks = MaxLocks;
        type MaxReserves = MaxReserves;
        type ReserveIdentifier = [u8; 8];
        type Balance = Balance;
        type RuntimeEvent = RuntimeEvent;
        type DustRemoval = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
        type WeightInfo = ();
        type FreezeIdentifier = ();
        type MaxFreezes = ();
        type RuntimeHoldReason = ();
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type DoneSlashHandler = ();
    }

    impl crate::Config for BenchmarkRuntime {
        type RuntimeEvent = RuntimeEvent;
        type Agreement = crate::signed::SignedAgreement<
            crate::technics::IPFS,
            crate::economics::SimpleMarket<Self::AccountId, Balances>,
            Self::AccountId,
            sp_runtime::MultiSignature,
        >;
        type Report = crate::signed::SignedReport<
            Self::Nonce,
            Self::AccountId,
            sp_runtime::MultiSignature,
            crate::technics::IPFS,
        >;
    }
}

#[benchmarks]
mod benchmarks {
    use super::*;
    use crate::economics::SimpleMarket;
    use crate::signed::{SignedAgreement, SignedReport};
    use crate::technics::IPFS;
    use frame_support::pallet_prelude::PhantomData;
    use frame_support::{assert_ok, parameter_types};
    use parity_scale_codec::Encode;
    use sp_core::{sr25519, H256};
    use sp_runtime::MultiSignature;

    use sp_keyring::AccountKeyring;
    use sp_runtime::{
        traits::{IdentifyAccount, IdentityLookup, Verify},
        AccountId32,
    };

    #[benchmark]
    fn create() {
        let caller: T::AccountId = whitelisted_caller();

        let technics = IPFS {
            hash: IPFS_HASH.into(),
        };

        // Create SimpleMarket with the BenchmarkRuntime's Balances type
        #[cfg(feature = "runtime-benchmarks")]
        use benchmark_runtime::{Balances as BenchmarkBalances, BenchmarkRuntime};

        let economics = SimpleMarket::<AccountId32, BenchmarkBalances> {
            price: 10, // This represents 10 units of the currency
        };

        // Use test accounts from AccountKeyring
        let promisee: AccountId32 = AccountKeyring::Alice.into();
        let promisor: AccountId32 = AccountKeyring::Bob.into();
        
        // In benchmarking context, signature verification is bypassed (see signed.rs verify() method)
        // so we can use dummy signatures
        let dummy_signature = MultiSignature::Sr25519(sr25519::Signature::from_raw([0u8; 64]));
        let promisee_signature = dummy_signature.clone();
        let promisor_signature = dummy_signature;

        // Create the signed agreement with concrete types
        let signed = SignedAgreement {
            technics,
            economics,
            promisee,
            promisor,
            promisee_signature,
            promisor_signature,
        };

        // Encode and decode into T::Agreement
        let agreement = T::Agreement::decode(&mut &signed.encode()[..])
            .expect("Failed to decode agreement for benchmarking");

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), agreement);
    }

    #[benchmark]
    fn finalize() {
        let caller: T::AccountId = whitelisted_caller();

        // Create a liability first
        let technics = IPFS { hash: IPFS_HASH.into() };
        let economics = SimpleMarket::<AccountId32, benchmark_runtime::Balances> { price: 10 };
        let promisee: AccountId32 = AccountKeyring::Alice.into();
        let promisor: AccountId32 = AccountKeyring::Bob.into();
        let dummy_signature = MultiSignature::Sr25519(sr25519::Signature::from_raw([0u8; 64]));

        let agreement = SignedAgreement {
            technics,
            economics,
            promisee,
            promisor: promisor.clone(),
            promisee_signature: dummy_signature.clone(),
            promisor_signature: dummy_signature.clone(),
        };

        let agreement_typed = T::Agreement::decode(&mut &agreement.encode()[..])
            .expect("Failed to decode agreement");

        let _ = Liability::<T>::create(RawOrigin::Signed(caller.clone()).into(), agreement_typed);

        // Create report to finalize the liability
        let signed_report = SignedReport {
            index: 0u32,
            sender: promisor,
            payload: IPFS { hash: IPFS_HASH.into() },
            signature: dummy_signature,
        };

        let report = ReportFor::<T>::decode(&mut &signed_report.encode()[..])
            .expect("Failed to decode report");

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), report);
    }

    impl_benchmark_test_suite!(
        Liability,
        crate::tests::new_test_ext(),
        crate::tests::Runtime,
    );
}
