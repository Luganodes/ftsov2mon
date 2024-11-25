use crate::rpc::RpcClient;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub tg_api_key: String,
    pub tg_chat_id: String,
    pub rpc_client: RpcClient,
    pub block_window: u16,
    pub submit_address: String,
    pub submit_signature_address: String,
    pub signing_policy_address: String,
}
