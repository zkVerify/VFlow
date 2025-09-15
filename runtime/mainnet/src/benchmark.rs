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

frame_benchmarking::define_benchmarks!(
    [frame_system, SystemBench::<Runtime>]
    [frame_system_extensions, SystemExtensionsBench::<Runtime>]
    [cumulus_pallet_parachain_system, ParachainSystem]
    [pallet_timestamp, Timestamp]
    [pallet_proxy, Proxy]
    [pallet_utility, Utility]
    [pallet_multisig, Multisig]
    [pallet_transaction_payment, TransactionPayment]

    [pallet_balances, Balances]

    [pallet_sudo, Sudo]

    [pallet_collator_selection, CollatorSelection]
    [pallet_session, SessionBench::<Runtime>]

    [cumulus_pallet_xcmp_queue, XcmpQueue]
    [pallet_xcm, PalletXcmExtrinsicsBenchmark::<Runtime>]
    [pallet_message_queue, MessageQueue]

    [pallet_evm, EVM]
    [pallet_deployment_permissions, DeploymentPermissions]

    [pallet_deployment_permissions, DeploymentPermissions]
    [pallet_xcm_benchmarks::generic, xcm::XcmPalletBenchGeneric::<Runtime>]
    [pallet_xcm_benchmarks::fungible, xcm::XcmPalletBenchFungible::<Runtime>]
);
