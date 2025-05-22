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
// Benchmarks for Datalog Pallet

#![cfg(feature = "runtime-benchmarks")]

use super::{Pallet as Rws, *};
use frame_benchmarking::v2::*;
use frame_support::{
    pallet_prelude::{Get, MaxEncodedLen},
    traits::Currency,
};
use frame_system::RawOrigin;
use parity_scale_codec::{Decode, Encode};
use sp_std::prelude::*;

use crate::*;

const SEED: u32 = 0;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
    let caller: T::AccountId = account(name, index, SEED);
    T::AuctionCurrency::make_free_balance_be(&caller, T::MinimalBid::get() * 100u32.into());
    caller
}

#[benchmarks]
mod benchmarks {
    use super::*;
    #[cfg(test)]
    use frame_system::RawOrigin;

    // ???
    // #[benchmark]
    // fn call() {}

    #[benchmark]
    fn bid() {
        let caller = funded_account::<T>("caller", 0);
        Pallet::<T>::new_auction(Default::default());
        let queue = Pallet::<T>::auction_queue();
        let index = queue.first().unwrap();
        let amount = T::MinimalBid::get() * 10u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), index.clone(), amount);
    }

    #[benchmark]
    fn set_devices() {
        let caller: T::AccountId = whitelisted_caller();
        let device: T::AccountId = account("device", 1, SEED);
        let mut devices = frame_support::BoundedVec::new();
        let _ = devices.try_push(device);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), devices);
    }

    // #[benchmark]
    // fn set_oracle() {}

    // #[benchmark]
    // fn set_subscription() {
    //     let caller: T::AccountId = account("caller", 1, SEED);
    //     let oracle: T::AccountId = account("caller", 2, SEED);
    //
    //     // let device: T::AccountId = account("device", 2, SEED);
    //     // let mut devices = frame_support::BoundedVec::new();
    //     // let _ = devices.try_push(device);
    //
    //     #[extrinsic_call]
    //     set_subscription(RawOrigin::Signed(caller), caller, Default::default());
    //
    //     // let oracle = CHARLIE;
    //     // new_test_ext().execute_with(|| {
    //     //     assert_ok!(RWS::set_oracle(Origin::root(), oracle));
    //     //
    //     //     assert_err!(
    //     //         RWS::set_subscription(Origin::none(), ALICE, Default::default()),
    //     //         DispatchError::BadOrigin,
    //     //     );
    //     //
    //     //     assert_err!(
    //     //         RWS::set_subscription(Origin::signed(ALICE), ALICE, Default::default()),
    //     //         Error::<Runtime>::OracleOnlyCall,
    //     //     );
    //     //
    //     //     let lifetime = Subscription::Lifetime { tps: 10 };
    //     //     assert_ok!(RWS::set_subscription(
    //     //         Origin::signed(oracle),
    //     //         ALICE,
    //     //         lifetime.clone(),
    //     //     ));
    //     //     assert_eq!(RWS::ledger(ALICE).unwrap().kind, lifetime);
    //     // })
    // }

    // #[benchmark]
    // fn start_auction() {}

    impl_benchmark_test_suite!(Rws, crate::tests::new_test_ext(), crate::tests::Runtime,);
}
