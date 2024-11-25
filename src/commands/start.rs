use colored::Colorize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::watch;

use clap::ArgMatches;
use tracing::{error, info};
use web3::futures::future::join_all;

use crate::{
    monitor,
    rpc::RpcClient,
    server,
    types::{Data, MonError, RuntimeConfig},
};

pub async fn start(args: &ArgMatches) -> Result<(), MonError> {
    let tg_api_key = args
        .get_one::<String>("tg-api-key")
        .map(|s| s.clone())
        .unwrap();
    let tg_chat_id = args
        .get_one::<String>("tg-chat-id")
        .map(|s| s.clone())
        .clone()
        .unwrap();
    let metrics_port = args.get_one::<u16>("metrics-port").copied().unwrap();
    let metrics_addr = args.get_one::<String>("metrics-addr").unwrap().to_string();
    let rpc_url = args.get_one::<String>("rpc-url").unwrap().to_string();
    let block_window = *args.get_one::<u16>("block-window").unwrap();
    let submit_address = args
        .get_one::<String>("submit-address")
        .unwrap()
        .to_string();
    let submit_signature_address = args
        .get_one::<String>("submit-signature-address")
        .unwrap()
        .to_string();
    let signing_policy_address = args
        .get_one::<String>("signing-policy-address")
        .unwrap()
        .to_string();

    info!("===================");
    info!("Args found: ");
    info!("--tg-api-key: {:?}", tg_api_key);
    info!("--tg-chat-id: {:?}", tg_chat_id);
    info!("--metrics-port: {}", metrics_port);
    info!("--metrics-addr: {}", metrics_addr);
    info!("--rpc-url: {}", rpc_url);
    info!("--block-window: {}", block_window);
    info!("--submit-address: {}", submit_address);
    info!("--submit-signature-address: {}", submit_signature_address);
    info!("--signing-policy-address: {}", signing_policy_address);
    info!("===================");

    let rpc_client = RpcClient::new(rpc_url.clone())?;
    let config = RuntimeConfig {
        // Create the runtime config
        tg_api_key,
        tg_chat_id,
        rpc_client,
        block_window,
        submit_address,
        submit_signature_address,
        signing_policy_address,
    };
    let config_clone = config.clone();

    let stop_flag = Arc::new(AtomicBool::new(false));
    let (s, rx1) = watch::channel::<Data>(Data {
        signing_policy_tx_found: false,
        signing_policy_balance: 0.0,
        submit_tx_found: false,
        submit_balance: 0.0,
        submit_signature_tx_found: false,
        submit_signature_balance: 0.0,
    });

    let stop_flag_clone = stop_flag.clone();

    // Start the monitoring here
    // And share the monitoring finding with the metrics server via a broadcast channel
    info!("{}", String::from("Starting monitoring thread...").green());
    let monitor_handle = tokio::spawn(async move {
        match monitor(config_clone, s, stop_flag_clone).await {
            Ok(_) => {
                let output = String::from("Monitoring server stopped gracefully...").green();
                info!("{output}");
            }
            Err(err) => {
                let output = format!("Monitoring error: {err:?}");
                error!("{output}");
            }
        }
    });

    info!(
        "{}",
        String::from("Starting metrics server thread...").green()
    );
    let server_handle = tokio::spawn(async move {
        match server::run(metrics_addr, metrics_port, rpc_url, config, rx1)
            .unwrap()
            .await
        {
            Ok(_) => {
                let output = String::from("Metrics server stopped gracefully...").green();
                info!("{output}");
                stop_flag.store(true, Ordering::Relaxed);
            }
            Err(err) => {
                let output = format!("Metrics server error: {err:?}").green();
                error!("{output}");
            }
        }
    });

    // Wait for both the threads to finish
    _ = join_all([monitor_handle, server_handle]).await;

    Ok(())
}
