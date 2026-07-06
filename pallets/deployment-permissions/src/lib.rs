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

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unused_crate_dependencies)]

extern crate alloc;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
pub mod runner;
#[cfg(test)]
mod tests;
pub mod weights;

pub use crate::weights::WeightInfo;
use alloc::vec::Vec;
use frame_support::sp_runtime::DispatchError;
pub use pallet::*;
use sp_core::H160;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*, DefaultNoBound};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Deploy permission has been granted to an address.
        DeployPermissionGranted {
            /// The address to which deploy permission has been granted.
            address: H160,
        },
        /// Deploy permission has been revoked from an address.
        DeployPermissionRevoked {
            /// The address from which deploy permission has been revoked.
            address: H160,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Tried to grant deploy permission to an address already having it.
        AddressAlreadyHasDeployPermission,
        /// Tried to revoke deploy permission from an address not having it.
        AddressDoesNotHaveDeployPermission,
    }

    #[pallet::storage]
    pub type Deployers<T> = StorageMap<_, Blake2_128Concat, H160, (), OptionQuery>;

    #[pallet::genesis_config]
    #[derive(DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub deployers: Vec<H160>,
        _phantom: PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            self.deployers.iter().for_each(|deployer| {
                Deployers::<T>::insert(deployer, ());
            });
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::grant_deploy_permission())]
        pub fn grant_deploy_permission(origin: OriginFor<T>, address: H160) -> DispatchResult {
            ensure_root(origin)?;
            if !Deployers::<T>::contains_key(address) {
                Deployers::<T>::insert(address, ());
                Self::deposit_event(Event::<T>::DeployPermissionGranted { address });
                Ok(())
            } else {
                Err(Error::<T>::AddressAlreadyHasDeployPermission)?
            }
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::revoke_deploy_permission())]
        pub fn revoke_deploy_permission(origin: OriginFor<T>, address: H160) -> DispatchResult {
            ensure_root(origin)?;
            if Deployers::<T>::contains_key(address) {
                Deployers::<T>::remove(address);
                Self::deposit_event(Event::<T>::DeployPermissionRevoked { address });
                Ok(())
            } else {
                Err(Error::<T>::AddressDoesNotHaveDeployPermission)?
            }
        }
    }
}

impl<T: Config> EnsureCreateOrigin<T> for Pallet<T> {
    type Error = DispatchError;

    fn check_create_origin(address: &H160) -> Result<(), Self::Error> {
        if Deployers::<T>::contains_key(address) {
            Ok(())
        } else {
            Err(DispatchError::Other("Not allowed to deploy"))
        }
    }
}

pub trait EnsureCreateOrigin<T> {
    type Error: Into<DispatchError>;

    fn check_create_origin(address: &H160) -> Result<(), Self::Error>;
}
