// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Assets pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use sp_std::prelude::*;
use super::*;
use sp_runtime::traits::Bounded;
use frame_system::RawOrigin as SystemOrigin;
use frame_benchmarking::{
	benchmarks_instance_pallet, account, whitelisted_caller, whitelist_account, impl_benchmark_test_suite
};
use frame_support::{traits::{Get, EnsureOrigin}, dispatch::UnfilteredDispatchable};

use crate::Pallet as Uniques;

const SEED: u32 = 0;

fn create_class<T: Config<I>, I: 'static>()
	-> (T::ClassId, T::AccountId, <T::Lookup as StaticLookup>::Source)
{
	let caller: T::AccountId = whitelisted_caller();
	let caller_lookup = T::Lookup::unlookup(caller.clone());
	let class = Default::default();
	T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());
	assert!(Uniques::<T, I>::create(
		SystemOrigin::Signed(caller.clone()).into(),
		class,
		caller_lookup.clone(),
	).is_ok());
	(class, caller, caller_lookup)
}

fn add_class_metadata<T: Config<I>, I: 'static>()
	-> (T::AccountId, <T::Lookup as StaticLookup>::Source)
{
	let caller = Class::<T, I>::get(T::ClassId::default()).unwrap().owner;
	if caller != whitelisted_caller() {
		whitelist_account!(caller);
	}
	let caller_lookup = T::Lookup::unlookup(caller.clone());
	assert!(Uniques::<T, I>::set_class_metadata(
		SystemOrigin::Signed(caller.clone()).into(),
		Default::default(),
		vec![0; T::StringLimit::get() as usize],
		vec![0; T::StringLimit::get() as usize],
		false,
	).is_ok());
	(caller, caller_lookup)
}

fn mint_instance<T: Config<I>, I: 'static>(index: u16)
	-> (T::InstanceId, T::AccountId, <T::Lookup as StaticLookup>::Source)
{
	let caller = Class::<T, I>::get(T::ClassId::default()).unwrap().admin;
	if caller != whitelisted_caller() {
		whitelist_account!(caller);
	}
	let caller_lookup = T::Lookup::unlookup(caller.clone());
	let instance = index.into();
	assert!(Uniques::<T, I>::mint(
		SystemOrigin::Signed(caller.clone()).into(),
		Default::default(),
		instance,
		caller_lookup.clone(),
	).is_ok());
	(instance, caller, caller_lookup)
}

