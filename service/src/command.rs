///////////////////////////////////////////////////////////////////////////////
//
//  Copyright 2018-2021 Robonomics Network <research@robonomics.network>
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

use crate::cli::{Cli, Subcommand};

#[cfg(feature = "full")]
use crate::{chain_spec::*, service::robonomics};
use robonomics_protocol::id;
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};
use std::path::Path;

#[cfg(feature = "discovery")]
use libp2p::{
    core::upgrade,
    futures::StreamExt,
    gossipsub, mdns, mplex, noise,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp, Multiaddr, PeerId, Transport,
};
use robonomics_protocol::network::{behaviour::RobonomicsNetworkBehaviour, discovery};

#[cfg(feature = "parachain")]
use crate::parachain;

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "airalab-robonomics".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/airalab/robonomics/issues/new".into()
    }

    fn copyright_start_year() -> i32 {
        2018
    }

    fn executable_name() -> String {
        "robonomics".into()
    }

    #[cfg(feature = "full")]
    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(development_config()),
            #[cfg(feature = "parachain")]
            path => parachain::load_spec(path, self.parachain_id.unwrap_or(2048).into())?,
            #[cfg(not(feature = "parachain"))]
            path => Box::new(crate::chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }

    #[cfg(not(feature = "full"))]
    fn load_spec(&self, _id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
        Err("Chain spec isn't supported for zero build")?
    }

    #[cfg(feature = "full")]
    fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        match chain_spec.family() {
            RobonomicsFamily::Development => &local_runtime::VERSION,
            #[cfg(feature = "parachain")]
            RobonomicsFamily::Alpha => &alpha_runtime::VERSION,
            #[cfg(feature = "parachain")]
            RobonomicsFamily::Ipci => &ipci_runtime::VERSION,
            #[cfg(feature = "kusama")]
            RobonomicsFamily::Main => &main_runtime::VERSION,
        }
    }

    #[cfg(not(feature = "full"))]
    fn native_runtime_version(_chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        unimplemented!()
    }
}

#[cfg(feature = "discovery")]
pub async fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();
    let local_key = cli.local_key_file.map_or(id::random(), |file_name| {
        id::load(Path::new(&file_name)).expect("Correct file path")
    });
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let transport =
        libp2p::tokio_development_transport(local_key.clone()).expect("Correct transport");
    let behaviour = RobonomicsNetworkBehaviour::new(local_key, local_peer_id, 1000, true, true)
        .expect("Correct behaviour");
    // ???
    let mut swarm = SwarmBuilder::new(transport, behaviour, local_peer_id)
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();

    // discovery::add_peers(&mut swarm, cli.robonomics_bootnodes);

    // use libp2p::Multiaddr;
    // if let Some(to_dial) = std::env::args().nth(1) {
    //     let addr: Multiaddr = to_dial.parse()?;
    //     swarm.dial(addr.clone())?;
    //     println!("Dialed {to_dial:?}");
    //
    //     if let Some(peer) = PeerId::try_from_multiaddr(&addr) {
    //         swarm.behaviour_mut().pubsub.add_explicit_peer(&peer);
    //     }
    // }

    swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .expect("Swarm listen");

    loop {
        tokio::select! {
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {:?}", address),
                SwarmEvent::Behaviour(event) =>
                {
                    println!("Event: {:?}", event);
                    for p in swarm.connected_peers() {
                        println!("Connected peer: {p}");
                    }
                }
                _ => {}
            }
        }
    }
}

