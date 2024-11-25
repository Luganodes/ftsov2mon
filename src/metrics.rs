use anyhow::Context;
use prometheus::{Encoder, Gauge, IntGauge, Registry, TextEncoder};
use tracing::{debug, error};

use crate::{rpc::RpcClient, types::MonError};

#[derive(Debug, Clone)]
pub struct Metrics {
    pub signing_policy_tx_found: IntGauge,
    pub signing_policy_balance: Gauge,

    pub submit_tx_found: IntGauge,
    pub submit_balance: Gauge,

    pub submit_signature_tx_found: IntGauge,
    pub submit_signature_balance: Gauge,

    pub registered_for_this_epoch: IntGauge,
    pub is_syncing: IntGauge,
    pub rpc_current_block: Gauge,
    registry: Registry,
}

impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            signing_policy_tx_found: IntGauge::new(
                "ftso_signing_policy_tx_found",
                "Was a tx from the signing policy address found within the block window?",
            )
            .unwrap(),
            signing_policy_balance: Gauge::new(
                "ftso_signing_policy_balance",
                "The balance of the signing policy address",
            )
            .unwrap(),

            submit_tx_found: IntGauge::new(
                "ftso_submit_tx_found",
                "Was a tx from the submit address found within the block window?",
            )
            .unwrap(),
            submit_balance: Gauge::new("ftso_submit_balance", "The balance of the submit address")
                .unwrap(),

            submit_signature_tx_found: IntGauge::new(
                "ftso_submit_signature_tx_found",
                "Was a tx from the submit signature address found within the block window?",
            )
            .unwrap(),
            submit_signature_balance: Gauge::new(
                "ftso_submit_signature_balance",
                "The balance of the submit signature address",
            )
            .unwrap(),

            registered_for_this_epoch: IntGauge::new(
                "ftso_registered_for_this_epoch",
                "Did the client register for this reward epoch?",
            )
            .unwrap(),
            is_syncing: IntGauge::new("ftso_rpc_is_syncing", "Is the RPC syncing?").unwrap(),
            rpc_current_block: Gauge::new(
                "ftso_rpc_current_block",
                "The latest block from the RPC",
            )
            .unwrap(),
            registry: Registry::new(),
        }
    }

    pub fn register(&self) -> Result<(), MonError> {
        self.registry
            .register(Box::new(self.signing_policy_tx_found.clone()))
            .context("Couldn't register signing_policy_tx_found")
            .map_err(|e| MonError::RegisterError(e.into()))?;
        self.registry
            .register(Box::new(self.signing_policy_balance.clone()))
            .context("Couldn't register signing_policy_balance")
            .map_err(|e| MonError::RegisterError(e.into()))?;
        self.registry
            .register(Box::new(self.submit_tx_found.clone()))
            .context("Couldn't register submit_tx_found")
            .map_err(|e| MonError::RegisterError(e.into()))?;
        self.registry
            .register(Box::new(self.submit_balance.clone()))
            .context("Couldn't register submit_balance")
            .map_err(|e| MonError::RegisterError(e.into()))?;
        self.registry
            .register(Box::new(self.submit_signature_tx_found.clone()))
            .context("Couldn't register submit_signature_tx_found")
            .map_err(|e| MonError::RegisterError(e.into()))?;
        self.registry
            .register(Box::new(self.submit_signature_balance.clone()))
            .context("Couldn't register submit_signature_balance")
            .map_err(|e| MonError::RegisterError(e.into()))?;
        self.registry
            .register(Box::new(self.registered_for_this_epoch.clone()))
            .context("Couldn't register registered_for_this_epoch")
            .map_err(|e| MonError::RegisterError(e.into()))?;
        self.registry
            .register(Box::new(self.is_syncing.clone()))
            .context("Couldn't register is_syncing")
            .map_err(|e| MonError::RegisterError(e.into()))?;
        self.registry
            .register(Box::new(self.rpc_current_block.clone()))
            .context("Couldn't register rpc_current_block")
            .map_err(|e| MonError::RegisterError(e.into()))?;
        Ok(())
    }

    pub fn get_encoder_and_buffer(&self) -> Result<(TextEncoder, Vec<u8>), MonError> {
        let encoder = TextEncoder::new();

        let metric_families = self.registry.gather();
        let mut buffer = vec![];
        encoder
            .encode(&metric_families, &mut buffer)
            .context("Couldn't encode metric families")
            .map_err(|e| MonError::EncodeError(e.into()))?;

        Ok((encoder, buffer))
    }

    pub async fn update_for_rpc(&self, rpc_client: &RpcClient) -> Result<(), MonError> {
        debug!("Updating metrics for RPC");

        // If the RPC starts malfunctioning, syncing should go to false
        // the rest should just stop updating but not error out
        let mut is_syncing = true;
        match rpc_client.syncing_info().await {
            Ok(sync_info) => {
                if sync_info.is_none() {
                    is_syncing = false;
                } else {
                    debug!("{sync_info:?}");
                }

                if is_syncing {
                    self.is_syncing.set(1);
                } else {
                    self.is_syncing.set(0);
                }
            }
            Err(err) => {
                error!("{err:?}");
            }
        };

        // Set the latest block from the RPC
        let current_block = rpc_client.current_block().await?;
        self.rpc_current_block.set(current_block as f64);

        Ok(())
    }

    pub async fn update_for_monitoring_data(
        &self,
        data: &crate::types::Data,
    ) -> Result<(), MonError> {
        self.signing_policy_balance.set(data.signing_policy_balance);
        self.submit_balance.set(data.submit_balance);
        self.submit_signature_balance
            .set(data.submit_signature_balance);

        if data.submit_signature_tx_found {
            self.submit_signature_tx_found.set(1);
        } else {
            self.submit_signature_tx_found.set(0);
        }

        if data.signing_policy_tx_found {
            self.signing_policy_tx_found.set(1);
        } else {
            self.signing_policy_tx_found.set(0);
        }

        if data.submit_tx_found {
            self.submit_tx_found.set(1);
        } else {
            self.submit_tx_found.set(0);
        }
        Ok(())
    }
}
