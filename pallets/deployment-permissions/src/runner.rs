// Copyright 2025, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::{sp_runtime::DispatchError, weights::Weight};
use pallet_evm::{
    runner::Runner as RunnerT, BalanceOf, Config, EvmConfig, FeeCalculator, RunnerError,
};
use sp_core::{H160, H256, U256};

use crate::EnsureCreateOrigin;

#[derive(Default)]
pub struct PermissionedDeploy<T, R, C> {
    _marker: PhantomData<(T, R, C)>,
}

#[derive(Debug, PartialEq)]
pub enum PermissionedDeployError<T, R, C>
where
    T: Config,
    R: RunnerT<T>,
    C: EnsureCreateOrigin<T>,
{
    Runner(R::Error),
    Permission(C::Error),
}

impl<T, R, C> From<PermissionedDeployError<T, R, C>> for DispatchError
where
    T: Config,
    R: RunnerT<T>,
    C: EnsureCreateOrigin<T>,
{
    fn from(value: PermissionedDeployError<T, R, C>) -> Self {
        match value {
            PermissionedDeployError::Runner(error) => error.into(),
            PermissionedDeployError::Permission(error) => error.into(),
        }
    }
}

pub type RunnerOf<T> = <T as pallet_evm::Config>::Runner;
pub type RunnerErrorOf<T> = RunnerError<<RunnerOf<T> as RunnerT<T>>::Error>;

impl<T, R, C> RunnerT<T> for PermissionedDeploy<T, R, C>
where
    T: Config,
    R: RunnerT<T>,
    C: EnsureCreateOrigin<T>,
    BalanceOf<T>: TryFrom<U256> + Into<U256>,
{
    type Error = PermissionedDeployError<T, R, C>;

    fn validate(
        source: H160,
        target: Option<H160>,
        input: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: Option<U256>,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<H256>)>,
        is_transactional: bool,
        weight_limit: Option<Weight>,
        proof_size_base_cost: Option<u64>,
        evm_config: &EvmConfig,
    ) -> Result<(), pallet_evm::RunnerError<Self::Error>> {
        R::validate(
            source,
            target,
            input,
            value,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            nonce,
            access_list,
            is_transactional,
            weight_limit,
            proof_size_base_cost,
            evm_config,
        )
        .map_err(|err| RunnerError {
            error: PermissionedDeployError::Runner(err.error),
            weight: err.weight,
        })
    }

    fn call(
        source: H160,
        target: H160,
        input: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: Option<U256>,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<H256>)>,
        is_transactional: bool,
        validate: bool,
        weight_limit: Option<Weight>,
        proof_size_base_cost: Option<u64>,
        config: &EvmConfig,
    ) -> Result<pallet_evm::CallInfo, pallet_evm::RunnerError<Self::Error>> {
        R::call(
            source,
            target,
            input,
            value,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            nonce,
            access_list,
            is_transactional,
            validate,
            weight_limit,
            proof_size_base_cost,
            config,
        )
        .map_err(|err| RunnerError {
            error: PermissionedDeployError::Runner(err.error),
            weight: err.weight,
        })
    }

    fn create(
        source: H160,
        init: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: Option<U256>,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<H256>)>,
        is_transactional: bool,
        validate: bool,
        weight_limit: Option<Weight>,
        proof_size_base_cost: Option<u64>,
        config: &EvmConfig,
    ) -> Result<pallet_evm::CreateInfo, pallet_evm::RunnerError<Self::Error>> {
        let (_, weight) = T::FeeCalculator::min_gas_price();
        C::check_create_origin(&source).map_err(|err| RunnerError {
            error: PermissionedDeployError::Permission(err),
            weight,
        })?;

        R::create(
            source,
            init,
            value,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            nonce,
            access_list,
            is_transactional,
            validate,
            weight_limit,
            proof_size_base_cost,
            config,
        )
        .map_err(|err| RunnerError {
            error: PermissionedDeployError::Runner(err.error),
            weight: err.weight,
        })
    }

    fn create2(
        source: H160,
        init: Vec<u8>,
        salt: H256,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: Option<U256>,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<H256>)>,
        is_transactional: bool,
        validate: bool,
        weight_limit: Option<Weight>,
        proof_size_base_cost: Option<u64>,
        config: &EvmConfig,
    ) -> Result<pallet_evm::CreateInfo, pallet_evm::RunnerError<Self::Error>> {
        let (_, weight) = T::FeeCalculator::min_gas_price();
        C::check_create_origin(&source).map_err(|err| RunnerError {
            error: PermissionedDeployError::Permission(err),
            weight,
        })?;

        R::create2(
            source,
            init,
            salt,
            value,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            nonce,
            access_list,
            is_transactional,
            validate,
            weight_limit,
            proof_size_base_cost,
            config,
        )
        .map_err(|err| RunnerError {
            error: PermissionedDeployError::Runner(err.error),
            weight: err.weight,
        })
    }

    fn create_force_address(
        source: H160,
        init: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: Option<U256>,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<H256>)>,
        is_transactional: bool,
        validate: bool,
        weight_limit: Option<Weight>,
        proof_size_base_cost: Option<u64>,
        config: &fp_evm::Config,
        force_address: H160,
    ) -> Result<fp_evm::CreateInfo, RunnerError<Self::Error>> {
        let (_, weight) = T::FeeCalculator::min_gas_price();
        C::check_create_origin(&source).map_err(|err| RunnerError {
            error: PermissionedDeployError::Permission(err),
            weight,
        })?;

        R::create_force_address(
            source,
            init,
            value,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            nonce,
            access_list,
            is_transactional,
            validate,
            weight_limit,
            proof_size_base_cost,
            config,
            force_address,
        )
        .map_err(|err| RunnerError {
            error: PermissionedDeployError::Runner(err.error),
            weight: err.weight,
        })
    }
}
#[cfg(test)]
mod mock {
    use super::*;
    use educe::Educe;
    use frame_support::{construct_runtime, derive_impl, weights::Weight};
    use sp_core::{H160, U256};

