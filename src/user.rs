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
pub struct SimpleUserTokenStore {
    pub map: DashMap<AuthToken, User>,
}

#[derive(Default)]
pub struct SimpleAuthTokenStore {
    pub project_token: DashMap<AuthToken, String>,
}

impl SimpleAuthTokenStore {
    pub fn insert(&self, auth_token: AuthToken, project_key: String) {
        self.project_token.insert(auth_token, project_key);
    }
}

pub trait AuthTokenStore: Send + Sync {
    fn exists(&self, token: &AuthToken, project: &str) -> bool;
}

pub trait UserTokenStore: Send + Sync {
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
