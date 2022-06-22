//! Deepgram billing API response types.

use serde::Deserialize;
use uuid::Uuid;

/// The balances for a Deepgram Project.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#billing
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Balances {
    #[allow(missing_docs)]
    balances: Vec<Balance>,
}

/// Information about a specific balance.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#billing
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Balance {
    #[allow(missing_docs)]
    pub balance_id: Uuid,

    #[allow(missing_docs)]
    pub amount: f64,

    #[allow(missing_docs)]
    pub units: BillingUnits,

    #[allow(missing_docs)]
    pub purchase_order_id: Uuid,
}

/// Units for the [`Balance::amount`] field.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#billing
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Deserialize)]
pub enum BillingUnits {
    #[allow(missing_docs)]
    #[serde(rename = "usd")]
    Usd,

    #[allow(missing_docs)]
    #[serde(rename = "hour")]
    Hour,
}