    construct_runtime!(
        pub enum Test{
            System: frame_system = 0,
            Timestamp: pallet_timestamp = 1,
            Balances: pallet_balances = 2,
            Ethereum: pallet_ethereum = 3,
            Evm: pallet_evm = 4,
        }
    );

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl frame_system::Config for Test {
        type Block = frame_system::mocking::MockBlock<Test>;
        type AccountId = sp_runtime::AccountId32;
        type AccountData = pallet_balances::AccountData<u64>;
        type Lookup = sp_runtime::traits::IdentityLookup<sp_runtime::AccountId32>;
    }

    #[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig)]
    impl pallet_timestamp::Config for Test {}

    #[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
    impl pallet_balances::Config for Test {
        type AccountStore = System;
    }

    #[derive_impl(pallet_ethereum::config_preludes::TestDefaultConfig)]
    impl pallet_ethereum::Config for Test {}

    #[derive_impl(pallet_evm::config_preludes::TestDefaultConfig)]
    impl pallet_evm::Config for Test {
        type AccountProvider = pallet_evm::FrameSystemAccountProvider<Self>;
        type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
        type Currency = Balances;
        type Runner = PermissionedDeploy<Test, MockRunner, MockDeploymentPermissions>;
        type Timestamp = Timestamp;
    }

    mockall::mock! {
        #[derive(Debug)]
        pub DeploymentPermissions {}
        impl crate::EnsureCreateOrigin<crate::runner::mock::Test> for DeploymentPermissions {
            type Error = sp_runtime::DispatchError;

            fn check_create_origin(address: &H160) -> Result<(), sp_runtime::DispatchError>;
        }
    }

    impl PartialEq for MockDeploymentPermissions {
        fn eq(&self, _other: &Self) -> bool {
            true
        }
    }

    impl PartialEq for MockRunner {
        fn eq(&self, _other: &Self) -> bool {
            true
        }
    }

