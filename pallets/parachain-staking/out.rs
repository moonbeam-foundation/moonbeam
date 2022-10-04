#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks {
    #![cfg(feature = "runtime-benchmarks")]
    //! Benchmarking
    use crate::{
        AwardedPts, BalanceOf, Call, CandidateBondLessRequest, CandidateInfo, Config,
        DelegationAction, Pallet, Points, Range, Round, ScheduledRequest, COLLATOR_LOCK_ID,
    };
    use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec};
    use frame_support::traits::{
        Currency, Get, LockIdentifier, OnFinalize, OnInitialize, ReservableCurrency,
    };
    use frame_system::RawOrigin;
    use pallet_balances::pallet::Pallet as PalletBalances;
    use parity_scale_codec::EncodeLike;
    use sp_runtime::{Perbill, Percent};
    use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
    fn get_lock_amount<T: Config>(
        account_id: T::AccountId,
        id: LockIdentifier,
    ) -> Option<<T as pallet_balances::Config>::Balance> {
        for lock in <PalletBalances<T>>::locks(account_id) {
            if lock.id == id {
                return Some(lock.amount);
            }
        }
        None
    }
    /// Minimum collator candidate stake
    fn min_candidate_stk<T: Config>() -> BalanceOf<T> {
        <<T as Config>::MinCollatorStk as Get<BalanceOf<T>>>::get()
    }
    /// Minimum delegator stake
    fn min_delegator_stk<T: Config>() -> BalanceOf<T> {
        <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get()
    }
    /// Create a funded user.
    /// Extra + min_candidate_stk is total minted funds
    /// Returns tuple (id, balance)
    fn create_funded_user<T: Config>(
        string: &'static str,
        n: u32,
        extra: BalanceOf<T>,
    ) -> (T::AccountId, BalanceOf<T>) {
        const SEED: u32 = 0;
        let user = account(string, n, SEED);
        let min_candidate_stk = min_candidate_stk::<T>();
        let total = min_candidate_stk + extra;
        T::Currency::make_free_balance_be(&user, total);
        T::Currency::issue(total);
        (user, total)
    }
    /// Create a funded delegator.
    fn create_funded_delegator<T: Config>(
        string: &'static str,
        n: u32,
        extra: BalanceOf<T>,
        collator: T::AccountId,
        min_bond: bool,
        collator_delegator_count: u32,
    ) -> Result<T::AccountId, &'static str> {
        let (user, total) = create_funded_user::<T>(string, n, extra);
        let bond = if min_bond {
            min_delegator_stk::<T>()
        } else {
            total
        };
        Pallet::<T>::delegate(
            RawOrigin::Signed(user.clone()).into(),
            collator,
            bond,
            collator_delegator_count,
            0u32,
        )?;
        Ok(user)
    }
    /// Create a funded collator.
    fn create_funded_collator<T: Config>(
        string: &'static str,
        n: u32,
        extra: BalanceOf<T>,
        min_bond: bool,
        candidate_count: u32,
    ) -> Result<T::AccountId, &'static str> {
        let (user, total) = create_funded_user::<T>(string, n, extra);
        let bond = if min_bond {
            min_candidate_stk::<T>()
        } else {
            total
        };
        Pallet::<T>::join_candidates(
            RawOrigin::Signed(user.clone()).into(),
            bond,
            candidate_count,
        )?;
        Ok(user)
    }
    fn parachain_staking_on_finalize<T: Config>(author: T::AccountId) {
        let now = <Round<T>>::get().current;
        let score_plus_20 = <AwardedPts<T>>::get(now, &author).saturating_add(20);
        <AwardedPts<T>>::insert(now, author, score_plus_20);
        <Points<T>>::mutate(now, |x| *x = x.saturating_add(20));
    }
    /// Run to end block and author
    fn roll_to_and_author<T: Config>(round_delay: u32, author: T::AccountId) {
        let total_rounds = round_delay + 1u32;
        let round_length: T::BlockNumber = Pallet::<T>::round().length.into();
        let mut now = <frame_system::Pallet<T>>::block_number() + 1u32.into();
        let end = Pallet::<T>::round().first + (round_length * total_rounds.into());
        while now < end {
            parachain_staking_on_finalize::<T>(author.clone());
            <frame_system::Pallet<T>>::on_finalize(<frame_system::Pallet<T>>::block_number());
            <frame_system::Pallet<T>>::set_block_number(
                <frame_system::Pallet<T>>::block_number() + 1u32.into(),
            );
            <frame_system::Pallet<T>>::on_initialize(<frame_system::Pallet<T>>::block_number());
            Pallet::<T>::on_initialize(<frame_system::Pallet<T>>::block_number());
            now += 1u32.into();
        }
    }
    const USER_SEED: u32 = 999666;
    struct Seed {
        pub inner: u32,
    }
    impl Seed {
        fn new() -> Self {
            Seed { inner: USER_SEED }
        }
        pub fn take(&mut self) -> u32 {
            let v = self.inner;
            self.inner += 1;
            v
        }
    }
    #[allow(non_camel_case_types)]
    struct set_staking_expectations;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for set_staking_expectations {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let stake_range: Range<BalanceOf<T>> = Range {
                min: 100u32.into(),
                ideal: 200u32.into(),
                max: 300u32.into(),
            };
            let __call: _ = Call::<T>::new_call_variant_set_staking_expectations(stake_range);
            let __benchmarked_call_encoded: _ =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Root.into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            match (&Pallet::<T>::inflation_config().expect, &stake_range) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct set_inflation;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for set_inflation {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let inflation_range: Range<Perbill> = Range {
                min: Perbill::from_perthousand(1),
                ideal: Perbill::from_perthousand(2),
                max: Perbill::from_perthousand(3),
            };
            let __call: _ = Call::<T>::new_call_variant_set_inflation(inflation_range);
            let __benchmarked_call_encoded: _ =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Root.into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            match (&Pallet::<T>::inflation_config().annual, &inflation_range) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct set_parachain_bond_account;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for set_parachain_bond_account {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let parachain_bond_account: T::AccountId = account("TEST", 0u32, USER_SEED);
            let __call: _ = Call::<T>::new_call_variant_set_parachain_bond_account(
                parachain_bond_account.clone(),
            );
            let __benchmarked_call_encoded: _ =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Root.into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            match (
                                &Pallet::<T>::parachain_bond_info().account,
                                &parachain_bond_account,
                            ) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct set_parachain_bond_reserve_percent;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for set_parachain_bond_reserve_percent {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let __call: _ = Call::<T>::new_call_variant_set_parachain_bond_reserve_percent(
                Percent::from_percent(33),
            );
            let __benchmarked_call_encoded: _ =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Root.into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            match (
                                &Pallet::<T>::parachain_bond_info().percent,
                                &Percent::from_percent(33),
                            ) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct set_total_selected;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for set_total_selected {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            Pallet::<T>::set_blocks_per_round(RawOrigin::Root.into(), 100u32)?;
            let __call = Call::<T>::new_call_variant_set_total_selected(100u32);
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Root.into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            match (&Pallet::<T>::total_selected(), &100u32) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct set_collator_commission;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for set_collator_commission {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let __call: _ =
                Call::<T>::new_call_variant_set_collator_commission(Perbill::from_percent(33));
            let __benchmarked_call_encoded: _ =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Root.into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            match (
                                &Pallet::<T>::collator_commission(),
                                &Perbill::from_percent(33),
                            ) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct set_blocks_per_round;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for set_blocks_per_round {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let __call: _ = Call::<T>::new_call_variant_set_blocks_per_round(1200u32);
            let __benchmarked_call_encoded: _ =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Root.into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            match (&Pallet::<T>::round().length, &1200u32) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct join_candidates;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for join_candidates {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            <[_]>::into_vec(box [(::frame_benchmarking::BenchmarkParameter::x, 3, 1_000)])
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let x = components
                .iter()
                .find(|&c| c.0 == ::frame_benchmarking::BenchmarkParameter::x)
                .ok_or("Could not find component in benchmark preparation.")?
                .1;
            ();
            let mut candidate_count = 1u32;
            for i in 2..x {
                let seed = USER_SEED - i;
                let collator = create_funded_collator::<T>(
                    "collator",
                    seed,
                    0u32.into(),
                    true,
                    candidate_count,
                )?;
                candidate_count += 1u32;
            }
            let (caller, min_candidate_stk) =
                create_funded_user::<T>("caller", USER_SEED, 0u32.into());
            let __call =
                Call::<T>::new_call_variant_join_candidates(min_candidate_stk, candidate_count);
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            if !Pallet::<T>::is_candidate(&caller) {
                                ::core::panicking::panic(
                                    "assertion failed: Pallet::<T>::is_candidate(&caller)",
                                )
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct schedule_leave_candidates;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for schedule_leave_candidates {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            <[_]>::into_vec(box [(::frame_benchmarking::BenchmarkParameter::x, 3, 1_000)])
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let x = components
                .iter()
                .find(|&c| c.0 == ::frame_benchmarking::BenchmarkParameter::x)
                .ok_or("Could not find component in benchmark preparation.")?
                .1;
            ();
            let mut candidate_count = 1u32;
            for i in 2..x {
                let seed = USER_SEED - i;
                let collator = create_funded_collator::<T>(
                    "collator",
                    seed,
                    0u32.into(),
                    true,
                    candidate_count,
                )?;
                candidate_count += 1u32;
            }
            let caller: T::AccountId = create_funded_collator::<T>(
                "caller",
                USER_SEED,
                0u32.into(),
                true,
                candidate_count,
            )?;
            candidate_count += 1u32;
            let __call = Call::<T>::new_call_variant_schedule_leave_candidates(candidate_count);
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            if !Pallet::<T>::candidate_info(&caller).unwrap().is_leaving() {
                                :: core :: panicking :: panic ("assertion failed: Pallet::<T>::candidate_info(&caller).unwrap().is_leaving()")
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct execute_leave_candidates;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for execute_leave_candidates {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            <[_]>::into_vec(box [(
                ::frame_benchmarking::BenchmarkParameter::x,
                2,
                (<<T as Config>::MaxTopDelegationsPerCandidate as Get<u32>>::get()
                    + <<T as Config>::MaxBottomDelegationsPerCandidate as Get<u32>>::get()),
            )])
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let x = components
                .iter()
                .find(|&c| c.0 == ::frame_benchmarking::BenchmarkParameter::x)
                .ok_or("Could not find component in benchmark preparation.")?
                .1;
            ();
            let candidate: T::AccountId = create_funded_collator::<T>(
                "unique_caller",
                USER_SEED - 100,
                0u32.into(),
                true,
                1u32,
            )?;
            let second_candidate: T::AccountId = create_funded_collator::<T>(
                "unique__caller",
                USER_SEED - 99,
                0u32.into(),
                true,
                2u32,
            )?;
            let mut delegators: Vec<T::AccountId> = Vec::new();
            let mut col_del_count = 0u32;
            for i in 1..x {
                let seed = USER_SEED + i;
                let delegator = create_funded_delegator::<T>(
                    "delegator",
                    seed,
                    min_delegator_stk::<T>(),
                    candidate.clone(),
                    true,
                    col_del_count,
                )?;
                Pallet::<T>::delegate(
                    RawOrigin::Signed(delegator.clone()).into(),
                    second_candidate.clone(),
                    min_delegator_stk::<T>(),
                    col_del_count,
                    1u32,
                )?;
                Pallet::<T>::schedule_revoke_delegation(
                    RawOrigin::Signed(delegator.clone()).into(),
                    candidate.clone(),
                )?;
                delegators.push(delegator);
                col_del_count += 1u32;
            }
            Pallet::<T>::schedule_leave_candidates(
                RawOrigin::Signed(candidate.clone()).into(),
                3u32,
            )?;
            roll_to_and_author::<T>(2, candidate.clone());
            let __call = Call::<T>::new_call_variant_execute_leave_candidates(
                candidate.clone(),
                col_del_count,
            );
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(candidate.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            if !Pallet::<T>::candidate_info(&candidate).is_none() {
                                :: core :: panicking :: panic ("assertion failed: Pallet::<T>::candidate_info(&candidate).is_none()")
                            };
                            if !Pallet::<T>::candidate_info(&second_candidate).is_some() {
                                :: core :: panicking :: panic ("assertion failed: Pallet::<T>::candidate_info(&second_candidate).is_some()")
                            };
                            for delegator in delegators {
                                if !Pallet::<T>::is_delegator(&delegator) {
                                    ::core::panicking::panic(
                                        "assertion failed: Pallet::<T>::is_delegator(&delegator)",
                                    )
                                };
                            }
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct cancel_leave_candidates;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for cancel_leave_candidates {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            <[_]>::into_vec(box [(::frame_benchmarking::BenchmarkParameter::x, 3, 1_000)])
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let x = components
                .iter()
                .find(|&c| c.0 == ::frame_benchmarking::BenchmarkParameter::x)
                .ok_or("Could not find component in benchmark preparation.")?
                .1;
            ();
            let mut candidate_count = 1u32;
            for i in 2..x {
                let seed = USER_SEED - i;
                let collator = create_funded_collator::<T>(
                    "collator",
                    seed,
                    0u32.into(),
                    true,
                    candidate_count,
                )?;
                candidate_count += 1u32;
            }
            let caller: T::AccountId = create_funded_collator::<T>(
                "caller",
                USER_SEED,
                0u32.into(),
                true,
                candidate_count,
            )?;
            candidate_count += 1u32;
            Pallet::<T>::schedule_leave_candidates(
                RawOrigin::Signed(caller.clone()).into(),
                candidate_count,
            )?;
            candidate_count -= 1u32;
            let __call = Call::<T>::new_call_variant_cancel_leave_candidates(candidate_count);
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            if !Pallet::<T>::candidate_info(&caller).unwrap().is_active() {
                                :: core :: panicking :: panic ("assertion failed: Pallet::<T>::candidate_info(&caller).unwrap().is_active()")
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct go_offline;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for go_offline {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let caller: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, 0u32.into(), true, 1u32)?;
            let __call: _ = Call::<T>::new_call_variant_go_offline();
            let __benchmarked_call_encoded: _ =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            if !!Pallet::<T>::candidate_info(&caller).unwrap().is_active() {
                                :: core :: panicking :: panic ("assertion failed: !Pallet::<T>::candidate_info(&caller).unwrap().is_active()")
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct go_online;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for go_online {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let caller: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, 0u32.into(), true, 1u32)?;
            Pallet::<T>::go_offline(RawOrigin::Signed(caller.clone()).into())?;
            let __call = Call::<T>::new_call_variant_go_online();
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            if !Pallet::<T>::candidate_info(&caller).unwrap().is_active() {
                                :: core :: panicking :: panic ("assertion failed: Pallet::<T>::candidate_info(&caller).unwrap().is_active()")
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct candidate_bond_more;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for candidate_bond_more {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let more: _ = min_candidate_stk::<T>();
            let caller: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, more, true, 1u32)?;
            {
                let lvl = ::log::Level::Info;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api_log(
                        ::core::fmt::Arguments::new_v1(
                            &[""],
                            &[::core::fmt::ArgumentV1::new_debug(
                                &<CandidateInfo<T>>::get(&caller).unwrap(),
                            )],
                        ),
                        lvl,
                        &(
                            "pallet_parachain_staking::benchmarks",
                            "pallet_parachain_staking::benchmarks",
                            "pallets/parachain-staking/src/benchmarks.rs",
                            414u32,
                        ),
                        ::log::__private_api::Option::None,
                    );
                }
            };
            let __call = Call::<T>::new_call_variant_candidate_bond_more(more);
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            let expected_bond = Some(more * 2u32.into());
                            {
                                let lvl = ::log::Level::Info;
                                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                    ::log::__private_api_log(
                                        ::core::fmt::Arguments::new_v1(
                                            &[""],
                                            &[::core::fmt::ArgumentV1::new_debug(
                                                &<CandidateInfo<T>>::get(&caller).unwrap(),
                                            )],
                                        ),
                                        lvl,
                                        &(
                                            "pallet_parachain_staking::benchmarks",
                                            "pallet_parachain_staking::benchmarks",
                                            "pallets/parachain-staking/src/benchmarks.rs",
                                            418u32,
                                        ),
                                        ::log::__private_api::Option::None,
                                    );
                                }
                            };
                            match (
                                &get_lock_amount::<T>(caller, COLLATOR_LOCK_ID),
                                &expected_bond,
                            ) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct schedule_candidate_bond_less;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for schedule_candidate_bond_less {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let min_candidate_stk: _ = min_candidate_stk::<T>();
            let caller: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, min_candidate_stk, false, 1u32)?;
            let __call: _ =
                Call::<T>::new_call_variant_schedule_candidate_bond_less(min_candidate_stk);
            let __benchmarked_call_encoded: _ =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            let state = Pallet::<T>::candidate_info(&caller)
                                .expect("request bonded less so exists");
                            match (
                                &state.request,
                                &Some(CandidateBondLessRequest {
                                    amount: min_candidate_stk,
                                    when_executable: 3,
                                }),
                            ) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct execute_candidate_bond_less;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for execute_candidate_bond_less {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let min_candidate_stk: _ = min_candidate_stk::<T>();
            let caller: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, min_candidate_stk, false, 1u32)?;
            Pallet::<T>::schedule_candidate_bond_less(
                RawOrigin::Signed(caller.clone()).into(),
                min_candidate_stk,
            )?;
            roll_to_and_author::<T>(2, caller.clone());
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        Pallet::<T>::execute_candidate_bond_less(
                            RawOrigin::Signed(caller.clone()).into(),
                            caller.clone(),
                        )?;
                    };
                    if verify {
                        {
                            match (&T::Currency::reserved_balance(&caller), &min_candidate_stk) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct cancel_candidate_bond_less;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for cancel_candidate_bond_less {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let min_candidate_stk: _ = min_candidate_stk::<T>();
            let caller: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, min_candidate_stk, false, 1u32)?;
            Pallet::<T>::schedule_candidate_bond_less(
                RawOrigin::Signed(caller.clone()).into(),
                min_candidate_stk,
            )?;
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        Pallet::<T>::cancel_candidate_bond_less(
                            RawOrigin::Signed(caller.clone()).into(),
                        )?;
                    };
                    if verify {
                        {
                            if !Pallet::<T>::candidate_info(&caller)
                                .unwrap()
                                .request
                                .is_none()
                            {
                                :: core :: panicking :: panic ("assertion failed: Pallet::<T>::candidate_info(&caller).unwrap().request.is_none()")
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct delegate;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for delegate {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            <[_]>::into_vec(box [
                (
                    ::frame_benchmarking::BenchmarkParameter::x,
                    3,
                    <<T as Config>::MaxDelegationsPerDelegator as Get<u32>>::get(),
                ),
                (
                    ::frame_benchmarking::BenchmarkParameter::y,
                    2,
                    <<T as Config>::MaxTopDelegationsPerCandidate as Get<u32>>::get(),
                ),
            ])
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let x = components
                .iter()
                .find(|&c| c.0 == ::frame_benchmarking::BenchmarkParameter::x)
                .ok_or("Could not find component in benchmark preparation.")?
                .1;
            let y = components
                .iter()
                .find(|&c| c.0 == ::frame_benchmarking::BenchmarkParameter::y)
                .ok_or("Could not find component in benchmark preparation.")?
                .1;
            ();
            ();
            let mut collators: Vec<T::AccountId> = Vec::new();
            for i in 2..x {
                let seed = USER_SEED - i;
                let collator = create_funded_collator::<T>(
                    "collator",
                    seed,
                    0u32.into(),
                    true,
                    collators.len() as u32 + 1u32,
                )?;
                collators.push(collator.clone());
            }
            let bond = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
            let extra =
                if (bond * (collators.len() as u32 + 1u32).into()) > min_candidate_stk::<T>() {
                    (bond * (collators.len() as u32 + 1u32).into()) - min_candidate_stk::<T>()
                } else {
                    0u32.into()
                };
            let (caller, _) = create_funded_user::<T>("caller", USER_SEED, extra.into());
            let mut del_del_count = 0u32;
            for col in collators.clone() {
                Pallet::<T>::delegate(
                    RawOrigin::Signed(caller.clone()).into(),
                    col,
                    bond,
                    0u32,
                    del_del_count,
                )?;
                del_del_count += 1u32;
            }
            let collator: T::AccountId = create_funded_collator::<T>(
                "collator",
                USER_SEED,
                0u32.into(),
                true,
                collators.len() as u32 + 1u32,
            )?;
            let mut col_del_count = 0u32;
            for i in 1..y {
                let seed = USER_SEED + i;
                let _ = create_funded_delegator::<T>(
                    "delegator",
                    seed,
                    0u32.into(),
                    collator.clone(),
                    true,
                    col_del_count,
                )?;
                col_del_count += 1u32;
            }
            let __call =
                Call::<T>::new_call_variant_delegate(collator, bond, col_del_count, del_del_count);
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            if !Pallet::<T>::is_delegator(&caller) {
                                ::core::panicking::panic(
                                    "assertion failed: Pallet::<T>::is_delegator(&caller)",
                                )
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct schedule_leave_delegators;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for schedule_leave_delegators {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let collator: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, 0u32.into(), true, 1u32)?;
            let (caller, _): _ = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
            let bond: _ = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
            Pallet::<T>::delegate(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
                bond,
                0u32,
                0u32,
            )?;
            let __call = Call::<T>::new_call_variant_schedule_leave_delegators();
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            if !Pallet::<T>::delegation_scheduled_requests(&collator)
                                .iter()
                                .any(|r| {
                                    r.delegator == caller
                                        && match r.action {
                                            DelegationAction::Revoke(_) => true,
                                            _ => false,
                                        }
                                })
                            {
                                :: core :: panicking :: panic ("assertion failed: Pallet::<T>::delegation_scheduled_requests(&collator).iter().any(|r|\\n        r.delegator == caller &&\\n            matches!(r.action, DelegationAction :: Revoke(_)))")
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct execute_leave_delegators;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for execute_leave_delegators {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            <[_]>::into_vec(box [(
                ::frame_benchmarking::BenchmarkParameter::x,
                2,
                <<T as Config>::MaxDelegationsPerDelegator as Get<u32>>::get(),
            )])
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let x = components
                .iter()
                .find(|&c| c.0 == ::frame_benchmarking::BenchmarkParameter::x)
                .ok_or("Could not find component in benchmark preparation.")?
                .1;
            ();
            let mut collators: Vec<T::AccountId> = Vec::new();
            for i in 1..x {
                let seed = USER_SEED - i;
                let collator = create_funded_collator::<T>(
                    "collator",
                    seed,
                    0u32.into(),
                    true,
                    collators.len() as u32 + 1u32,
                )?;
                collators.push(collator.clone());
            }
            let bond = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
            let need = bond * (collators.len() as u32).into();
            let default_minted = min_candidate_stk::<T>();
            let need: BalanceOf<T> = if need > default_minted {
                need - default_minted
            } else {
                0u32.into()
            };
            let (caller, _) = create_funded_user::<T>("caller", USER_SEED, need);
            let mut delegation_count = 0u32;
            let author = collators[0].clone();
            for col in collators {
                Pallet::<T>::delegate(
                    RawOrigin::Signed(caller.clone()).into(),
                    col,
                    bond,
                    0u32,
                    delegation_count,
                )?;
                delegation_count += 1u32;
            }
            Pallet::<T>::schedule_leave_delegators(RawOrigin::Signed(caller.clone()).into())?;
            roll_to_and_author::<T>(2, author);
            let __call = Call::<T>::new_call_variant_execute_leave_delegators(
                caller.clone(),
                delegation_count,
            );
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            if !Pallet::<T>::delegator_state(&caller).is_none() {
                                :: core :: panicking :: panic ("assertion failed: Pallet::<T>::delegator_state(&caller).is_none()")
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct cancel_leave_delegators;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for cancel_leave_delegators {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let collator: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, 0u32.into(), true, 1u32)?;
            let (caller, _): _ = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
            let bond: _ = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
            Pallet::<T>::delegate(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
                bond,
                0u32,
                0u32,
            )?;
            Pallet::<T>::schedule_leave_delegators(RawOrigin::Signed(caller.clone()).into())?;
            let __call = Call::<T>::new_call_variant_cancel_leave_delegators();
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            if !Pallet::<T>::delegator_state(&caller).unwrap().is_active() {
                                :: core :: panicking :: panic ("assertion failed: Pallet::<T>::delegator_state(&caller).unwrap().is_active()")
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct schedule_revoke_delegation;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for schedule_revoke_delegation {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let collator: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, 0u32.into(), true, 1u32)?;
            let (caller, _): _ = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
            let bond: _ = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
            Pallet::<T>::delegate(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
                bond,
                0u32,
                0u32,
            )?;
            let __call = Call::<T>::new_call_variant_schedule_revoke_delegation(collator.clone());
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            match (
                                &Pallet::<T>::delegation_scheduled_requests(&collator),
                                &<[_]>::into_vec(box [ScheduledRequest {
                                    delegator: caller,
                                    when_executable: 3,
                                    action: DelegationAction::Revoke(bond),
                                }]),
                            ) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct delegator_bond_more;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for delegator_bond_more {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let collator: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, 0u32.into(), true, 1u32)?;
            let (caller, _): _ = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
            let bond: _ = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
            Pallet::<T>::delegate(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
                bond,
                0u32,
                0u32,
            )?;
            let __call = Call::<T>::new_call_variant_delegator_bond_more(collator.clone(), bond);
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            let expected_bond = bond * 2u32.into();
                            match (&T::Currency::reserved_balance(&caller), &expected_bond) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct schedule_delegator_bond_less;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for schedule_delegator_bond_less {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let collator: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, 0u32.into(), true, 1u32)?;
            let (caller, total): _ = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
            Pallet::<T>::delegate(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
                total,
                0u32,
                0u32,
            )?;
            let bond_less = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
            let __call = Call::<T>::new_call_variant_schedule_delegator_bond_less(
                collator.clone(),
                bond_less,
            );
            let __benchmarked_call_encoded =
                ::frame_benchmarking::frame_support::codec::Encode::encode(&__call);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let __call_decoded = < Call < T > as :: frame_benchmarking :: frame_support :: codec :: Decode > :: decode (& mut & __benchmarked_call_encoded [..]) . expect ("call is encoded above, encoding must be correct") ;
                        let __origin = RawOrigin::Signed(caller.clone()).into();
                        < Call < T > as :: frame_benchmarking :: frame_support :: traits :: UnfilteredDispatchable > :: dispatch_bypass_filter (__call_decoded , __origin) ? ;
                    };
                    if verify {
                        {
                            let state = Pallet::<T>::delegator_state(&caller)
                                .expect("just request bonded less so exists");
                            match (
                                &Pallet::<T>::delegation_scheduled_requests(&collator),
                                &<[_]>::into_vec(box [ScheduledRequest {
                                    delegator: caller,
                                    when_executable: 3,
                                    action: DelegationAction::Decrease(bond_less),
                                }]),
                            ) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct execute_revoke_delegation;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for execute_revoke_delegation {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let collator: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, 0u32.into(), true, 1u32)?;
            let (caller, _): _ = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
            let bond: _ = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
            Pallet::<T>::delegate(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
                bond,
                0u32,
                0u32,
            )?;
            Pallet::<T>::schedule_revoke_delegation(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
            )?;
            roll_to_and_author::<T>(2, collator.clone());
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        Pallet::<T>::execute_delegation_request(
                            RawOrigin::Signed(caller.clone()).into(),
                            caller.clone(),
                            collator.clone(),
                        )?;
                    };
                    if verify {
                        {
                            if !!Pallet::<T>::is_delegator(&caller) {
                                ::core::panicking::panic(
                                    "assertion failed: !Pallet::<T>::is_delegator(&caller)",
                                )
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct execute_delegator_bond_less;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for execute_delegator_bond_less {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let collator: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, 0u32.into(), true, 1u32)?;
            let (caller, total): _ = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
            Pallet::<T>::delegate(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
                total,
                0u32,
                0u32,
            )?;
            let bond_less = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
            Pallet::<T>::schedule_delegator_bond_less(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
                bond_less,
            )?;
            roll_to_and_author::<T>(2, collator.clone());
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        Pallet::<T>::execute_delegation_request(
                            RawOrigin::Signed(caller.clone()).into(),
                            caller.clone(),
                            collator.clone(),
                        )?;
                    };
                    if verify {
                        {
                            let expected = total - bond_less;
                            match (&T::Currency::reserved_balance(&caller), &expected) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct cancel_revoke_delegation;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for cancel_revoke_delegation {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let collator: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, 0u32.into(), true, 1u32)?;
            let (caller, _): _ = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
            let bond: _ = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
            Pallet::<T>::delegate(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
                bond,
                0u32,
                0u32,
            )?;
            Pallet::<T>::schedule_revoke_delegation(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
            )?;
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        Pallet::<T>::cancel_delegation_request(
                            RawOrigin::Signed(caller.clone()).into(),
                            collator.clone(),
                        )?;
                    };
                    if verify {
                        {
                            if !!Pallet::<T>::delegation_scheduled_requests(&collator)
                                .iter()
                                .any(|x| &x.delegator == &caller)
                            {
                                :: core :: panicking :: panic ("assertion failed: !Pallet::<T>::delegation_scheduled_requests(&collator).iter().any(|x|\\n            &x.delegator == &caller)")
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct cancel_delegator_bond_less;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for cancel_delegator_bond_less {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let collator: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, 0u32.into(), true, 1u32)?;
            let (caller, total): _ = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
            Pallet::<T>::delegate(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
                total,
                0u32,
                0u32,
            )?;
            let bond_less = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
            Pallet::<T>::schedule_delegator_bond_less(
                RawOrigin::Signed(caller.clone()).into(),
                collator.clone(),
                bond_less,
            )?;
            roll_to_and_author::<T>(2, collator.clone());
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        Pallet::<T>::cancel_delegation_request(
                            RawOrigin::Signed(caller.clone()).into(),
                            collator.clone(),
                        )?;
                    };
                    if verify {
                        {
                            if !!Pallet::<T>::delegation_scheduled_requests(&collator)
                                .iter()
                                .any(|x| &x.delegator == &caller)
                            {
                                :: core :: panicking :: panic ("assertion failed: !Pallet::<T>::delegation_scheduled_requests(&collator).iter().any(|x|\\n            &x.delegator == &caller)")
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct round_transition_on_initialize;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for round_transition_on_initialize {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            <[_]>::into_vec(box [
                (::frame_benchmarking::BenchmarkParameter::x, 8, 100),
                (
                    ::frame_benchmarking::BenchmarkParameter::y,
                    0,
                    (<<T as Config>::MaxTopDelegationsPerCandidate as Get<u32>>::get() * 100),
                ),
            ])
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let x = components
                .iter()
                .find(|&c| c.0 == ::frame_benchmarking::BenchmarkParameter::x)
                .ok_or("Could not find component in benchmark preparation.")?
                .1;
            let y = components
                .iter()
                .find(|&c| c.0 == ::frame_benchmarking::BenchmarkParameter::y)
                .ok_or("Could not find component in benchmark preparation.")?
                .1;
            ();
            ();
            let max_delegators_per_collator: _ =
                <<T as Config>::MaxTopDelegationsPerCandidate as Get<u32>>::get();
            let max_delegations = x * max_delegators_per_collator;
            let total_delegations: u32 = if max_delegations < y {
                max_delegations
            } else {
                y
            };
            let high_inflation: Range<Perbill> = Range {
                min: Perbill::one(),
                ideal: Perbill::one(),
                max: Perbill::one(),
            };
            Pallet::<T>::set_inflation(RawOrigin::Root.into(), high_inflation.clone())?;
            Pallet::<T>::set_blocks_per_round(RawOrigin::Root.into(), 100u32)?;
            Pallet::<T>::set_total_selected(RawOrigin::Root.into(), 100u32)?;
            let mut collators: Vec<T::AccountId> = Vec::new();
            let mut collator_count = 1u32;
            for i in 0..x {
                let seed = USER_SEED - i;
                let collator = create_funded_collator::<T>(
                    "collator",
                    seed,
                    min_candidate_stk::<T>() * 1_000_000u32.into(),
                    true,
                    collator_count,
                )?;
                collators.push(collator);
                collator_count += 1u32;
            }
            let collator_starting_balances: Vec<(
                T::AccountId,
                <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
            )> = collators
                .iter()
                .map(|x| (x.clone(), T::Currency::free_balance(&x)))
                .collect();
            let mut col_del_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
            collators.iter().for_each(|x| {
                col_del_count.insert(x.clone(), 0u32);
            });
            let mut delegators: Vec<T::AccountId> = Vec::new();
            let mut remaining_delegations = if total_delegations > max_delegators_per_collator {
                for j in 1..(max_delegators_per_collator + 1) {
                    let seed = USER_SEED + j;
                    let delegator = create_funded_delegator::<T>(
                        "delegator",
                        seed,
                        min_candidate_stk::<T>() * 1_000_000u32.into(),
                        collators[0].clone(),
                        true,
                        delegators.len() as u32,
                    )?;
                    delegators.push(delegator);
                }
                total_delegations - max_delegators_per_collator
            } else {
                for j in 1..(total_delegations + 1) {
                    let seed = USER_SEED + j;
                    let delegator = create_funded_delegator::<T>(
                        "delegator",
                        seed,
                        min_candidate_stk::<T>() * 1_000_000u32.into(),
                        collators[0].clone(),
                        true,
                        delegators.len() as u32,
                    )?;
                    delegators.push(delegator);
                }
                0u32
            };
            col_del_count.insert(collators[0].clone(), delegators.len() as u32);
            if remaining_delegations > 0 {
                for (col, n_count) in col_del_count.iter_mut() {
                    if n_count < &mut (delegators.len() as u32) {
                        let mut open_spots = delegators.len() as u32 - *n_count;
                        while open_spots > 0 && remaining_delegations > 0 {
                            let caller = delegators[open_spots as usize - 1usize].clone();
                            if let Ok(_) = Pallet::<T>::delegate(
                                RawOrigin::Signed(caller.clone()).into(),
                                col.clone(),
                                <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get(),
                                *n_count,
                                collators.len() as u32,
                            ) {
                                *n_count += 1;
                                remaining_delegations -= 1;
                            }
                            open_spots -= 1;
                        }
                    }
                    if remaining_delegations == 0 {
                        break;
                    }
                }
            }
            let delegator_starting_balances: Vec<(
                T::AccountId,
                <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
            )> = delegators
                .iter()
                .map(|x| (x.clone(), T::Currency::free_balance(&x)))
                .collect();
            let before_running_round_index = Pallet::<T>::round().current;
            let round_length: T::BlockNumber = Pallet::<T>::round().length.into();
            let reward_delay = <<T as Config>::RewardPaymentDelay as Get<u32>>::get() + 2u32;
            let mut now = <frame_system::Pallet<T>>::block_number() + 1u32.into();
            let mut counter = 0usize;
            let end = Pallet::<T>::round().first + (round_length * reward_delay.into());
            while now < end {
                let author = collators[counter % collators.len()].clone();
                parachain_staking_on_finalize::<T>(author);
                <frame_system::Pallet<T>>::on_finalize(<frame_system::Pallet<T>>::block_number());
                <frame_system::Pallet<T>>::set_block_number(
                    <frame_system::Pallet<T>>::block_number() + 1u32.into(),
                );
                <frame_system::Pallet<T>>::on_initialize(<frame_system::Pallet<T>>::block_number());
                Pallet::<T>::on_initialize(<frame_system::Pallet<T>>::block_number());
                now += 1u32.into();
                counter += 1usize;
            }
            parachain_staking_on_finalize::<T>(collators[counter % collators.len()].clone());
            <frame_system::Pallet<T>>::on_finalize(<frame_system::Pallet<T>>::block_number());
            <frame_system::Pallet<T>>::set_block_number(
                <frame_system::Pallet<T>>::block_number() + 1u32.into(),
            );
            <frame_system::Pallet<T>>::on_initialize(<frame_system::Pallet<T>>::block_number());
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        Pallet::<T>::on_initialize(<frame_system::Pallet<T>>::block_number());
                    };
                    if verify {
                        {
                            for (col, initial) in collator_starting_balances {
                                if !(T::Currency::free_balance(&col) > initial) {
                                    :: core :: panicking :: panic ("assertion failed: T::Currency::free_balance(&col) > initial")
                                };
                            }
                            for (col, initial) in delegator_starting_balances {
                                if !(T::Currency::free_balance(&col) > initial) {
                                    :: core :: panicking :: panic ("assertion failed: T::Currency::free_balance(&col) > initial")
                                };
                            }
                            match (
                                &Pallet::<T>::round().current,
                                &(before_running_round_index + reward_delay),
                            ) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct pay_one_collator_reward;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for pay_one_collator_reward {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            <[_]>::into_vec(box [(
                ::frame_benchmarking::BenchmarkParameter::y,
                0,
                <<T as Config>::MaxTopDelegationsPerCandidate as Get<u32>>::get(),
            )])
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let y = components
                .iter()
                .find(|&c| c.0 == ::frame_benchmarking::BenchmarkParameter::y)
                .ok_or("Could not find component in benchmark preparation.")?
                .1;
            ();
            use crate::{
                DelayedPayout, DelayedPayouts, AtStake, CollatorSnapshot, BondWithAutoCompound,
                Points, AwardedPts,
            };
            let before_running_round_index = Pallet::<T>::round().current;
            let initial_stake_amount = min_candidate_stk::<T>() * 1_000_000u32.into();
            let mut total_staked = 0u32.into();
            let sole_collator =
                create_funded_collator::<T>("collator", 0, initial_stake_amount, true, 1u32)?;
            total_staked += initial_stake_amount;
            let mut delegators: Vec<T::AccountId> = Vec::new();
            for i in 0..y {
                let seed = USER_SEED + i;
                let delegator = create_funded_delegator::<T>(
                    "delegator",
                    seed,
                    initial_stake_amount,
                    sole_collator.clone(),
                    true,
                    delegators.len() as u32,
                )?;
                delegators.push(delegator);
                total_staked += initial_stake_amount;
            }
            let round_for_payout = 5;
            <DelayedPayouts<T>>::insert(
                &round_for_payout,
                DelayedPayout {
                    round_issuance: 1000u32.into(),
                    total_staking_reward: total_staked,
                    collator_commission: Perbill::from_rational(1u32, 100u32),
                },
            );
            let mut delegations: Vec<BondWithAutoCompound<T::AccountId, BalanceOf<T>>> = Vec::new();
            for delegator in &delegators {
                delegations.push(BondWithAutoCompound {
                    owner: delegator.clone(),
                    amount: 100u32.into(),
                    auto_compound: Percent::zero(),
                });
            }
            <AtStake<T>>::insert(
                round_for_payout,
                &sole_collator,
                CollatorSnapshot {
                    bond: 1_000u32.into(),
                    delegations,
                    total: 1_000_000u32.into(),
                },
            );
            <Points<T>>::insert(round_for_payout, 100);
            <AwardedPts<T>>::insert(round_for_payout, &sole_collator, 20);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        let round_for_payout = 5;
                        let payout_info = Pallet::<T>::delayed_payouts(round_for_payout)
                            .expect("payout expected");
                        let result =
                            Pallet::<T>::pay_one_collator_reward(round_for_payout, payout_info);
                        if !result.0.is_some() {
                            ::core::panicking::panic("assertion failed: result.0.is_some()")
                        };
                    };
                    if verify {
                        {
                            if !(T::Currency::free_balance(&sole_collator) > initial_stake_amount) {
                                ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                                    &["collator should have been paid in pay_one_collator_reward"],
                                    &[],
                                ))
                            };
                            for delegator in &delegators {
                                if !(T::Currency::free_balance(&delegator) > initial_stake_amount) {
                                    :: core :: panicking :: panic_fmt (:: core :: fmt :: Arguments :: new_v1 (& ["delegator should have been paid in pay_one_collator_reward"] , & []))
                                };
                            }
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct base_on_initialize;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for base_on_initialize {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            ::alloc::vec::Vec::new()
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let collator: T::AccountId =
                create_funded_collator::<T>("collator", USER_SEED, 0u32.into(), true, 1u32)?;
            let start: _ = <frame_system::Pallet<T>>::block_number();
            parachain_staking_on_finalize::<T>(collator.clone());
            <frame_system::Pallet<T>>::on_finalize(start);
            <frame_system::Pallet<T>>::set_block_number(start + 1u32.into());
            let end = <frame_system::Pallet<T>>::block_number();
            <frame_system::Pallet<T>>::on_initialize(end);
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        Pallet::<T>::on_initialize(end);
                    };
                    if verify {
                        {
                            match (&(start + 1u32.into()), &end) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::None,
                                        );
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    struct delegation_set_auto_compounding_reward;
    #[allow(unused_variables)]
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T>
        for delegation_set_auto_compounding_reward
    {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            <[_]>::into_vec(box [
                (
                    ::frame_benchmarking::BenchmarkParameter::x,
                    0,
                    <<T as Config>::MaxDelegationsPerDelegator as Get<u32>>::get(),
                ),
                (
                    ::frame_benchmarking::BenchmarkParameter::y,
                    0,
                    <<T as Config>::MaxTopDelegationsPerCandidate as Get<u32>>::get(),
                ),
            ])
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            let x = components
                .iter()
                .find(|&c| c.0 == ::frame_benchmarking::BenchmarkParameter::x)
                .ok_or("Could not find component in benchmark preparation.")?
                .1;
            let y = components
                .iter()
                .find(|&c| c.0 == ::frame_benchmarking::BenchmarkParameter::y)
                .ok_or("Could not find component in benchmark preparation.")?
                .1;
            ();
            ();
            use crate::AutoCompoundingDelegations;
            use crate::auto_compounding::{self, DelegationAutoCompoundConfig};
            let min_candidate_stake = min_candidate_stk::<T>();
            let min_delegator_stake = min_delegator_stk::<T>();
            let mut seed = Seed::new();
            let prime_candidate =
                create_funded_collator::<T>("collator", seed.take(), min_candidate_stake, true, 1)?;
            let prime_delegator = create_funded_delegator::<T>(
                "delegator",
                seed.take(),
                min_delegator_stake * (x + 1).into(),
                prime_candidate.clone(),
                true,
                0,
            )?;
            for i in 1..x {
                let collator = create_funded_collator::<T>(
                    "collator",
                    seed.take(),
                    min_candidate_stake,
                    true,
                    i + 1,
                )?;
                Pallet::<T>::delegate(
                    RawOrigin::Signed(prime_delegator.clone()).into(),
                    collator,
                    min_delegator_stake,
                    0,
                    i,
                )?;
            }
            let mut auto_compounding_state = <AutoCompoundingDelegations<T>>::get(&prime_candidate);
            for i in 1..y {
                let delegator = create_funded_delegator::<T>(
                    "delegator",
                    seed.take(),
                    min_delegator_stake,
                    prime_candidate.clone(),
                    true,
                    i,
                )?;
                auto_compounding::set_delegation_config(
                    &mut auto_compounding_state,
                    delegator,
                    Percent::from_percent(100),
                );
            }
            <AutoCompoundingDelegations<T>>::insert(
                prime_candidate.clone(),
                auto_compounding_state,
            );
            Ok(::frame_benchmarking::Box::new(
                move || -> Result<(), ::frame_benchmarking::BenchmarkError> {
                    {
                        Pallet::<T>::delegation_set_auto_compounding_reward(
                            RawOrigin::Signed(prime_delegator.clone()).into(),
                            prime_candidate.clone(),
                            Percent::from_percent(50),
                            x + 1,
                            y + 1,
                        )?;
                    };
                    if verify {
                        {
                            let actual_auto_compound =
                                <AutoCompoundingDelegations<T>>::get(&prime_candidate)
                                    .into_iter()
                                    .find(|d| d.delegator == prime_delegator);
                            let expected_auto_compound = Some(DelegationAutoCompoundConfig {
                                delegator: prime_delegator,
                                value: Percent::from_percent(50),
                            });
                            match (&expected_auto_compound, &actual_auto_compound) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        :: core :: panicking :: assert_failed (kind , & * left_val , & * right_val , :: core :: option :: Option :: Some (:: core :: fmt :: Arguments :: new_v1 (& ["delegation must have an auto-compound entry"] , & []))) ;
                                    }
                                }
                            };
                        };
                    }
                    Ok(())
                },
            ))
        }
    }
    #[allow(non_camel_case_types)]
    enum SelectedBenchmark {
        set_staking_expectations,
        set_inflation,
        set_parachain_bond_account,
        set_parachain_bond_reserve_percent,
        set_total_selected,
        set_collator_commission,
        set_blocks_per_round,
        join_candidates,
        schedule_leave_candidates,
        execute_leave_candidates,
        cancel_leave_candidates,
        go_offline,
        go_online,
        candidate_bond_more,
        schedule_candidate_bond_less,
        execute_candidate_bond_less,
        cancel_candidate_bond_less,
        delegate,
        schedule_leave_delegators,
        execute_leave_delegators,
        cancel_leave_delegators,
        schedule_revoke_delegation,
        delegator_bond_more,
        schedule_delegator_bond_less,
        execute_revoke_delegation,
        execute_delegator_bond_less,
        cancel_revoke_delegation,
        cancel_delegator_bond_less,
        round_transition_on_initialize,
        pay_one_collator_reward,
        base_on_initialize,
        delegation_set_auto_compounding_reward,
    }
    impl<T: Config> ::frame_benchmarking::BenchmarkingSetup<T> for SelectedBenchmark {
        fn components(
            &self,
        ) -> ::frame_benchmarking::Vec<(::frame_benchmarking::BenchmarkParameter, u32, u32)>
        {
            match self { Self :: set_staking_expectations => < set_staking_expectations as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& set_staking_expectations) , Self :: set_inflation => < set_inflation as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& set_inflation) , Self :: set_parachain_bond_account => < set_parachain_bond_account as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& set_parachain_bond_account) , Self :: set_parachain_bond_reserve_percent => < set_parachain_bond_reserve_percent as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& set_parachain_bond_reserve_percent) , Self :: set_total_selected => < set_total_selected as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& set_total_selected) , Self :: set_collator_commission => < set_collator_commission as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& set_collator_commission) , Self :: set_blocks_per_round => < set_blocks_per_round as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& set_blocks_per_round) , Self :: join_candidates => < join_candidates as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& join_candidates) , Self :: schedule_leave_candidates => < schedule_leave_candidates as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& schedule_leave_candidates) , Self :: execute_leave_candidates => < execute_leave_candidates as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& execute_leave_candidates) , Self :: cancel_leave_candidates => < cancel_leave_candidates as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& cancel_leave_candidates) , Self :: go_offline => < go_offline as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& go_offline) , Self :: go_online => < go_online as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& go_online) , Self :: candidate_bond_more => < candidate_bond_more as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& candidate_bond_more) , Self :: schedule_candidate_bond_less => < schedule_candidate_bond_less as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& schedule_candidate_bond_less) , Self :: execute_candidate_bond_less => < execute_candidate_bond_less as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& execute_candidate_bond_less) , Self :: cancel_candidate_bond_less => < cancel_candidate_bond_less as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& cancel_candidate_bond_less) , Self :: delegate => < delegate as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& delegate) , Self :: schedule_leave_delegators => < schedule_leave_delegators as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& schedule_leave_delegators) , Self :: execute_leave_delegators => < execute_leave_delegators as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& execute_leave_delegators) , Self :: cancel_leave_delegators => < cancel_leave_delegators as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& cancel_leave_delegators) , Self :: schedule_revoke_delegation => < schedule_revoke_delegation as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& schedule_revoke_delegation) , Self :: delegator_bond_more => < delegator_bond_more as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& delegator_bond_more) , Self :: schedule_delegator_bond_less => < schedule_delegator_bond_less as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& schedule_delegator_bond_less) , Self :: execute_revoke_delegation => < execute_revoke_delegation as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& execute_revoke_delegation) , Self :: execute_delegator_bond_less => < execute_delegator_bond_less as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& execute_delegator_bond_less) , Self :: cancel_revoke_delegation => < cancel_revoke_delegation as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& cancel_revoke_delegation) , Self :: cancel_delegator_bond_less => < cancel_delegator_bond_less as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& cancel_delegator_bond_less) , Self :: round_transition_on_initialize => < round_transition_on_initialize as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& round_transition_on_initialize) , Self :: pay_one_collator_reward => < pay_one_collator_reward as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& pay_one_collator_reward) , Self :: base_on_initialize => < base_on_initialize as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& base_on_initialize) , Self :: delegation_set_auto_compounding_reward => < delegation_set_auto_compounding_reward as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& delegation_set_auto_compounding_reward) , }
        }
        fn instance(
            &self,
            components: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            verify: bool,
        ) -> Result<
            ::frame_benchmarking::Box<
                dyn FnOnce() -> Result<(), ::frame_benchmarking::BenchmarkError>,
            >,
            ::frame_benchmarking::BenchmarkError,
        > {
            match self { Self :: set_staking_expectations => < set_staking_expectations as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& set_staking_expectations , components , verify) , Self :: set_inflation => < set_inflation as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& set_inflation , components , verify) , Self :: set_parachain_bond_account => < set_parachain_bond_account as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& set_parachain_bond_account , components , verify) , Self :: set_parachain_bond_reserve_percent => < set_parachain_bond_reserve_percent as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& set_parachain_bond_reserve_percent , components , verify) , Self :: set_total_selected => < set_total_selected as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& set_total_selected , components , verify) , Self :: set_collator_commission => < set_collator_commission as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& set_collator_commission , components , verify) , Self :: set_blocks_per_round => < set_blocks_per_round as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& set_blocks_per_round , components , verify) , Self :: join_candidates => < join_candidates as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& join_candidates , components , verify) , Self :: schedule_leave_candidates => < schedule_leave_candidates as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& schedule_leave_candidates , components , verify) , Self :: execute_leave_candidates => < execute_leave_candidates as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& execute_leave_candidates , components , verify) , Self :: cancel_leave_candidates => < cancel_leave_candidates as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& cancel_leave_candidates , components , verify) , Self :: go_offline => < go_offline as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& go_offline , components , verify) , Self :: go_online => < go_online as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& go_online , components , verify) , Self :: candidate_bond_more => < candidate_bond_more as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& candidate_bond_more , components , verify) , Self :: schedule_candidate_bond_less => < schedule_candidate_bond_less as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& schedule_candidate_bond_less , components , verify) , Self :: execute_candidate_bond_less => < execute_candidate_bond_less as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& execute_candidate_bond_less , components , verify) , Self :: cancel_candidate_bond_less => < cancel_candidate_bond_less as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& cancel_candidate_bond_less , components , verify) , Self :: delegate => < delegate as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& delegate , components , verify) , Self :: schedule_leave_delegators => < schedule_leave_delegators as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& schedule_leave_delegators , components , verify) , Self :: execute_leave_delegators => < execute_leave_delegators as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& execute_leave_delegators , components , verify) , Self :: cancel_leave_delegators => < cancel_leave_delegators as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& cancel_leave_delegators , components , verify) , Self :: schedule_revoke_delegation => < schedule_revoke_delegation as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& schedule_revoke_delegation , components , verify) , Self :: delegator_bond_more => < delegator_bond_more as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& delegator_bond_more , components , verify) , Self :: schedule_delegator_bond_less => < schedule_delegator_bond_less as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& schedule_delegator_bond_less , components , verify) , Self :: execute_revoke_delegation => < execute_revoke_delegation as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& execute_revoke_delegation , components , verify) , Self :: execute_delegator_bond_less => < execute_delegator_bond_less as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& execute_delegator_bond_less , components , verify) , Self :: cancel_revoke_delegation => < cancel_revoke_delegation as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& cancel_revoke_delegation , components , verify) , Self :: cancel_delegator_bond_less => < cancel_delegator_bond_less as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& cancel_delegator_bond_less , components , verify) , Self :: round_transition_on_initialize => < round_transition_on_initialize as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& round_transition_on_initialize , components , verify) , Self :: pay_one_collator_reward => < pay_one_collator_reward as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& pay_one_collator_reward , components , verify) , Self :: base_on_initialize => < base_on_initialize as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& base_on_initialize , components , verify) , Self :: delegation_set_auto_compounding_reward => < delegation_set_auto_compounding_reward as :: frame_benchmarking :: BenchmarkingSetup < T > > :: instance (& delegation_set_auto_compounding_reward , components , verify) , }
        }
    }
    impl<T: Config> ::frame_benchmarking::Benchmarking for Pallet<T>
    where
        T: frame_system::Config,
    {
        fn benchmarks(
            extra: bool,
        ) -> ::frame_benchmarking::Vec<::frame_benchmarking::BenchmarkMetadata> {
            let mut all_names = <[_]>::into_vec(box [
                "set_staking_expectations".as_ref(),
                "set_inflation".as_ref(),
                "set_parachain_bond_account".as_ref(),
                "set_parachain_bond_reserve_percent".as_ref(),
                "set_total_selected".as_ref(),
                "set_collator_commission".as_ref(),
                "set_blocks_per_round".as_ref(),
                "join_candidates".as_ref(),
                "schedule_leave_candidates".as_ref(),
                "execute_leave_candidates".as_ref(),
                "cancel_leave_candidates".as_ref(),
                "go_offline".as_ref(),
                "go_online".as_ref(),
                "candidate_bond_more".as_ref(),
                "schedule_candidate_bond_less".as_ref(),
                "execute_candidate_bond_less".as_ref(),
                "cancel_candidate_bond_less".as_ref(),
                "delegate".as_ref(),
                "schedule_leave_delegators".as_ref(),
                "execute_leave_delegators".as_ref(),
                "cancel_leave_delegators".as_ref(),
                "schedule_revoke_delegation".as_ref(),
                "delegator_bond_more".as_ref(),
                "schedule_delegator_bond_less".as_ref(),
                "execute_revoke_delegation".as_ref(),
                "execute_delegator_bond_less".as_ref(),
                "cancel_revoke_delegation".as_ref(),
                "cancel_delegator_bond_less".as_ref(),
                "round_transition_on_initialize".as_ref(),
                "pay_one_collator_reward".as_ref(),
                "base_on_initialize".as_ref(),
                "delegation_set_auto_compounding_reward".as_ref(),
            ]);
            if !extra {
                let extra = [];
                all_names.retain(|x| !extra.contains(x));
            }
            all_names . into_iter () . map (| benchmark | { let selected_benchmark = match benchmark { "set_staking_expectations" => SelectedBenchmark :: set_staking_expectations , "set_inflation" => SelectedBenchmark :: set_inflation , "set_parachain_bond_account" => SelectedBenchmark :: set_parachain_bond_account , "set_parachain_bond_reserve_percent" => SelectedBenchmark :: set_parachain_bond_reserve_percent , "set_total_selected" => SelectedBenchmark :: set_total_selected , "set_collator_commission" => SelectedBenchmark :: set_collator_commission , "set_blocks_per_round" => SelectedBenchmark :: set_blocks_per_round , "join_candidates" => SelectedBenchmark :: join_candidates , "schedule_leave_candidates" => SelectedBenchmark :: schedule_leave_candidates , "execute_leave_candidates" => SelectedBenchmark :: execute_leave_candidates , "cancel_leave_candidates" => SelectedBenchmark :: cancel_leave_candidates , "go_offline" => SelectedBenchmark :: go_offline , "go_online" => SelectedBenchmark :: go_online , "candidate_bond_more" => SelectedBenchmark :: candidate_bond_more , "schedule_candidate_bond_less" => SelectedBenchmark :: schedule_candidate_bond_less , "execute_candidate_bond_less" => SelectedBenchmark :: execute_candidate_bond_less , "cancel_candidate_bond_less" => SelectedBenchmark :: cancel_candidate_bond_less , "delegate" => SelectedBenchmark :: delegate , "schedule_leave_delegators" => SelectedBenchmark :: schedule_leave_delegators , "execute_leave_delegators" => SelectedBenchmark :: execute_leave_delegators , "cancel_leave_delegators" => SelectedBenchmark :: cancel_leave_delegators , "schedule_revoke_delegation" => SelectedBenchmark :: schedule_revoke_delegation , "delegator_bond_more" => SelectedBenchmark :: delegator_bond_more , "schedule_delegator_bond_less" => SelectedBenchmark :: schedule_delegator_bond_less , "execute_revoke_delegation" => SelectedBenchmark :: execute_revoke_delegation , "execute_delegator_bond_less" => SelectedBenchmark :: execute_delegator_bond_less , "cancel_revoke_delegation" => SelectedBenchmark :: cancel_revoke_delegation , "cancel_delegator_bond_less" => SelectedBenchmark :: cancel_delegator_bond_less , "round_transition_on_initialize" => SelectedBenchmark :: round_transition_on_initialize , "pay_one_collator_reward" => SelectedBenchmark :: pay_one_collator_reward , "base_on_initialize" => SelectedBenchmark :: base_on_initialize , "delegation_set_auto_compounding_reward" => SelectedBenchmark :: delegation_set_auto_compounding_reward , _ => :: core :: panicking :: panic_fmt (:: core :: fmt :: Arguments :: new_v1 (& ["all benchmarks should be selectable"] , & [])) , } ; let components = < SelectedBenchmark as :: frame_benchmarking :: BenchmarkingSetup < T > > :: components (& selected_benchmark) ; :: frame_benchmarking :: BenchmarkMetadata { name : benchmark . as_bytes () . to_vec () , components , } }) . collect :: < :: frame_benchmarking :: Vec < _ > > ()
        }
        fn run_benchmark(
            extrinsic: &[u8],
            c: &[(::frame_benchmarking::BenchmarkParameter, u32)],
            whitelist: &[::frame_benchmarking::TrackedStorageKey],
            verify: bool,
            internal_repeats: u32,
        ) -> Result<
            ::frame_benchmarking::Vec<::frame_benchmarking::BenchmarkResult>,
            ::frame_benchmarking::BenchmarkError,
        > {
            let extrinsic = ::frame_benchmarking::str::from_utf8(extrinsic)
                .map_err(|_| "`extrinsic` is not a valid utf8 string!")?;
            let selected_benchmark = match extrinsic {
                "set_staking_expectations" => SelectedBenchmark::set_staking_expectations,
                "set_inflation" => SelectedBenchmark::set_inflation,
                "set_parachain_bond_account" => SelectedBenchmark::set_parachain_bond_account,
                "set_parachain_bond_reserve_percent" => {
                    SelectedBenchmark::set_parachain_bond_reserve_percent
                }
                "set_total_selected" => SelectedBenchmark::set_total_selected,
                "set_collator_commission" => SelectedBenchmark::set_collator_commission,
                "set_blocks_per_round" => SelectedBenchmark::set_blocks_per_round,
                "join_candidates" => SelectedBenchmark::join_candidates,
                "schedule_leave_candidates" => SelectedBenchmark::schedule_leave_candidates,
                "execute_leave_candidates" => SelectedBenchmark::execute_leave_candidates,
                "cancel_leave_candidates" => SelectedBenchmark::cancel_leave_candidates,
                "go_offline" => SelectedBenchmark::go_offline,
                "go_online" => SelectedBenchmark::go_online,
                "candidate_bond_more" => SelectedBenchmark::candidate_bond_more,
                "schedule_candidate_bond_less" => SelectedBenchmark::schedule_candidate_bond_less,
                "execute_candidate_bond_less" => SelectedBenchmark::execute_candidate_bond_less,
                "cancel_candidate_bond_less" => SelectedBenchmark::cancel_candidate_bond_less,
                "delegate" => SelectedBenchmark::delegate,
                "schedule_leave_delegators" => SelectedBenchmark::schedule_leave_delegators,
                "execute_leave_delegators" => SelectedBenchmark::execute_leave_delegators,
                "cancel_leave_delegators" => SelectedBenchmark::cancel_leave_delegators,
                "schedule_revoke_delegation" => SelectedBenchmark::schedule_revoke_delegation,
                "delegator_bond_more" => SelectedBenchmark::delegator_bond_more,
                "schedule_delegator_bond_less" => SelectedBenchmark::schedule_delegator_bond_less,
                "execute_revoke_delegation" => SelectedBenchmark::execute_revoke_delegation,
                "execute_delegator_bond_less" => SelectedBenchmark::execute_delegator_bond_less,
                "cancel_revoke_delegation" => SelectedBenchmark::cancel_revoke_delegation,
                "cancel_delegator_bond_less" => SelectedBenchmark::cancel_delegator_bond_less,
                "round_transition_on_initialize" => {
                    SelectedBenchmark::round_transition_on_initialize
                }
                "pay_one_collator_reward" => SelectedBenchmark::pay_one_collator_reward,
                "base_on_initialize" => SelectedBenchmark::base_on_initialize,
                "delegation_set_auto_compounding_reward" => {
                    SelectedBenchmark::delegation_set_auto_compounding_reward
                }
                _ => return Err("Could not find extrinsic.".into()),
            };
            let mut whitelist = whitelist.to_vec();
            let whitelisted_caller_key = < frame_system :: Account < T > as :: frame_benchmarking :: frame_support :: storage :: StorageMap < _ , _ > > :: hashed_key_for (:: frame_benchmarking :: whitelisted_caller :: < T :: AccountId > ()) ;
            whitelist.push(whitelisted_caller_key.into());
            let transactional_layer_key = ::frame_benchmarking::TrackedStorageKey::new(
                ::frame_benchmarking::frame_support::storage::transactional::TRANSACTION_LEVEL_KEY
                    .into(),
            );
            whitelist.push(transactional_layer_key);
            ::frame_benchmarking::benchmarking::set_whitelist(whitelist);
            let mut results: ::frame_benchmarking::Vec<::frame_benchmarking::BenchmarkResult> =
                ::frame_benchmarking::Vec::new();
            for _ in 0..internal_repeats.max(1) {
                let closure_to_benchmark =
                    <SelectedBenchmark as ::frame_benchmarking::BenchmarkingSetup<T>>::instance(
                        &selected_benchmark,
                        c,
                        verify,
                    )?;
                if ::frame_benchmarking::Zero::is_zero(&frame_system::Pallet::<T>::block_number()) {
                    frame_system::Pallet::<T>::set_block_number(1u32.into());
                }
                ::frame_benchmarking::benchmarking::commit_db();
                ::frame_benchmarking::benchmarking::reset_read_write_count();
                {
                    let lvl = ::log::Level::Trace;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api_log(
                            ::core::fmt::Arguments::new_v1(
                                &["Start Benchmark: "],
                                &[::core::fmt::ArgumentV1::new_debug(&c)],
                            ),
                            lvl,
                            &(
                                "benchmark",
                                "pallet_parachain_staking::benchmarks",
                                "pallets/parachain-staking/src/benchmarks.rs",
                                176u32,
                            ),
                            ::log::__private_api::Option::None,
                        );
                    }
                };
                let start_pov = ::frame_benchmarking::benchmarking::proof_size();
                let start_extrinsic = ::frame_benchmarking::benchmarking::current_time();
                closure_to_benchmark()?;
                let finish_extrinsic = ::frame_benchmarking::benchmarking::current_time();
                let end_pov = ::frame_benchmarking::benchmarking::proof_size();
                let elapsed_extrinsic = finish_extrinsic.saturating_sub(start_extrinsic);
                let diff_pov = match (start_pov, end_pov) {
                    (Some(start), Some(end)) => end.saturating_sub(start),
                    _ => Default::default(),
                };
                ::frame_benchmarking::benchmarking::commit_db();
                {
                    let lvl = ::log::Level::Trace;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api_log(
                            ::core::fmt::Arguments::new_v1(
                                &["End Benchmark: ", " ns"],
                                &[::core::fmt::ArgumentV1::new_display(&elapsed_extrinsic)],
                            ),
                            lvl,
                            &(
                                "benchmark",
                                "pallet_parachain_staking::benchmarks",
                                "pallets/parachain-staking/src/benchmarks.rs",
                                176u32,
                            ),
                            ::log::__private_api::Option::None,
                        );
                    }
                };
                let read_write_count = ::frame_benchmarking::benchmarking::read_write_count();
                {
                    let lvl = ::log::Level::Trace;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api_log(
                            ::core::fmt::Arguments::new_v1(
                                &["Read/Write Count "],
                                &[::core::fmt::ArgumentV1::new_debug(&read_write_count)],
                            ),
                            lvl,
                            &(
                                "benchmark",
                                "pallet_parachain_staking::benchmarks",
                                "pallets/parachain-staking/src/benchmarks.rs",
                                176u32,
                            ),
                            ::log::__private_api::Option::None,
                        );
                    }
                };
                let start_storage_root = ::frame_benchmarking::benchmarking::current_time();
                ::frame_benchmarking::storage_root(::frame_benchmarking::StateVersion::V1);
                let finish_storage_root = ::frame_benchmarking::benchmarking::current_time();
                let elapsed_storage_root = finish_storage_root - start_storage_root;
                let skip_meta = [];
                let read_and_written_keys = if skip_meta.contains(&extrinsic) {
                    <[_]>::into_vec(box [(b"Skipped Metadata".to_vec(), 0, 0, false)])
                } else {
                    ::frame_benchmarking::benchmarking::get_read_and_written_keys()
                };
                results.push(::frame_benchmarking::BenchmarkResult {
                    components: c.to_vec(),
                    extrinsic_time: elapsed_extrinsic,
                    storage_root_time: elapsed_storage_root,
                    reads: read_write_count.0,
                    repeat_reads: read_write_count.1,
                    writes: read_write_count.2,
                    repeat_writes: read_write_count.3,
                    proof_size: diff_pov,
                    keys: read_and_written_keys,
                });
                ::frame_benchmarking::benchmarking::wipe_db();
            }
            return Ok(results);
        }
    }
}
