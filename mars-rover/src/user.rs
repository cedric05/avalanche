/// `AuthToken` represents an authentication token.
///
/// It is used for authentication purposes.
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AuthToken(pub(crate) String);