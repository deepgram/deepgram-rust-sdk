//! Deepgram billing API response types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// A project billing breakdown over a time range.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BillingBreakdown {
    /// Start date of the requested range.
    pub start: String,
    /// End date of the requested range.
    pub end: String,
    /// Resolution used to aggregate results.
    pub resolution: BreakdownResolution,
    /// Billing totals for each requested grouping.
    pub results: Vec<BillingBreakdownResult>,
}

/// Resolution used in a billing or usage breakdown.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BreakdownResolution {
    /// Time unit used for the resolution.
    pub units: String,
    /// Number of time units per result.
    pub amount: f64,
}

/// One grouped billing total.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BillingBreakdownResult {
    /// Cost in US dollars for this grouping.
    pub dollars: f64,
    /// Dimensions identifying this result.
    pub grouping: BillingGrouping,
}

/// Dimensions identifying a billing breakdown result.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BillingGrouping {
    /// Start date when grouped by time.
    pub start: Option<String>,
    /// End date when grouped by time.
    pub end: Option<String>,
    /// Accessor ID when grouped by accessor.
    pub accessor: Option<String>,
    /// Deployment when grouped by deployment.
    pub deployment: Option<String>,
    /// Billing line item when grouped by line item.
    pub line_item: Option<String>,
    /// Tags when grouped by tags.
    pub tags: Option<Vec<String>>,
}

/// Filter values available for project billing data.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BillingFields {
    /// Accessor IDs used in the requested period.
    pub accessors: Option<Vec<String>>,
    /// Deployments used in the requested period.
    pub deployments: Option<Vec<String>>,
    /// Tags used in the requested period.
    pub tags: Option<Vec<String>>,
    /// Billing line items and their human-readable descriptions.
    pub line_items: Option<HashMap<String, String>>,
}

/// Purchase records for a project.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Purchases {
    /// Zero-based page number returned by the API.
    pub page: Option<u64>,
    /// Maximum number of purchase orders returned per page.
    pub limit: Option<u64>,
    /// Purchase orders returned by the API.
    pub orders: Option<Vec<Purchase>>,
}

/// One project purchase order.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Purchase {
    /// Purchase order ID.
    pub order_id: Option<String>,
    /// Purchase expiration timestamp.
    pub expiration: Option<String>,
    /// Purchase creation timestamp.
    pub created: Option<String>,
    /// Purchased amount.
    pub amount: Option<f64>,
    /// Units for the purchased amount.
    pub units: Option<String>,
    /// Type of purchase order.
    pub order_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{BillingBreakdown, BillingFields, Purchases};

    #[test]
    fn deserializes_billing_breakdown() {
        let response: BillingBreakdown = serde_json::from_str(
            r#"{
                "start": "2026-01-01",
                "end": "2026-01-31",
                "resolution": { "units": "day", "amount": 1 },
                "results": [{
                    "dollars": 0.25,
                    "grouping": { "deployment": "hosted", "tags": ["production"] }
                }]
            }"#,
        )
        .unwrap();

        assert_eq!(response.results[0].dollars, 0.25);
        assert_eq!(
            response.results[0].grouping.deployment.as_deref(),
            Some("hosted")
        );
    }

    #[test]
    fn deserializes_optional_billing_fields_and_purchase_fields() {
        let fields: BillingFields = serde_json::from_str(r#"{}"#).unwrap();
        assert!(fields.accessors.is_none());
        assert!(fields.line_items.is_none());

        let purchases: Purchases =
            serde_json::from_str(r#"{"page":0,"limit":5,"orders":[{}]}"#).unwrap();
        assert_eq!(purchases.page, Some(0));
        assert_eq!(purchases.limit, Some(5));
        assert_eq!(purchases.orders.unwrap().len(), 1);
    }
}
