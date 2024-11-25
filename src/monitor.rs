use colored::Colorize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::watch;

use tracing::{debug, error, info, warn};

use crate::{
    helpers::Sender,
    types::{Data, MonError, RuntimeConfig},
};

pub async fn monitor(
    config: RuntimeConfig,
    monitoring_sender: watch::Sender<Data>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), MonError> {
    let sender = Sender {
        token: config.tg_api_key,
        chat_id: config.tg_chat_id,
    };

    loop {
        // 1. Get the current block and "block-window" blocks in the past
        let block_num = config.rpc_client.latest_block_id().await?;
        let block_range_start = block_num - config.block_window as u64;
        let block_range = block_range_start..block_num;

        let mut ssa_tx_found = false;
        let mut sa_tx_found = false;
        let mut spa_tx_found = false;

        let colored_output = format!(
            "Starting block_id: {block_num} and going back {} blocks",
            config.block_window
        )
        .green()
        .underline();
        info!("{colored_output}");

        // Get all the balances
        let signing_policy_balance = config
            .rpc_client
            .get_balance(config.signing_policy_address.clone())
            .await?
            / 10f64.powf(18.0);
        let submit_balance = config
            .rpc_client
            .get_balance(config.submit_address.clone())
            .await?
            / 10f64.powf(18.0);
        let submit_signature_balance = config
            .rpc_client
            .get_balance(config.submit_signature_address.clone())
            .await?
            / 10f64.powf(18.0);

        let output = format!("SPA, SA, SSA balances: {signing_policy_balance}, {submit_balance}, {submit_signature_balance}").yellow();
        info!("{output}");

        // Create alerts for the following
        // TODO: If a register tx was not made from the signing policy address for this epoch
        // TODO: If there weren't 2 txs made from the submit address within 90
        // TODO: If there isn't 1 tx made by the signing policy address every 90s
        // TODO: If there isn't 1 tx made every 90s submit sig address

        // 2. for each block id
        // Go through each block in the block_range
        for block_id in block_range {
            // info!("{} found", block_id);

            // Create a new task for analyzing the txs for each block found
            let rpc_client_clone = config.rpc_client.clone();

            // Start of new thread
            // 3. get the block's contents
            let block = match rpc_client_clone.get_block(block_id).await {
                Ok(block) => block,
                Err(err) => {
                    let output = format!(
                        "Couldn't get block contents of {} because {:?}!",
                        block_id, err
                    )
                    .red();
                    error!("{output}");
                    None
                }
            };

            // If block is not found, move on
            let Some(block) = block else {
                warn!("Couldn't get block {}... Moving on...", block_id);
                continue;
            };

            // 4. Get all transactions
            let block_txs = block.transactions;

            // 5. Go through each tx and find the ones for the ftso
            for mut tx in block_txs {
                let from_address = format!("{:?}", tx.from.take().unwrap());

                if from_address
                    .to_lowercase()
                    .eq(&config.submit_signature_address.to_lowercase())
                {
                    ssa_tx_found = true;
                }

                if from_address
                    .to_lowercase()
                    .eq(&config.signing_policy_address.to_lowercase())
                {
                    spa_tx_found = true;
                }

                if from_address
                    .to_lowercase()
                    .eq(&config.submit_address.to_lowercase())
                {
                    sa_tx_found = true;
                }

                if stop_flag.load(Ordering::Relaxed) {
                    return Ok(());
                }
            }
        }

        // If a tx from submit signature address was not found
        if !ssa_tx_found {
            info!(
                "{}",
                String::from("Sent message for Submit Signature Address not signing!").red()
            );
            _ = sender
                .send_message(format!(
                    "v2: Submit Signature Address has not signed for {} blocks!",
                    config.block_window
                ))
                .await;
        }

        // If a tx from submit address was not found
        if !sa_tx_found {
            info!(
                "{}",
                String::from("Sent message for Submit Address not signing!").red()
            );
            _ = sender
                .send_message(format!(
                    "v2: Submit Address has not signed for {} blocks!",
                    config.block_window
                ))
                .await;
        }

        // If a tx from signing policy address was not found
        if !spa_tx_found {
            info!(
                "{}",
                String::from("Sent message for Signing Policy Address not signing!").red()
            );
            _ = sender
                .send_message(format!(
                    "v2: Signing Policy Address has not signed for {} blocks!",
                    config.block_window
                ))
                .await;
        }

        // Gather all the relevant data
        let data = Data {
            signing_policy_tx_found: spa_tx_found,
            signing_policy_balance,
            submit_tx_found: sa_tx_found,
            submit_balance,
            submit_signature_tx_found: ssa_tx_found,
            submit_signature_balance,
        };

        // Send it to the metrics task
        match monitoring_sender.send(data.clone()) {
            Ok(_) => {
                debug!("Sent data: {data:?}");
            }
            Err(e) => {
                error!("{}", format!("Couldn't send to metrics task: {e:?}").red());
            }
        }
    }
}