    mockall::mock! {
        #[derive(Debug)]
        pub Runner {}
        impl super::RunnerT<crate::runner::mock::Test> for Runner {
            type Error = sp_runtime::DispatchError;

            fn validate(
                source: H160,
                target: Option<H160>,
                input: Vec<u8>,
                value: U256,
                gas_limit: u64,
                max_fee_per_gas: Option<U256>,
                max_priority_fee_per_gas: Option<U256>,
                nonce: Option<U256>,
                access_list: Vec<(H160, Vec<H256>)>,
                is_transactional: bool,
                weight_limit: Option<Weight>,
                proof_size_base_cost: Option<u64>,
                evm_config: &pallet_evm::EvmConfig,
            ) -> Result<(), RunnerError<sp_runtime::DispatchError>>;

            fn call(
                source: H160,
                target: H160,
                input: Vec<u8>,
                value: U256,
                gas_limit: u64,
                max_fee_per_gas: Option<U256>,
                max_priority_fee_per_gas: Option<U256>,
                nonce: Option<U256>,
                access_list: Vec<(H160, Vec<H256>)>,
                is_transactional: bool,
                validate: bool,
                weight_limit: Option<Weight>,
                proof_size_base_cost: Option<u64>,
                config: &pallet_evm::EvmConfig,
            ) -> Result<pallet_evm::CallInfo, RunnerError<sp_runtime::DispatchError>>;

            fn create(
                source: H160,
                init: Vec<u8>,
                value: U256,
                gas_limit: u64,
                max_fee_per_gas: Option<U256>,
                max_priority_fee_per_gas: Option<U256>,
                nonce: Option<U256>,
                access_list: Vec<(H160, Vec<H256>)>,
                is_transactional: bool,
                validate: bool,
                weight_limit: Option<Weight>,
                proof_size_base_cost: Option<u64>,
                config: &pallet_evm::EvmConfig,
            ) -> Result<pallet_evm::CreateInfo, RunnerError<sp_runtime::DispatchError>>;

            fn create2(
                source: H160,
                init: Vec<u8>,
                salt: H256,
                value: U256,
                gas_limit: u64,
                max_fee_per_gas: Option<U256>,
                max_priority_fee_per_gas: Option<U256>,
                nonce: Option<U256>,
                access_list: Vec<(H160, Vec<H256>)>,
                is_transactional: bool,
                validate: bool,
                weight_limit: Option<Weight>,
                proof_size_base_cost: Option<u64>,
                config: &pallet_evm::EvmConfig,
            ) -> Result<pallet_evm::CreateInfo, RunnerError<sp_runtime::DispatchError>>;

            fn create_force_address(
                source: H160,
                init: Vec<u8>,
                value: U256,
                gas_limit: u64,
                max_fee_per_gas: Option<U256>,
                max_priority_fee_per_gas: Option<U256>,
                nonce: Option<U256>,
                access_list: Vec<(H160, Vec<H256>)>,
                is_transactional: bool,
                validate: bool,
                weight_limit: Option<Weight>,
                proof_size_base_cost: Option<u64>,
                config: &pallet_evm::EvmConfig,
                contract_address: H160,
            ) -> Result<pallet_evm::CreateInfo, RunnerError<sp_runtime::DispatchError>>;
        }

    }

    #[derive(Educe)]
    #[educe(Debug, Default, Clone)]
    pub struct ValidateArgs {
        pub source: H160,
        pub target: Option<H160>,
        pub input: Vec<u8>,
        pub value: U256,
        pub gas_limit: u64,
        pub max_fee_per_gas: Option<U256>,
        pub max_priority_fee_per_gas: Option<U256>,
        pub nonce: Option<U256>,
        pub access_list: Vec<(H160, Vec<H256>)>,
        pub is_transactional: bool,
        pub weight_limit: Option<Weight>,
        pub proof_size_base_cost: Option<u64>,
        #[educe(Default = pallet_evm::EvmConfig::frontier())]
        pub config: pallet_evm::EvmConfig,
    }

    #[derive(Educe)]
    #[educe(Debug, Default, Clone)]
    pub struct CallArgs {
        pub source: H160,
        pub target: H160,
        pub input: Vec<u8>,
        pub value: U256,
        pub gas_limit: u64,
        pub max_fee_per_gas: Option<U256>,
        pub max_priority_fee_per_gas: Option<U256>,
        pub nonce: Option<U256>,
        pub access_list: Vec<(H160, Vec<H256>)>,
        pub is_transactional: bool,
        pub validate: bool,
        pub weight_limit: Option<Weight>,
        pub proof_size_base_cost: Option<u64>,
        #[educe(Default = pallet_evm::EvmConfig::frontier())]
        pub config: pallet_evm::EvmConfig,
    }

    #[derive(Educe)]
    #[educe(Debug, Default, Clone)]
    pub struct CreateArgs {
        pub source: H160,
        pub init: Vec<u8>,
        pub value: U256,
        pub gas_limit: u64,
        pub max_fee_per_gas: Option<U256>,
        pub max_priority_fee_per_gas: Option<U256>,
        pub nonce: Option<U256>,
        pub access_list: Vec<(H160, Vec<H256>)>,
        pub is_transactional: bool,
        pub validate: bool,
        pub weight_limit: Option<Weight>,
        pub proof_size_base_cost: Option<u64>,
        #[educe(Default = pallet_evm::EvmConfig::frontier())]
        pub config: pallet_evm::EvmConfig,
    }

