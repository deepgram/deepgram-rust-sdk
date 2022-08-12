//! Get the outstanding balances for a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#billing

use crate::{send_and_translate_response, Deepgram};

pub mod response;

use response::{Balance, Balances};

/// Get the outstanding balances for a Deepgram Project.
///
/// Constructed using [`Deepgram::billing`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#billing
#[derive(Debug, Clone)]
pub struct Billing<'a>(&'a Deepgram);

impl<'a> Deepgram {
    /// Construct a new [`Billing`] from a [`Deepgram`].
    pub fn billing(&'a self) -> Billing<'a> {
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
        let url = format!(
            "https://api.deepgram.com/v1/projects/{}/balances",
            project_id,
        );

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
        let url = format!(
            "https://api.deepgram.com/v1/projects/{}/balances/{}",
            project_id, balance_id,
        );

        send_and_translate_response(self.0.client.get(url)).await
    }
}

#[cfg(test)]
mod tests {
    use super::{response::BillingUnits, *};

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
