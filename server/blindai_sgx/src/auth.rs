use std::{fs, lazy::SyncOnceCell};

use anyhow::{Context, Result};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use log::*;
use serde::{Deserialize, Serialize};
use tonic::{Request, Status};

static JWT_POLICY: SyncOnceCell<(Validation, DecodingKey)> = SyncOnceCell::new();

pub fn setup() -> Result<()> {
    let key = if let Ok(key) = fs::read("./jwt_key.pem") {
        info!("Using JWT validation.");
        key
    } else {
        info!("NOT using JWT validation, since file `jwt_key.pem` does not exist.");
        return Ok(());
    };

    let key = DecodingKey::from_ec_pem(&key)
        .context("Parsing JWT validation ES256 `jwt_key.pem` file.")?;

    // See https://docs.rs/jsonwebtoken/8.1.1/jsonwebtoken/struct.Validation.html for more info about JWT validation policy
    let mut validation_policy = Validation::new(Algorithm::ES256);
    validation_policy.validate_exp = true; // validation "exp" (expiry time) field
    let _ = JWT_POLICY.set((validation_policy, key));

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub username: String,
    pub userid: usize,

    // Expiration time (as UTC timestamp)
    pub exp: usize,
}

#[derive(Debug, Default, Clone)]
pub struct AuthExtension {
    pub claims: Option<JwtClaims>,
}

impl AuthExtension {
    #[allow(unused)]
    pub fn is_logged(&self) -> bool {
        self.claims.is_some()
    }

    #[allow(unused)]
    pub fn require_logged(&self) -> Result<&JwtClaims, Status> {
        self.claims
            .as_ref()
            .ok_or_else(|| Status::unauthenticated("You are not authenticated"))
    }

    pub fn userid(&self) -> Option<usize> {
        self.claims.as_ref().map(|c| c.userid)
    }

    pub fn username(&self) -> Option<&str> {
        self.claims.as_ref().map(|c| c.username.as_ref())
    }
}

/// This tonic interceptor will extend the request with `AuthExtension` as an
/// extension.
pub fn auth_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    let (policy, key) = if let Some(p) = JWT_POLICY.get() {
        p
    } else {
        // JWT verification is disabled
        return Ok(req);
    };

    let t = if let Some(t) = req.metadata().get("accesstoken") {
        t
    } else {
        req.extensions_mut().insert(AuthExtension::default());
        return Ok(req);
    };

    let t = t
        .to_str()
        .map_err(|_| Status::invalid_argument("Invalid AccessToken header"))?;

    let token = jsonwebtoken::decode::<JwtClaims>(t, key, policy)
        .map_err(|_| Status::invalid_argument("Invalid AccessToken header"))?;

    req.extensions_mut().insert(AuthExtension {
        claims: Some(token.claims),
    });

    Ok(req)
}
