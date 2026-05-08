//! Get the outstanding balances for a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#billing

use crate::{
    manage::billing::response::{
        Balance, Balances, BillingBreakdown, BillingFields, PurchaseOrders,
    },
    send_and_translate_response, Deepgram,
};

pub mod breakdown_options;
pub mod response;

/// Get the outstanding balances for a Deepgram Project.
///
/// Constructed using [`Deepgram::billing`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#billing
#[derive(Debug, Clone)]
pub struct Billing<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`Billing`] from a [`Deepgram`].
    pub fn billing(&self) -> Billing<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Billing<'a> {
    /// Construct a new [`Billing`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl Billing<'_> {
    /// Get the outstanding balances for the specified project.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#billing-all
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use deepgram::{Deepgram, DeepgramError};
    /// # use std::env;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// # let project_id =
    /// #     env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// let balances = dg_client
    ///     .billing()
    ///     .list_balance(&project_id)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_balance(&self, project_id: &str) -> crate::Result<Balances> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/balances",);

        send_and_translate_response(self.0.client.get(url)).await
    }

    /// Get the details of a specific balance.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#billing-get
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use deepgram::{Deepgram, DeepgramError};
    /// # use std::env;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// # let project_id =
    /// #     env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");
    /// #
    /// # let balance_id =
    /// #     env::var("DEEPGRAM_BALANCE_ID").expect("DEEPGRAM_BALANCE_ID environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// let balance = dg_client
    ///     .billing()
    ///     .get_balance(&project_id, &balance_id)
    ///     .await?;
    ///
    /// assert_eq!(balance_id, balance.balance_id.to_string());
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_balance(&self, project_id: &str, balance_id: &str) -> crate::Result<Balance> {
        let url =
            format!("https://api.deepgram.com/v1/projects/{project_id}/balances/{balance_id}",);

        send_and_translate_response(self.0.client.get(url)).await
    }

    /// `GET /v1/projects/{project_id}/billing/breakdown` — billing
    /// summary for the project, with optional filters and grouping.
    pub async fn breakdown(
        &self,
        project_id: &str,
        options: &breakdown_options::Options,
    ) -> crate::Result<BillingBreakdown> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/billing/breakdown");
        let request = self
            .0
            .client
            .get(url)
            .query(&breakdown_options::SerializableOptions(options));
        send_and_translate_response(request).await
    }

    /// `GET /v1/projects/{project_id}/billing/fields` — list the
    /// dimensions (accessors, deployments, tags, line items) available
    /// for filtering [`Billing::breakdown`] queries over the given
    /// date range.
    pub async fn fields(
        &self,
        project_id: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> crate::Result<BillingFields> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/billing/fields");
        let mut request = self.0.client.get(url);
        let mut query: Vec<(&str, &str)> = Vec::new();
        if let Some(start) = start {
            query.push(("start", start));
        }
        if let Some(end) = end {
            query.push(("end", end));
        }
        if !query.is_empty() {
            request = request.query(&query);
        }
        send_and_translate_response(request).await
    }

    /// `GET /v1/projects/{project_id}/purchases` — list purchase
    /// orders for the project. `limit` is forwarded as the
    /// per-page-size query param (1-1000 per spec).
    pub async fn purchases(
        &self,
        project_id: &str,
        limit: Option<usize>,
    ) -> crate::Result<PurchaseOrders> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/purchases");
        let mut request = self.0.client.get(url);
        if let Some(limit) = limit {
            request = request.query(&[("limit", limit)]);
        }
        send_and_translate_response(request).await
    }
}

#[cfg(test)]
mod tests {
    use crate::manage::billing::response::{Balance, BillingUnits};

    #[test]
    fn test() {
        assert_eq!(
            serde_json::from_str::<Balance>(
                "{\"balance_id\":\"a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8\",\"amount\":1,\"units\":\"usd\",\"purchase_order_id\":\"a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8\"}",
            ).unwrap().units,
            BillingUnits::Usd
        );
    }
}
