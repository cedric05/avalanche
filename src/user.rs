use dashmap::DashMap;

use dashmap::DashSet;

#[derive(Hash, PartialEq, Eq)]
pub struct User {
    pub id: String,
}

#[derive(Default)]
pub struct UserStore {
    pub users: DashSet<User>,
}

#[derive(Hash, PartialEq, Eq)]
pub struct AuthToken(pub String);

#[derive(Default)]
pub struct UserTokenStore {
    pub map: DashMap<AuthToken, User>,
}

#[derive(Default)]
pub struct AuthTokensStore {
    pub project_token: DashMap<AuthToken, String>,
}

pub trait AuthTokenStoreT: Send + Sync {
    fn exists(&self, token: &AuthToken, project: &str) -> bool;
}

pub trait UserTokenStoreT: Send + Sync {
    fn exists(&self, token: &AuthToken) -> bool;
}

impl AuthTokenStoreT for AuthTokensStore {
    fn exists(&self, token: &AuthToken, project: &str) -> bool {
        let allowed_project = self.project_token.get(token);
        if let Some(allowed_project) = allowed_project {
            *allowed_project == project
        } else {
            false
        }
    }
}

impl UserTokenStoreT for UserTokenStore {
    fn exists(&self, token: &AuthToken) -> bool {
        self.map.contains_key(token)
    }
}
