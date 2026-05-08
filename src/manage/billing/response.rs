//! Deepgram billing API response types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The balances for a Deepgram Project.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#billing
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Balances {
    #[allow(missing_docs)]
    pub balances: Vec<Balance>,
}

/// Information about a specific balance.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#billing
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
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
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BillingUnits {
    #[allow(missing_docs)]
    #[serde(rename = "usd")]
    Usd,

    #[allow(missing_docs)]
    #[serde(rename = "hour")]
    Hour,
}

/// Returned by [`Billing::breakdown`](super::Billing::breakdown).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BillingBreakdown {
    /// Start of the billing period.
    pub start: String,
    /// End of the billing period.
    pub end: String,
    /// Resolution of each row in `results`.
    pub resolution: Resolution,
    /// One row per grouping bucket.
    pub results: Vec<BillingBreakdownResult>,
}

/// Time resolution shared by [`BillingBreakdown`] and `UsageBreakdown`.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Resolution {
    /// Time unit for the resolution (e.g. `day`).
    pub units: String,
    /// Amount of units (e.g. `1`).
    pub amount: f64,
}

/// One row of a [`BillingBreakdown`].
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BillingBreakdownResult {
    /// USD cost for this grouping bucket.
    pub dollars: f64,
    /// Grouping dimensions that produced this row.
    pub grouping: BillingBreakdownGrouping,
}

/// Grouping metadata on a [`BillingBreakdownResult`].
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BillingBreakdownGrouping {
    #[allow(missing_docs)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,

    #[allow(missing_docs)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,

    #[allow(missing_docs)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessor: Option<String>,

    #[allow(missing_docs)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment: Option<String>,

    #[allow(missing_docs)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_item: Option<String>,

    #[allow(missing_docs)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Returned by [`Billing::fields`](super::Billing::fields). Lists the
/// dimensions available for filtering [`BillingBreakdown`] queries.
#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BillingFields {
    /// Accessor UUIDs that have produced billing in the time range.
    #[serde(default)]
    pub accessors: Vec<String>,

    /// Deployment types that have produced billing.
    #[serde(default)]
    pub deployments: Vec<String>,

    /// Tags that have produced billing.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Line item identifiers mapped to human-readable descriptions.
    /// e.g. `"streaming::nova-3" -> "Nova-3 (Stream)"`.
    #[serde(default)]
    pub line_items: HashMap<String, String>,
}

/// Returned by [`Billing::purchases`](super::Billing::purchases).
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PurchaseOrders {
    /// Purchase orders.
    #[serde(default)]
    pub orders: Vec<PurchaseOrder>,
}

/// A single purchase order.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PurchaseOrder {
    /// Order UUID.
    pub order_id: Uuid,
    /// Expiration timestamp (ISO 8601).
    pub expiration: String,
    /// Creation timestamp (ISO 8601).
    pub created: String,
    /// Amount of the purchase.
    pub amount: f64,
    /// Units of the amount (e.g. `usd`).
    pub units: String,
    /// Type of order (e.g. `promotional`).
    pub order_type: String,
}
