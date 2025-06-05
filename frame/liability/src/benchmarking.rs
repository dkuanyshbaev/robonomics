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

const SEED: u32 = 0;

use hex_literal::hex;
// IPFS raw hash (sha256)
const IPFS_HASH: [u8; 32] =
    hex!["30f3d649b3d140a6601e11a2cfbe3560e60dc5434f62d702ac8ceff4e1890015"];

// #[benchmarks]
// mod benchmarks {
//     use super::*;
//     use crate::economics::SimpleMarket;
//     use crate::signed::*;
//     use crate::technics::IPFS;
//     use crate::traits::*;
//     use crate::{self as liability, *};
//     use frame_support::pallet_prelude::PhantomData;
//     use frame_support::{assert_ok, parameter_types};
//     use frame_system::offchain::AppCrypto;
//     use frame_system::RawOrigin;
//     use signed::*;
//     use sp_core::{crypto::Pair, sr25519, H256};
//     use sp_runtime::traits::IdentifyAccount;
//     use sp_runtime::traits::Verify;
//     use sp_runtime::MultiSignature;
//
//     #[benchmark]
//     fn create() {
//         let caller: T::AccountId = whitelisted_caller();
//         let technics = IPFS {
//             hash: IPFS_HASH.into(),
//         };
//         let economics = SimpleMarket { price: 10 };
//         // let pair = sr25519::Pair::from_string("//Alice", None).unwrap();
//         // let sender = <MultiSignature as Verify>::Signer::from(pair.public()).into_account();
//
//         let signature: MultiSignature = (technics, economics)
//             // .using_encoded(|params| AppCrypto::sign(params, sender.clone()))
//             .using_encoded(|params| AppCrypto::sign(params, caller.clone()))
//             .expect("unable to sign using runtime application key");
//
//         // type Agreement = pallet_robonomics_liability::SignedAgreement<
//         //     pallet_robonomics_liability::technics::IPFS,
//         //     pallet_robonomics_liability::economics::SimpleMarket<Self::AccountId, Balances>,
//         //     Self::AccountId,
//         //     sp_runtime::MultiSignature,
//         // >;
//
//         let agreement = SignedAgreement {
//             technics,
//             economics,
//             // promisee: sender.clone(),
//             // promisor: sender.clone(),
//             promisee: caller.clone(),
//             promisor: caller.clone(),
//             promisee_signature: signature.clone(),
//             promisor_signature: signature.clone(),
//         };
//
//         #[extrinsic_call]
//         _(RawOrigin::Signed(caller), agreement);
//     }
//
//     // #[benchmark]
//     // fn finalize() {
//     //     let caller: T::AccountId = whitelisted_caller();
//     //
//     //     let technics = IPFS {
//     //         hash: IPFS_HASH.into(),
//     //     };
//     //     let economics = SimpleMarket { price: 10 };
//     //     let pair = sr25519::Pair::from_string("//Alice", None).unwrap();
//     //     let sender = <MultiSignature as Verify>::Signer::from(pair.public()).into_account();
//     //
//     //     let signature: MultiSignature = (technics, economics)
//     //         .using_encoded(|params| AppCrypto::sign(params, sender.clone()))
//     //         .expect("unable to sign using runtime application key");
//     //
//     //     let index = 0u32;
//     //     let payload = IPFS {
//     //         hash: IPFS_HASH.into(),
//     //     };
//     //
//     //     let report: T::Report = SignedReport {
//     //         index,
//     //         sender,
//     //         payload,
//     //         signature,
//     //     };
//     //
//     //     #[extrinsic_call]
//     //     _(RawOrigin::Signed(caller), report);
//     // }
//
//     impl_benchmark_test_suite!(
//         Liability,
//         crate::tests::new_test_ext(),
//         crate::tests::Runtime,
//     );
// }
