use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    async_trait,
};
use axum::extract::FromRef;
use crate::user;
use crate::user::db::User;
use crate::devices;

/*
Токен и device теперь читаются только из headers.
Body-фолбэк для pre-Node клиентов сознательно не портируется:
поддержка версий, слевших токен в body, объявлена deprecated.
*/

fn header_value<'a>(parts: &'a Parts, name: &str) -> Option<&'a str> {
    parts.headers.get(name)?.to_str().ok().filter(|s| !s.is_empty())
}

// --- RequireToken ---

pub struct RequireToken(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for RequireToken
where
    S: Send + Sync,
    crate::AppState: axum::extract::FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = header_value(parts, "User-Token")
            .ok_or((StatusCode::UNAUTHORIZED, "Access denied"))?
            .to_string();

        let app_state = crate::AppState::from_ref(state);

        let found = user::db::fetch_user_by_token(&app_state.db, &token)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Access denied"))?;

        match found {
            Some(u) => Ok(RequireToken(u)),
            None => Err((StatusCode::UNAUTHORIZED, "Access denied")),
        }
    }
}

// --- RequireDevice ---

pub struct RequireDevice(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for RequireDevice
where
    S: Send + Sync,
    crate::AppState: axum::extract::FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = header_value(parts, "User-Token")
            .ok_or((StatusCode::UNAUTHORIZED, "Access denied"))?
            .to_string();
        let device_token = header_value(parts, "Device-Static")
            .ok_or((StatusCode::UNAUTHORIZED, "Access denied"))?
            .to_string();

        let app_state = crate::AppState::from_ref(state);

        let found = user::db::fetch_user_by_token(&app_state.db, &token)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Access denied"))?
            .ok_or((StatusCode::UNAUTHORIZED, "Access denied"))?;

        let device_ok = devices::easy_check_device(&app_state.db, found.id, &device_token)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Access denied"))?;

        if !device_ok {
            return Err((StatusCode::UNAUTHORIZED, "Access denied"));
        }

        if found.activated == 0 {
            return Err((StatusCode::FORBIDDEN, "Access denied"));
        }

        Ok(RequireDevice(found))
    }
}

// --- RequireDeviceForInactiveAccs ---

pub struct RequireDeviceForInactiveAccs(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for RequireDeviceForInactiveAccs
where
    S: Send + Sync,
    crate::AppState: axum::extract::FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = header_value(parts, "User-Token")
            .ok_or((StatusCode::UNAUTHORIZED, "Access denied"))?
            .to_string();
        let device_token = header_value(parts, "Device-Static")
            .ok_or((StatusCode::UNAUTHORIZED, "Access denied"))?
            .to_string();

        let app_state = crate::AppState::from_ref(state);

        let found = user::db::fetch_user_by_token(&app_state.db, &token)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Access denied"))?
            .ok_or((StatusCode::UNAUTHORIZED, "Access denied"))?;

        let device_ok = devices::easy_check_device(&app_state.db, found.id, &device_token)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Access denied"))?;

        if !device_ok {
            return Err((StatusCode::UNAUTHORIZED, "Access denied"));
        }

        Ok(RequireDeviceForInactiveAccs(found))
    }
}
