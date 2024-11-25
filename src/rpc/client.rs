use std::str::FromStr;

use anyhow::Context;
use web3::{
    transports::Http,
    types::{Block, BlockId, BlockNumber, SyncInfo, Transaction, TransactionId, H160, H256},
    Web3,
};

use crate::types::MonError;

#[derive(Debug, Clone)]
pub struct RpcClient {
    client: Web3<Http>,
    pub rpc_url: String,
}

impl RpcClient {
    pub fn new(rpc_url: String) -> Result<RpcClient, MonError> {
        let transport = web3::transports::Http::new(rpc_url.as_str())
            .context("Unable to get web3 transport!")
            .map_err(|e| MonError::RpcClientError(e))?;
        let web3 = web3::Web3::new(transport);

        Ok(RpcClient {
            client: web3,
            rpc_url,
        })
    }

    pub async fn latest_block(&self) -> Result<Option<Block<H256>>, MonError> {
        Ok(self
            .client
            .eth()
            .block(self.client.eth().block_number().await?.into())
            .await?)
    }

    pub async fn latest_block_id(&self) -> Result<u64, MonError> {
        Ok(self.client.eth().block_number().await?.as_u64())
    }

    pub async fn get_block(&self, block_id: u64) -> Result<Option<Block<Transaction>>, MonError> {
        Ok(self
            .client
            .eth()
            .block_with_txs(BlockId::Number(BlockNumber::Number(block_id.into())))
            .await?)
    }

    pub async fn current_block(&self) -> Result<u64, MonError> {
        Ok(self
            .client
            .eth()
            .block_number()
            .await
            .context(format!(
                "Couldn't get current block number from {}!",
                self.rpc_url
            ))
            .map_err(|e| MonError::RpcClientError(e))?
            .to_string()
            .parse()
            .unwrap_or(0))
    }

    pub async fn syncing_info(&self) -> Result<Option<SyncInfo>, MonError> {
        let res = self
            .client
            .eth()
            .syncing()
            .await
            .context(format!("Couldn't get syncing info for {}!", self.rpc_url))
            .map_err(|e| MonError::RpcClientError(e))?;

        match res {
            web3::types::SyncState::Syncing(sync_info) => Ok(Some(sync_info)),
            web3::types::SyncState::NotSyncing => Ok(None),
        }
    }

    pub async fn get_tx(&self, tx_id: TransactionId) -> Result<Option<Transaction>, MonError> {
        Ok(self.client.eth().transaction(tx_id).await?)
    }

    pub async fn get_balance(&self, address: String) -> Result<f64, MonError> {
        Ok(self
            .client
            .eth()
            .balance(
                H160::from_str(address.as_str())
                    .map_err(|e| MonError::ConversionError(e.into()))?,
                None,
            )
            .await
            .map_err(|e| MonError::RpcClientError(e.into()))?
            .as_u64() as f64)
    }
}
