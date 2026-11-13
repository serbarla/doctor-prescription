use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::{header, HeaderMap};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::api_server::require_actor;
use crate::auth_server::{self, AppState, AuthError};
use crate::db;
use crate::service;

#[derive(Debug, Deserialize)]
pub struct SyncQuery {
    pub since: Option<i64>,
}

fn sync_event_json(event: &db::SyncEvent) -> serde_json::Value {
    json!({
        "cursor": event.cursor,
        "action": event.action,
        "entity": event.entity,
        "entityId": event.entity_id,
        "at": event.at,
    })
}

pub async fn sync_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SyncQuery>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let rx_status = service::rx_block_status(&state.pool, &actor).await?;
    let since = query.since.unwrap_or(0);
    let (events, cursor) = auth_server::with_conn(&state.pool, move |connection| {
        let events = db::list_sync_events(connection, &clinic_id, since)?;
        let cursor = db::sync_cursor(connection, &clinic_id)?;
        Ok::<_, AuthError>((events, cursor))
    })
    .await?;
    Ok(Json(json!({
        "serverTime": auth_server::now(),
        "cursor": cursor,
        "version": cursor,
        "events": events.iter().map(sync_event_json).collect::<Vec<_>>(),
        "rxBlock": {
            "active": rx_status.active,
            "blockId": rx_status.block_id,
            "startNumber": rx_status.start_number,
            "endNumber": rx_status.end_number,
            "consumed": rx_status.consumed,
            "remaining": rx_status.remaining,
            "limit": rx_status.limit,
            "exhausted": rx_status.exhausted,
            "deviceId": rx_status.device_id,
        },
    }))
    .into_response())
}

pub async fn sync_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SyncQuery>,
) -> Result<Response, AuthError> {
    let actor = require_actor(&state, &headers).await?;
    let clinic_id = actor.clinic_id()?.to_owned();
    let pool = state.pool.clone();
    let since = query.since.unwrap_or(0);
    let (sender, receiver) = mpsc::channel::<Result<String, std::io::Error>>(16);
    tokio::spawn(async move {
        let mut cursor = since;
        if cursor == 0 {
            let clinic_id = clinic_id.clone();
            cursor = auth_server::with_conn(&pool, move |connection| {
                let cursor = db::sync_cursor(connection, &clinic_id)?;
                Ok::<_, AuthError>(cursor)
            })
            .await
            .unwrap_or(0);
        }
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let clinic_id = clinic_id.clone();
            let since_cursor = cursor;
            let next = auth_server::with_conn(&pool, move |connection| {
                let events = db::list_sync_events(connection, &clinic_id, since_cursor)?;
                Ok::<_, AuthError>(events)
            })
            .await;
            if let Ok(events) = next {
                for event in &events {
                    let mut payload = String::from("event: change\ndata: ");
                    payload.push_str(&sync_event_json(event).to_string());
                    payload.push_str("\n\n");
                    cursor = event.cursor;
                    if sender.send(Ok(payload)).await.is_err() {
                        return;
                    }
                }
            }
            if sender.send(Ok(": keepalive\n\n".to_owned())).await.is_err() {
                return;
            }
        }
    });
    let stream = ReceiverStream::new(receiver);
    Response::builder()
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(axum::body::Body::from_stream(stream))
        .map_err(|_| AuthError::Internal)
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/sync/state", get(sync_state))
        .route("/api/sync/events", get(sync_events))
        .with_state(state)
}
