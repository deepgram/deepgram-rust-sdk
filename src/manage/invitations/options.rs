//! Request options for [`Invitations::create`](super::Invitations::create).

use serde::Serialize;

/// Details for a project invitation.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Options {
    email: String,
    scope: String,
}

/// Builds [`Options`] for a project invitation.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OptionsBuilder(Options);

#[derive(Serialize)]
pub(crate) struct SerializableOptions<'a> {
    email: &'a str,
    scope: &'a str,
}

impl Options {
    /// Start building an invitation for `email` with the supplied project scope.
    pub fn builder(email: impl Into<String>, scope: impl Into<String>) -> OptionsBuilder {
        OptionsBuilder::new(email, scope)
    }

    /// Serialize this invitation request as JSON.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&SerializableOptions::from(self))
    }
}

impl OptionsBuilder {
    /// Start building an invitation for `email` with the supplied project scope.
    pub fn new(email: impl Into<String>, scope: impl Into<String>) -> Self {
        Self(Options {
            email: email.into(),
            scope: scope.into(),
        })
    }

    /// Replace the invited email address.
    pub fn email(mut self, value: impl Into<String>) -> Self {
        self.0.email = value.into();
        self
    }

    /// Replace the project scope granted by the invitation.
    pub fn scope(mut self, value: impl Into<String>) -> Self {
        self.0.scope = value.into();
        self
    }

    /// Finish building the invitation request.
    pub fn build(self) -> Options {
        self.0
    }
}

impl<'a> From<&'a Options> for SerializableOptions<'a> {
    fn from(options: &'a Options) -> Self {
        Self {
            email: &options.email,
            scope: &options.scope,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Options;

    #[test]
    fn serializes_required_fields() {
        let options = Options::builder("developer@example.com", "member").build();

        assert_eq!(
            options.json().unwrap(),
            r#"{"email":"developer@example.com","scope":"member"}"#
        );
    }
}
