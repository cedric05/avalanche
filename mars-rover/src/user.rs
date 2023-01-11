use dashmap::DashMap;

use dashmap::DashSet;

#[derive(Hash, PartialEq, Eq)]
pub(crate) struct User {
    pub(crate) id: String,
}

#[allow(unused)]
#[derive(Default)]
pub(crate) struct UserStore {
    pub(crate) users: DashSet<User>,
}

#[derive(Hash, PartialEq, Eq)]
pub(crate) struct AuthToken(pub(crate) String);

#[derive(Default)]
pub(crate) struct SimpleUserTokenStore {
    pub(crate) map: DashMap<AuthToken, User>,
}

#[derive(Default)]
pub(crate) struct SimpleAuthTokenStore {
    pub(crate) project_token: DashMap<AuthToken, String>,
}

impl SimpleAuthTokenStore {
    pub(crate) fn insert(&self, auth_token: AuthToken, project_key: String) {
        self.project_token.insert(auth_token, project_key);
    }
}

pub(crate) trait AuthTokenStore: Send + Sync {
    fn exists(&self, token: &AuthToken, project: &str) -> bool;
}

pub(crate) trait UserTokenStore: Send + Sync {
    fn exists(&self, token: &AuthToken) -> bool;
}

impl AuthTokenStore for SimpleAuthTokenStore {
    fn exists(&self, token: &AuthToken, project: &str) -> bool {
        let allowed_project = self.project_token.get(token);
        if let Some(allowed_project) = allowed_project {
            *allowed_project == project
        } else {
            false
        }
    }
}

impl UserTokenStore for SimpleUserTokenStore {
    fn exists(&self, token: &AuthToken) -> bool {
        self.map.contains_key(token)
    }
}