/// Parse command line arguments into service configuration.
#[cfg(not(feature = "discovery"))]
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        #[cfg(not(feature = "full"))]
        None => Ok(()),
        #[cfg(feature = "full")]
        None => {
            let runner = cli.create_runner(&cli.run.normalize())?;
            let collator_options = cli.run.collator_options();

            // Get local key
            let local_key = cli.local_key_file.map_or(id::random(), |file_name| {
                id::load(Path::new(&file_name)).expect("Correct file path")
            });

            // Default interval 1 sec
            let heartbeat_interval = cli.heartbeat_interval.unwrap_or_else(|| 1000);

            match runner.config().chain_spec.family() {
                RobonomicsFamily::Development => runner.run_node_until_exit(|config| async move {
                    robonomics::new(
                        config,
                        local_key,
                        heartbeat_interval,
                        cli.robonomics_bootnodes,
                        cli.disable_mdns,
                        cli.disable_kad,
                    )
                }),

                #[cfg(feature = "parachain")]
                RobonomicsFamily::Alpha => runner.run_node_until_exit(|config| async move {
                    let params = parachain::command::parse_args(
                        config,
                        &cli.relaychain_args,
                        cli.parachain_id,
                        cli.lighthouse_account,
                    )?;

                    parachain::alpha::start_node(
                        params.0,
                        params.1,
                        collator_options,
                        params.2,
                        params.3,
                        local_key,
                        heartbeat_interval,
                        cli.robonomics_bootnodes,
                        cli.disable_mdns,
                        cli.disable_kad,
                    )
                    .await
                }),

                #[cfg(feature = "parachain")]
                RobonomicsFamily::Ipci => runner.run_node_until_exit(|config| async move {
                    let params = parachain::command::parse_args(
                        config,
                        &cli.relaychain_args,
                        cli.parachain_id,
                        cli.lighthouse_account,
                    )?;

                    parachain::ipci::start_node(
                        params.0,
                        params.1,
                        collator_options,
                        params.2,
                        params.3,
                        local_key,
                        heartbeat_interval,
                        cli.robonomics_bootnodes,
                        cli.disable_mdns,
                        cli.disable_kad,
                    )
                    .await
                }),

                #[cfg(feature = "kusama")]
                RobonomicsFamily::Main => runner.run_node_until_exit(|config| async move {
                    let params = parachain::command::parse_args(
                        config,
                        &cli.relaychain_args,
                        cli.parachain_id,
                        cli.lighthouse_account,
                    )?;

                    parachain::main::start_node(
                        params.0,
                        params.1,
                        collator_options,
                        params.2,
                        params.3,
                        local_key,
                        heartbeat_interval,
                        cli.robonomics_bootnodes,
                        cli.disable_mdns,
                        cli.disable_kad,
                    )
                    .await
                }),
            }
            .map_err(Into::into)
        }
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::Sign(cmd)) => cmd.run(),
        Some(Subcommand::Verify(cmd)) => cmd.run(),
        Some(Subcommand::Vanity(cmd)) => cmd.run(),
        #[cfg(feature = "full")]
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        #[cfg(feature = "full")]
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }

        // #[cfg(feature = "full")]
        // Some(Subcommand::Pair(cmd)) => match &cmd.subcommand {
        //     Some(robonomics_pair::pair::PairSubCmds::Connect(cmd)) => {
        //         robonomics_pair::pair::ConnectCmd::run(cmd).map_err(|e| e.to_string().into())
        //     }
        //     Some(robonomics_pair::pair::PairSubCmds::Listen(cmd)) => {
        //         robonomics_pair::pair::ListenCmd::run(cmd).map_err(|e| e.to_string().into())
        //     }
        //     _ => {
        //         println!("pair args {:?}", cmd);
        //         Ok(())
        //     }
        // },
        #[cfg(feature = "robonomics-cli")]
        Some(Subcommand::Io(cmd)) => {
            let runner = cli.create_runner(&*cli.run)?;
            runner.sync_run(|_| cmd.run().map_err(|e| e.to_string().into()))
        }
        #[cfg(feature = "frame-benchmarking-cli")]
        Some(Subcommand::Benchmark(subcommand)) => {
            let runner = cli.create_runner(subcommand)?;
            match runner.config().chain_spec.family() {
                RobonomicsFamily::Development => runner.sync_run(|config| {
                    subcommand
                        .run::<robonomics_primitives::Block, robonomics::LocalExecutor>(config)
                }),
            }
        }
        #[cfg(feature = "parachain")]
        Some(Subcommand::ExportGenesisState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|_| {
                let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
                let state_version = Cli::native_runtime_version(&spec).state_version();
                cmd.run::<robonomics_primitives::Block>(&*spec, state_version)
            })
        }
        #[cfg(feature = "parachain")]
        Some(Subcommand::ExportGenesisWasm(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|_| {
                let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
                cmd.run(&*spec)
            })
        }
    }
}
