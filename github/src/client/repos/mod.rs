use crate::client::Client;

mod collaborators;

pub use collaborators::ListCollaboratorsOptions;

/// `RepositoryClient` handles communication with the Repository related methods of the GitHub API.
///
/// GitHub API docs: https://developer.github.com/v3/repos/
pub struct RepositoryClient<'a> {
    inner: &'a Client,
}

impl<'a> RepositoryClient<'a> {
    pub(super) fn new(client: &'a Client) -> Self {
        Self { inner: client }
    }

    // TODO: fill in endpoints from:
    // https://developer.github.com/v3/repos/
}