fn add_instance_metadata<T: Config<I>, I: 'static>(instance: T::InstanceId)
	-> (T::AccountId, <T::Lookup as StaticLookup>::Source)
{
	let caller = Class::<T, I>::get(T::ClassId::default()).unwrap().owner;
	if caller != whitelisted_caller() {
		whitelist_account!(caller);
	}
	let caller_lookup = T::Lookup::unlookup(caller.clone());
	assert!(Uniques::<T, I>::set_metadata(
		SystemOrigin::Signed(caller.clone()).into(),
		Default::default(),
		instance,
		vec![0; T::StringLimit::get() as usize],
		vec![0; T::StringLimit::get() as usize],
		false,
	).is_ok());
	(caller, caller_lookup)
}

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let frame_system::EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}
/*
fn assert_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	let events = frame_system::Pallet::<T>::events();
	assert!(events.iter().any(|event_record| {
		matches!(&event_record, frame_system::EventRecord { event, .. } if &system_event == event)
	}));
}
*/
benchmarks_instance_pallet! {
	create {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());
	}: _(SystemOrigin::Signed(caller.clone()), Default::default(), caller_lookup)
	verify {
		assert_last_event::<T, I>(Event::Created(Default::default(), caller.clone(), caller).into());
	}

	force_create {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
	}: _(SystemOrigin::Root, Default::default(), caller_lookup, true)
	verify {
		assert_last_event::<T, I>(Event::ForceCreated(Default::default(), caller).into());
	}

	destroy {
		let n in 0 .. 5_000;
		let m in 0 .. 5_000;
		let a in 0 .. 5_000;

		let (class, caller, caller_lookup) = create_class::<T, I>();
		add_class_metadata::<T, I>();
		for i in 0..n + m {
			// create instance
			let (instance, ..) = mint_instance::<T, I>(i as u16);
			if i < m {
				// add metadata
				add_instance_metadata::<T, I>(instance);
			}
		}
		for i in 0..a {
			assert!(Uniques::<T, I>::set_attribute(
				SystemOrigin::Signed(caller.clone()).into(),
				class,
				Some((i as u16).into()),
				vec![0; T::StringLimit::get() as usize],
				Some(vec![0; T::StringLimit::get() as usize]),
			).is_ok());
		}
		let witness = Class::<T, I>::get(class).unwrap().destroy_witness();
	}: _(SystemOrigin::Signed(caller), class, witness)
	verify {
		assert_last_event::<T, I>(Event::Destroyed(class).into());
	}

	mint {
		let (class, caller, caller_lookup) = create_class::<T, I>();
		let instance = Default::default();
	}: _(SystemOrigin::Signed(caller.clone()), class, instance, caller_lookup)
	verify {
		assert_last_event::<T, I>(Event::Issued(class, instance, caller).into());
	}

	burn {
		let (class, caller, caller_lookup) = create_class::<T, I>();
		let (instance, ..) = mint_instance::<T, I>(0);
		add_instance_metadata::<T, I>(instance);
	}: _(SystemOrigin::Signed(caller.clone()), class, instance, Some(caller_lookup))
	verify {
		assert_last_event::<T, I>(Event::Burned(class, instance, caller).into());
	}

	transfer {
		let (class, caller, caller_lookup) = create_class::<T, I>();
		let (instance, ..) = mint_instance::<T, I>(Default::default());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());
	}: _(SystemOrigin::Signed(caller.clone()), class, instance, target_lookup)
	verify {
		assert_last_event::<T, I>(Event::Transferred(class, instance, caller, target).into());
	}

	redeposit {
		let i in 0 .. 5_000;
		let (class, caller, caller_lookup) = create_class::<T, I>();
		let instances = (0..i).map(|x| mint_instance::<T, I>(x as u16).0).collect::<Vec<_>>();
		Uniques::<T, I>::force_asset_status(
			SystemOrigin::Root.into(),
			class,
			caller_lookup.clone(),
			caller_lookup.clone(),
			caller_lookup.clone(),
			caller_lookup.clone(),
			true,
			false,
		)?;
	}: _(SystemOrigin::Signed(caller.clone()), class, instances.clone())
	verify {
		assert_last_event::<T, I>(Event::Redeposited(class, instances).into());
	}

	freeze {
		let (class, caller, caller_lookup) = create_class::<T, I>();
		let (instance, ..) = mint_instance::<T, I>(Default::default());
	}: _(SystemOrigin::Signed(caller.clone()), Default::default(), Default::default())
	verify {
		assert_last_event::<T, I>(Event::Frozen(Default::default(), Default::default()).into());
	}

	thaw {
		let (class, caller, caller_lookup) = create_class::<T, I>();
		let (instance, ..) = mint_instance::<T, I>(Default::default());
		Uniques::<T, I>::freeze(
			SystemOrigin::Signed(caller.clone()).into(),
			class,
			instance,
		)?;
	}: _(SystemOrigin::Signed(caller.clone()), class, instance)
	verify {
		assert_last_event::<T, I>(Event::Thawed(class, instance).into());
	}

	freeze_class {
		let (class, caller, caller_lookup) = create_class::<T, I>();
	}: _(SystemOrigin::Signed(caller.clone()), class)
	verify {
		assert_last_event::<T, I>(Event::ClassFrozen(class).into());
	}

	thaw_class {
		let (class, caller, caller_lookup) = create_class::<T, I>();
		let origin = SystemOrigin::Signed(caller.clone()).into();
		Uniques::<T, I>::freeze_class(origin, class)?;
	}: _(SystemOrigin::Signed(caller.clone()), class)
	verify {
		assert_last_event::<T, I>(Event::ClassThawed(class).into());
	}

	transfer_ownership {
		let (class, caller, _) = create_class::<T, I>();
		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());
		T::Currency::make_free_balance_be(&target, T::Currency::minimum_balance());
	}: _(SystemOrigin::Signed(caller), class, target_lookup)
	verify {
		assert_last_event::<T, I>(Event::OwnerChanged(class, target).into());
	}

	set_team {
		let (class, caller, _) = create_class::<T, I>();
		let target0 = T::Lookup::unlookup(account("target", 0, SEED));
		let target1 = T::Lookup::unlookup(account("target", 1, SEED));
		let target2 = T::Lookup::unlookup(account("target", 2, SEED));
	}: _(SystemOrigin::Signed(caller), Default::default(), target0.clone(), target1.clone(), target2.clone())
	verify {
		assert_last_event::<T, I>(Event::TeamChanged(
			class,
			account("target", 0, SEED),
			account("target", 1, SEED),
			account("target", 2, SEED),
		).into());
	}

	force_asset_status {
		let (class, caller, caller_lookup) = create_class::<T, I>();
		let origin = T::ForceOrigin::successful_origin();
		let call = Call::<T, I>::force_asset_status(
			class,
			caller_lookup.clone(),
			caller_lookup.clone(),
			caller_lookup.clone(),
			caller_lookup.clone(),
			true,
			false,
		);
	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_last_event::<T, I>(Event::AssetStatusChanged(class).into());
	}

	set_attribute {
		let k in 0 .. T::StringLimit::get();
		let v in 0 .. T::StringLimit::get();

		let key = vec![0u8; k as usize];
		let value = vec![0u8; v as usize];

		let (class, caller, _) = create_class::<T, I>();
		let (instance, ..) = mint_instance::<T, I>(0);
	}: _(SystemOrigin::Signed(caller), class, Some(instance), key.clone(), Some(value.clone()))
	verify {
		assert_last_event::<T, I>(Event::AttributeSet(class, Some(instance), key, Some(value)).into());
	}

	set_metadata {
		let n in 0 .. T::StringLimit::get();
		let i in 0 .. T::StringLimit::get();

		let name = vec![0u8; n as usize];
		let info = vec![0u8; i as usize];

		let (class, caller, _) = create_class::<T, I>();
		let (instance, ..) = mint_instance::<T, I>(0);
	}: _(SystemOrigin::Signed(caller), class, instance, name.clone(), info.clone(), false)
	verify {
		assert_last_event::<T, I>(Event::MetadataSet(class, instance, name, info, false).into());
	}

	clear_metadata {
		let (class, caller, _) = create_class::<T, I>();
		let (instance, ..) = mint_instance::<T, I>(0);
		add_instance_metadata::<T, I>(instance);
	}: _(SystemOrigin::Signed(caller), class, instance)
	verify {
		assert_last_event::<T, I>(Event::MetadataCleared(class, instance).into());
	}

	set_class_metadata {
		let n in 0 .. T::StringLimit::get();
		let i in 0 .. T::StringLimit::get();

		let name = vec![0u8; n as usize];
		let info = vec![0u8; i as usize];

		let (class, caller, _) = create_class::<T, I>();
	}: _(SystemOrigin::Signed(caller), class, name.clone(), info.clone(), false)
	verify {
		assert_last_event::<T, I>(Event::ClassMetadataSet(class, name, info, false).into());
	}

	clear_class_metadata {
		let (class, caller, _) = create_class::<T, I>();
		add_class_metadata::<T, I>();
	}: _(SystemOrigin::Signed(caller), class)
	verify {
		assert_last_event::<T, I>(Event::ClassMetadataCleared(class).into());
	}

	approve_transfer {
		let (class, caller, _) = create_class::<T, I>();
		let (instance, ..) = mint_instance::<T, I>(0);
		let delegate: T::AccountId = account("delegate", 0, SEED);
		let delegate_lookup = T::Lookup::unlookup(delegate.clone());
	}: _(SystemOrigin::Signed(caller.clone()), class, instance, delegate_lookup)
	verify {
		assert_last_event::<T, I>(Event::ApprovedTransfer(class, instance, caller, delegate).into());
	}

	cancel_approval {
		let (class, caller, _) = create_class::<T, I>();
		let (instance, ..) = mint_instance::<T, I>(0);
		let delegate: T::AccountId = account("delegate", 0, SEED);
		let delegate_lookup = T::Lookup::unlookup(delegate.clone());
		let origin = SystemOrigin::Signed(caller.clone()).into();
		Uniques::<T, I>::approve_transfer(origin, class, instance, delegate_lookup.clone())?;
	}: _(SystemOrigin::Signed(caller.clone()), class, instance, Some(delegate_lookup))
	verify {
		assert_last_event::<T, I>(Event::ApprovalCancelled(class, instance, caller, delegate).into());
	}
}

impl_benchmark_test_suite!(Uniques, crate::mock::new_test_ext(), crate::mock::Test);