    #[derive(Educe)]
    #[educe(Debug, Default, Clone)]
    pub struct Create2Args {
        pub source: H160,
        pub init: Vec<u8>,
        pub salt: H256,
        pub value: U256,
        pub gas_limit: u64,
        pub max_fee_per_gas: Option<U256>,
        pub max_priority_fee_per_gas: Option<U256>,
        pub nonce: Option<U256>,
        pub access_list: Vec<(H160, Vec<H256>)>,
        pub is_transactional: bool,
        pub validate: bool,
        pub weight_limit: Option<Weight>,
        pub proof_size_base_cost: Option<u64>,
        #[educe(Default = pallet_evm::EvmConfig::frontier())]
        pub config: pallet_evm::EvmConfig,
    }
}

#[cfg(test)]
mod permissioned_runner {
    use fp_evm::UsedGas;
    use pallet_evm::{CallInfo, CreateInfo, ExitReason, ExitSucceed};

    use super::*;
    use std::sync::Mutex;

    static MTX: Mutex<()> = Mutex::new(());

    type PermissionedRunner =
        PermissionedDeploy<mock::Test, mock::MockRunner, mock::MockDeploymentPermissions>;

    const DUMMY_WEIGHT: Weight = Weight::from_parts(42, 24);

    const DUMMY_DISPATCH_ERROR: sp_runtime::DispatchError =
        sp_runtime::DispatchError::Other("dummy error");

    const DUMMY_RUNNER_ERROR: RunnerError<sp_runtime::DispatchError> = RunnerError {
        error: DUMMY_DISPATCH_ERROR,
        weight: DUMMY_WEIGHT,
    };

    const DUMMY_CALL_INFO: CallInfo = CallInfo {
        exit_reason: ExitReason::Succeed(ExitSucceed::Returned),
        value: vec![],
        used_gas: UsedGas {
            standard: U256::zero(),
            effective: U256::zero(),
        },
        weight_info: None,
        logs: vec![],
    };

    const DUMMY_CREATE_INFO: CreateInfo = CreateInfo {
        exit_reason: ExitReason::Succeed(ExitSucceed::Returned),
        value: H160::zero(),
        used_gas: UsedGas {
            standard: U256::zero(),
            effective: U256::zero(),
        },
        weight_info: None,
        logs: vec![],
    };

    mod validate_method {
        use super::*;
        use crate::runner::mock::ValidateArgs;

        #[test]
        fn is_permissionless() {
            let _m = MTX.lock();

            let ctx_runner = mock::MockRunner::validate_context();
            ctx_runner
                .expect()
                .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _| Ok(()));

            let ctx_deployment_permissions =
                mock::MockDeploymentPermissions::check_create_origin_context();
            ctx_deployment_permissions
                .expect()
                .returning(|_| Err(DUMMY_DISPATCH_ERROR));

            let params = ValidateArgs::default();

            assert!(PermissionedRunner::validate(
                params.source,
                params.target,
                params.input,
                params.value,
                params.gas_limit,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
                params.nonce,
                params.access_list,
                params.is_transactional,
                params.weight_limit,
                params.proof_size_base_cost,
                &params.config,
            )
            .is_ok());
        }

        #[test]
        fn routes_underlying_ok() {
            let _m = MTX.lock();

            let ctx_runner = mock::MockRunner::validate_context();
            ctx_runner
                .expect()
                .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _| Ok(()))
                .once();

            let params = ValidateArgs::default();

