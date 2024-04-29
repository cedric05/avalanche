use dashmap::DashMap;
use dashmap::DashSet;

/// `User` represents a user in the system.
///
/// It contains fields for the user's information.
#[derive(Hash, PartialEq, Eq)]
pub(crate) struct User {
    pub(crate) id: String,
}

/// `UserStore` is responsible for storing `User` instances.
///
/// It provides methods for adding, removing, and retrieving users.
#[allow(unused)]
#[derive(Default)]
pub(crate) struct UserStore {
    pub(crate) users: DashSet<User>,
}

/// `AuthToken` represents an authentication token.
///
/// It is used for authentication purposes.
#[derive(Hash, PartialEq, Eq)]
pub(crate) struct AuthToken(pub(crate) String);

/// `AuthTokenStore` is a trait that defines the interface for storing authentication tokens.
///
/// It requires the implementation of an `exists` method, which checks if a token exists for a given project.
pub(crate) trait AuthTokenStore: Send + Sync {
    fn exists(&self, token: &AuthToken, project: &str) -> bool;
}

/// `InMemoryAuthTokenStore` is an implementation of the `AuthTokenStore` trait that stores authentication tokens in memory.
#[derive(Default)]
pub(crate) struct InMemoryAuthTokenStore {
    pub(crate) project_token: DashMap<AuthToken, String>,
}

impl InMemoryAuthTokenStore {
    /// Inserts an authentication token and its associated project key into the store.
    pub(crate) fn insert(&self, auth_token: AuthToken, project_key: String) {
        self.project_token.insert(auth_token, project_key);
    }
}

impl AuthTokenStore for InMemoryAuthTokenStore {
    fn exists(&self, token: &AuthToken, project: &str) -> bool {
        let allowed_project = self.project_token.get(token);
        if let Some(allowed_project) = allowed_project {
            *allowed_project == project
        } else {
            false
        }
    }
}

/// `UserTokenStore` is a trait that defines the interface for storing user tokens.
///
/// It requires the implementation of an `exists` method, which checks if a user token exists.
pub(crate) trait UserTokenStore: Send + Sync {
    fn exists(&self, token: &AuthToken) -> bool;
}

/// `SimpleUserTokenStore` is an implementation of the `UserTokenStore` trait that stores user tokens in memory.
#[derive(Default)]
pub(crate) struct SimpleUserTokenStore {
    pub(crate) map: DashMap<AuthToken, User>,
}

impl UserTokenStore for SimpleUserTokenStore {
    fn exists(&self, token: &AuthToken) -> bool {
        self.map.contains_key(token)
    }
}
