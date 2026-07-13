//! Integration tests for `store::redis::RedisTimeStore` against a real
//! Redis 8 instance (Time Series built in), spun up via `testcontainers`.
//! Requires Docker on the host running these tests.

use fermenter::model::{ControllerState, Reading};
use fermenter::store::TimeStore;
use fermenter::store::redis::RedisTimeStore;
use testcontainers::ImageExt;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::redis::{REDIS_PORT, Redis};

fn reading(target: f64, average: f64) -> Reading {
    Reading {
        target,
        average,
        min: 18.0,
        max: 19.0,
        ambient: 20.0,
        action: "idle".to_string(),
        reason_code: String::new(),
        json_size: None,
    }
}

async fn start_store() -> (RedisTimeStore, testcontainers::ContainerAsync<Redis>) {
    let container = Redis::default()
        .with_tag("8")
        .start()
        .await
        .expect("redis container should start");
    let host = container.get_host().await.expect("container host");
    let port = container
        .get_host_port_ipv4(REDIS_PORT)
        .await
        .expect("container port");
    let url = format!("redis://{host}:{port}");

    let store = RedisTimeStore::connect(&url, 7).expect("should build the store");

    (store, container)
}

#[tokio::test]
async fn write_then_last_reading_roundtrip() {
    let (store, _container) = start_store().await;

    store
        .write_reading("brew-1", &reading(19.5, 18.2))
        .await
        .expect("write should succeed");

    let last = store
        .last_reading("brew-1")
        .await
        .expect("query should succeed")
        .expect("should have a reading");
    assert_eq!(last.average, 18.2);
    assert_eq!(last.target, 19.5);
    assert_eq!(last.min, 18.0);
    assert_eq!(last.max, 19.0);
    assert_eq!(last.action, "idle");
}

#[tokio::test]
async fn last_reading_on_brew_with_no_data_returns_none() {
    let (store, _container) = start_store().await;

    let last = store
        .last_reading("no-such-brew")
        .await
        .expect("query should succeed");
    assert!(last.is_none());
}

#[tokio::test]
async fn repeated_writes_are_idempotent_on_series_creation() {
    let (store, _container) = start_store().await;

    // Writing the same brew twice must not error on the second TS.CREATE
    // attempt (series already exists).
    store
        .write_reading("brew-1", &reading(19.5, 18.0))
        .await
        .expect("first write should succeed");
    store
        .write_reading("brew-1", &reading(19.5, 18.5))
        .await
        .expect("second write should succeed despite existing series");

    let last = store
        .last_reading("brew-1")
        .await
        .expect("query should succeed")
        .expect("should have a reading");
    assert_eq!(last.average, 18.5);
}

#[tokio::test]
async fn configured_retention_period_is_applied_at_creation() {
    let (store, container) = start_store().await;

    store
        .write_reading("brew-1", &reading(19.5, 18.0))
        .await
        .expect("write should succeed");

    // Inspect the series directly via TS.INFO to confirm the retention
    // period passed to `RedisTimeStore::connect` (7 days) was applied.
    let host = container.get_host().await.expect("container host");
    let port = container
        .get_host_port_ipv4(REDIS_PORT)
        .await
        .expect("container port");
    let client = redis::Client::open(format!("redis://{host}:{port}")).unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let info: Vec<redis::Value> = redis::cmd("TS.INFO")
        .arg("temp:brew-1:fermenter")
        .query_async(&mut conn)
        .await
        .expect("TS.INFO should succeed");

    let seven_days_ms = 7i64 * 24 * 60 * 60 * 1000;
    let retention = find_info_value(&info, "retentionTime").expect("retentionTime present");
    assert_eq!(retention, seven_days_ms);
}

/// `TS.INFO` returns a flat alternating key/value array with keys as simple
/// strings; find the integer value following a given field name.
fn find_info_value(info: &[redis::Value], field: &str) -> Option<i64> {
    info.iter()
        .position(|v| matches!(v, redis::Value::SimpleString(s) if s == field))
        .and_then(|idx| info.get(idx + 1))
        .and_then(|v| match v {
            redis::Value::Int(i) => Some(*i),
            _ => None,
        })
}

#[tokio::test]
async fn save_then_load_controller_state_roundtrip() {
    let (store, _container) = start_store().await;

    let state = ControllerState {
        target_temp: 19.5,
        brew_id: "brew-1".to_string(),
    };
    store.save_state(&state).await.expect("save should succeed");

    let loaded = store
        .load_state()
        .await
        .expect("load should succeed")
        .expect("should have saved state");
    assert_eq!(loaded, state);
}

#[tokio::test]
async fn load_state_on_empty_database_returns_none() {
    let (store, _container) = start_store().await;

    let loaded = store
        .load_state()
        .await
        .expect("load should succeed on empty db");
    assert!(loaded.is_none());
}