            assert!(PermissionedRunner::validate(
                params.source,
                params.target,
                params.input,
                params.value,
                params.gas_limit,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
                params.nonce,
                params.access_list,
                params.is_transactional,
                params.weight_limit,
                params.proof_size_base_cost,
                &params.config,
            )
            .is_ok());
        }

        #[test]
        fn routes_underlying_err() {
            let _m = MTX.lock();

            let ctx_runner = mock::MockRunner::validate_context();
            ctx_runner
                .expect()
                .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _| Err(DUMMY_RUNNER_ERROR))
                .once();

            let params = ValidateArgs::default();

            let RunnerError { error, weight } = PermissionedRunner::validate(
                params.source,
                params.target,
                params.input,
                params.value,
                params.gas_limit,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
                params.nonce,
                params.access_list,
                params.is_transactional,
                params.weight_limit,
                params.proof_size_base_cost,
                &params.config,
            )
            .unwrap_err();
            assert_eq!(
                error,
                PermissionedDeployError::Runner(DUMMY_RUNNER_ERROR.error)
            );
            assert_eq!(weight, DUMMY_RUNNER_ERROR.weight);
        }
    }

    mod call_method {
        use super::*;
        use crate::runner::mock::CallArgs;

        #[test]
        fn is_permissionless() {
            let _m = MTX.lock();

            let ctx_runner = mock::MockRunner::call_context();
            ctx_runner
                .expect()
                .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _, _| Ok(DUMMY_CALL_INFO));

            let ctx_deployment_permissions =
                mock::MockDeploymentPermissions::check_create_origin_context();
            ctx_deployment_permissions
                .expect()
                .returning(|_| Err(DUMMY_DISPATCH_ERROR));

            let params = CallArgs::default();

            assert!(PermissionedRunner::call(
                params.source,
                params.target,
                params.input,
                params.value,
                params.gas_limit,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
                params.nonce,
                params.access_list,
                params.is_transactional,
                params.validate,
                params.weight_limit,
                params.proof_size_base_cost,
                &params.config,
            )
            .is_ok());
        }

        #[test]
        fn routes_underlying_ok() {
            let _m = MTX.lock();

            let ctx_runner = mock::MockRunner::call_context();
            ctx_runner
                .expect()
                .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _, _| Ok(DUMMY_CALL_INFO))
                .once();

            let params = CallArgs::default();

            let execution_info = PermissionedRunner::call(
                params.source,
                params.target,
                params.input,
                params.value,
                params.gas_limit,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
                params.nonce,
                params.access_list,
                params.is_transactional,
                params.validate,
                params.weight_limit,
                params.proof_size_base_cost,
                &params.config,
            )
            .unwrap();
            assert!(execution_info == DUMMY_CALL_INFO)
        }

        #[test]
        fn routes_underlying_err() {
            let _m = MTX.lock();

            let ctx_runner = mock::MockRunner::call_context();
            ctx_runner
                .expect()
                .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _, _| Err(DUMMY_RUNNER_ERROR))
                .once();

            let params = CallArgs::default();

            let RunnerError { error, weight } = PermissionedRunner::call(
                params.source,
                params.target,
                params.input,
                params.value,
                params.gas_limit,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
                params.nonce,
                params.access_list,
                params.is_transactional,
                params.validate,
                params.weight_limit,
                params.proof_size_base_cost,
                &params.config,
            )
            .unwrap_err();
            assert_eq!(
                error,
                PermissionedDeployError::Runner(DUMMY_RUNNER_ERROR.error)
            );
            assert_eq!(weight, DUMMY_RUNNER_ERROR.weight);
        }
    }

    mod create_method {
        use super::*;
        use crate::runner::mock::CreateArgs;

        #[test]
        fn is_permissioned() {
            let _m = MTX.lock();

            let ctx_deployment_permissions =
                mock::MockDeploymentPermissions::check_create_origin_context();
            ctx_deployment_permissions
                .expect()
                .returning(|_| Err(DUMMY_DISPATCH_ERROR));

            let params = CreateArgs::default();

            let RunnerError { error, weight } = PermissionedRunner::create(
                params.source,
                params.init,
                params.value,
                params.gas_limit,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
                params.nonce,
                params.access_list,
                params.is_transactional,
                params.validate,
                params.weight_limit,
                params.proof_size_base_cost,
                &params.config,
            )
            .unwrap_err();
            assert_eq!(
                error,
                PermissionedDeployError::Permission(DUMMY_DISPATCH_ERROR)
            );
            assert_eq!(weight, Weight::zero());
        }

        #[test]
        fn routes_underlying_ok() {
            let _m = MTX.lock();

            let ctx_runner = mock::MockRunner::create_context();
            ctx_runner
                .expect()
                .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _| Ok(DUMMY_CREATE_INFO))
                .once();

            let ctx_deployment_permissions =
                mock::MockDeploymentPermissions::check_create_origin_context();
            ctx_deployment_permissions.expect().returning(|_| Ok(()));

            let params = CreateArgs::default();

            let create_info = PermissionedRunner::create(
                params.source,
                params.init,
                params.value,
                params.gas_limit,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
                params.nonce,
                params.access_list,
                params.is_transactional,
                params.validate,
                params.weight_limit,
                params.proof_size_base_cost,
                &params.config,
            )
            .unwrap();
            assert!(create_info == DUMMY_CREATE_INFO);
        }

        #[test]
        fn routes_underlying_err() {
            let _m = MTX.lock();

            let ctx_runner = mock::MockRunner::create_context();
            ctx_runner
                .expect()
                .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _| Err(DUMMY_RUNNER_ERROR))
                .times(1);

            let ctx_deployment_permissions =
                mock::MockDeploymentPermissions::check_create_origin_context();
            ctx_deployment_permissions.expect().returning(|_| Ok(()));

            let params = CreateArgs::default();

            let RunnerError { error, weight } = PermissionedRunner::create(
                params.source,
                params.init,
                params.value,
                params.gas_limit,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
                params.nonce,
                params.access_list,
                params.is_transactional,
                params.validate,
                params.weight_limit,
                params.proof_size_base_cost,
                &params.config,
            )
            .unwrap_err();
            assert_eq!(
                error,
                PermissionedDeployError::Runner(DUMMY_RUNNER_ERROR.error)
            );
            assert_eq!(weight, DUMMY_RUNNER_ERROR.weight);
        }
    }

    mod create2_method {
        use super::*;
        use crate::runner::mock::Create2Args;

        #[test]
        fn is_permissioned() {
            let _m = MTX.lock();

            let ctx_deployment_permissions =
                mock::MockDeploymentPermissions::check_create_origin_context();

            ctx_deployment_permissions
                .expect()
                .returning(|_| Err(DUMMY_DISPATCH_ERROR));

            let params = Create2Args::default();

            let RunnerError { error, weight } = PermissionedRunner::create2(
                params.source,
                params.init,
                params.salt,
                params.value,
                params.gas_limit,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
                params.nonce,
                params.access_list,
                params.is_transactional,
                params.validate,
                params.weight_limit,
                params.proof_size_base_cost,
                &params.config,
            )
            .unwrap_err();
            assert_eq!(
                error,
                PermissionedDeployError::Permission(DUMMY_DISPATCH_ERROR)
            );
            assert_eq!(weight, Weight::zero());
        }

        #[test]
        fn routes_underlying_ok() {
            let _m = MTX.lock();

            let ctx_runner = mock::MockRunner::create2_context();
            ctx_runner
                .expect()
                .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _, _| Ok(DUMMY_CREATE_INFO))
                .once();

            let ctx_deployment_permissions =
                mock::MockDeploymentPermissions::check_create_origin_context();
            ctx_deployment_permissions.expect().returning(|_| Ok(()));

            let params = Create2Args::default();

            let create2_info = PermissionedRunner::create2(
                params.source,
                params.init,
                params.salt,
                params.value,
                params.gas_limit,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
                params.nonce,
                params.access_list,
                params.is_transactional,
                params.validate,
                params.weight_limit,
                params.proof_size_base_cost,
                &params.config,
            )
            .unwrap();
            assert!(create2_info == DUMMY_CREATE_INFO);
        }

        #[test]
        fn routes_underlying_err() {
            let _m = MTX.lock();

            let ctx_runner = mock::MockRunner::create2_context();
            ctx_runner
                .expect()
                .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _, _| Err(DUMMY_RUNNER_ERROR))
                .once();

            let ctx_deployment_permissions =
                mock::MockDeploymentPermissions::check_create_origin_context();
            ctx_deployment_permissions.expect().returning(|_| Ok(()));

            let params = Create2Args::default();

            let RunnerError { error, weight } = PermissionedRunner::create2(
                params.source,
                params.init,
                params.salt,
                params.value,
                params.gas_limit,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
                params.nonce,
                params.access_list,
                params.is_transactional,
                params.validate,
                params.weight_limit,
                params.proof_size_base_cost,
                &params.config,
            )
            .unwrap_err();
            assert_eq!(
                error,
                PermissionedDeployError::Runner(DUMMY_RUNNER_ERROR.error)
            );
            assert_eq!(weight, DUMMY_RUNNER_ERROR.weight);
        }
    }
}
