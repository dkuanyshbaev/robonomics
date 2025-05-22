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

// #[benchmarks]
// mod benchmarks {
//     use super::*;
//     #[cfg(test)]
//     use frame_system::RawOrigin;
//
//     #[benchmark]
//     fn create() -> Result<(), BenchmarkError> {
//         Ok(())
//     }
//
//     #[benchmark]
//     fn finalize() -> Result<(), BenchmarkError> {
//         Ok(())
//     }
//
//     impl_benchmark_test_suite!(
//         Liability,
//         crate::tests::new_test_ext(),
//         crate::tests::Runtime,
//     );
// }
