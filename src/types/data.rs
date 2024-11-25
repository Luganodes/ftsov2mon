#[derive(Debug, Clone)]
pub struct Data {
    pub signing_policy_tx_found: bool,
    pub signing_policy_balance: f64,

    pub submit_tx_found: bool,
    pub submit_balance: f64,

    pub submit_signature_tx_found: bool,
    pub submit_signature_balance: f64,
}
