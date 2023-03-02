use crate::errors::ApiError;
use actix_web::http::header::Header;
use actix_web::HttpRequest;
use actix_web_httpauth::headers::authorization;
use anyhow::{anyhow, Context};
use jsonwebtoken::jwk::AlgorithmParameters;
use jsonwebtoken::{decode, decode_header, jwk, Algorithm, DecodingKey, Validation};
use lazy_static::lazy_static;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use subscriber::subscriber::details::SubscriberExternalId;
use use_cases::actor::{Actor, ExternalId, Permissions};

lazy_static! {
    static ref JWKS: String = "blackouts-development.eu.auth0.com".to_string();
    static ref AUTHORITIES: Vec<String> = vec!["blackouts-development.eu.auth0.com".to_string()];
    static ref JWKS_SET: jwk::JwkSet = {
        let uri = &format!("https://{}/{}", JWKS.as_str(), ".well-known/jwks.json");
        let res: serde_json::Value = ureq::get(uri)
            .call()
            .unwrap_or_else(|_| panic!("Could not fetch the JWKS from {}", &uri))
            .into_json()
            .expect("Parsed into json");
        serde_json::from_value(res).unwrap()
    };
    static ref AUDIENCES: Vec<String> = vec![
        "https://blackouts.co.ke".to_string(),
        "https://blackouts-development.eu.auth0.com/userinfo".to_string()
    ];
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: Vec<String>,
    exp: usize,
    iat: usize,
    iss: String,
    sub: String,
    permissions: Vec<String>,
}

#[derive(Debug)]
pub struct AuthenticatedUserInfo {
    permissions: Vec<String>,
    external_id: SubscriberExternalId,
}

impl TryFrom<&HttpRequest> for AuthenticatedUserInfo {
    type Error = ApiError;

    fn try_from(req: &HttpRequest) -> Result<Self, Self::Error> {
        let token = authorization::Authorization::<authorization::Bearer>::parse(req)
            .context("Failed to extract bearer token")
            .map_err(|err| ApiError::Unauthorized(format!("{err:?}")))?;

        let token = token.as_ref().token().to_string();
        let header = decode_header(&token)
            .context("Failed to decode header")
            .map_err(|err| ApiError::Unauthorized(format!("{err:?}")))?;

        let kid = header.kid.ok_or_else(|| {
            ApiError::Unauthorized(format!("Token does not have a `kid` header field"))
        })?;

        let jwk = JWKS_SET.find(&kid).ok_or_else(|| {
            ApiError::Unauthorized(format!("No matching JWK found for the given kid"))
        })?;

        let decoding_key = match &jwk.algorithm {
            AlgorithmParameters::RSA(rsa) => {
                let decoding_key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                    .context("Failed to get DecodingKey from rsa_components")
                    .map_err(|err| ApiError::Unauthorized(format!("{err:?}")))?;
                decoding_key
            }
            _ => return Err(ApiError::Unauthorized(format!("Algorithm should be RSA"))),
        };

        let validation = {
            let mut validation = Validation::new(Algorithm::RS256);
            validation.set_issuer(
                &AUTHORITIES
                    .iter()
                    .map(|a| format!("https://{a}/"))
                    .collect::<Vec<_>>(),
            );
            validation.set_audience(&AUDIENCES);
            validation
        };

        let validated_token = decode::<Claims>(&token, &decoding_key, &validation)
            .context("Failed to decode token")
            .map_err(|err| ApiError::Unauthorized(format!("{err:?}")))?;

        let claims = validated_token.claims;

        let external_id = claims
            .sub
            .clone()
            .try_into()
            .map_err(ApiError::Unauthorized)?;

        Ok(AuthenticatedUserInfo {
            permissions: claims.permissions,
            external_id,
        })
    }
}

impl Actor for AuthenticatedUserInfo {
    fn permissions(&self) -> Permissions {
        let permissions = &self.permissions[..];
        permissions.into()
    }

    fn external_id(&self) -> SubscriberExternalId {
        self.external_id.clone()
    }
}
