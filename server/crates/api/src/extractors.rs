//! Request extractors — most importantly `Auth`, which resolves the caller
//! from any of the credential carriers Immich supports:
//!
//!   1. `x-api-key` header                        → API key auth
//!   2. `Authorization: Bearer <token>`           → session auth
//!   3. cookie `immich_access_token`              → session auth (web)
//!   4. `x-immich-session-token` header           → session auth (legacy)
//!   5. query `?key=` / `?slug=`                  → shared-link auth

use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use domus_common::Error;
use domus_domain::services::auth::AuthContext;

pub const COOKIE_ACCESS_TOKEN: &str = "immich_access_token";
pub const HEADER_API_KEY: &str = "x-api-key";
pub const HEADER_SESSION_TOKEN: &str = "x-immich-session-token";

pub struct Auth(pub AuthContext);

impl FromRequestParts<AppState> for Auth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = &state.services.auth;

        if let Some(key) = header(parts, HEADER_API_KEY) {
            return Ok(Auth(auth.validate_api_key(&key).await?));
        }
        if let Some(token) = bearer(parts)
            .or_else(|| cookie(parts, COOKIE_ACCESS_TOKEN))
            .or_else(|| header(parts, HEADER_SESSION_TOKEN))
        {
            return Ok(Auth(auth.validate_session(&token).await?));
        }
        let (key, slug) = shared_link_query(parts);
        if key.is_some() || slug.is_some() {
            return Ok(Auth(
                auth.validate_shared_link(key.as_deref(), slug.as_deref())
                    .await?,
            ));
        }
        Err(ApiError(Error::Unauthorized(
            "Authentication required".into(),
        )))
    }
}

/// Same as `Auth` but additionally requires the admin flag.
pub struct AdminAuth(pub AuthContext);

impl FromRequestParts<AppState> for AdminAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Auth(ctx) = Auth::from_request_parts(parts, state).await?;
        if !ctx.is_admin {
            return Err(ApiError(Error::Forbidden("Forbidden".into())));
        }
        Ok(AdminAuth(ctx))
    }
}

fn header(parts: &Parts, name: &str) -> Option<String> {
    parts.headers.get(name)?.to_str().ok().map(str::to_owned)
}

fn bearer(parts: &Parts) -> Option<String> {
    let value = parts.headers.get("authorization")?.to_str().ok()?;
    value.strip_prefix("Bearer ").map(str::to_owned)
}

fn cookie(parts: &Parts, name: &str) -> Option<String> {
    let cookies = parts.headers.get("cookie")?.to_str().ok()?;
    cookies.split(';').find_map(|c| {
        let (k, v) = c.trim().split_once('=')?;
        (k == name).then(|| v.to_owned())
    })
}

fn shared_link_query(parts: &Parts) -> (Option<String>, Option<String>) {
    let mut key = None;
    let mut slug = None;
    if let Some(query) = parts.uri.query() {
        for pair in query.split('&') {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            match name {
                "key" => key = Some(value.to_owned()),
                "slug" => slug = Some(value.to_owned()),
                _ => {}
            }
        }
    }
    (key, slug)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    fn parts(request: Request<()>) -> Parts {
        request.into_parts().0
    }

    #[test]
    fn bearer_reads_authorization_header() {
        let parts = parts(
            Request::builder()
                .header("authorization", "Bearer session-token")
                .body(())
                .unwrap(),
        );
        assert_eq!(bearer(&parts), Some("session-token".to_owned()));
    }

    #[test]
    fn cookie_reads_immich_access_token() {
        let parts = parts(
            Request::builder()
                .header(
                    "cookie",
                    "other=1; immich_access_token=session-token; theme=dark",
                )
                .body(())
                .unwrap(),
        );
        assert_eq!(
            cookie(&parts, COOKIE_ACCESS_TOKEN),
            Some("session-token".to_owned())
        );
    }

    #[test]
    fn header_reads_api_key_and_legacy_session_token() {
        let parts = parts(
            Request::builder()
                .header(HEADER_API_KEY, "api-secret")
                .header(HEADER_SESSION_TOKEN, "legacy-session")
                .body(())
                .unwrap(),
        );
        assert_eq!(
            header(&parts, HEADER_API_KEY),
            Some("api-secret".to_owned())
        );
        assert_eq!(
            header(&parts, HEADER_SESSION_TOKEN),
            Some("legacy-session".to_owned())
        );
    }

    #[test]
    fn shared_link_query_reads_key_and_slug() {
        let parts = parts(
            Request::builder()
                .uri("/api/assets/1/original?key=abc123&slug=summer")
                .body(())
                .unwrap(),
        );
        assert_eq!(
            shared_link_query(&parts),
            (Some("abc123".to_owned()), Some("summer".to_owned()))
        );
    }
}
