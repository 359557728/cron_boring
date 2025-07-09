use serde::{Deserialize, Serialize};
use worker::{self, *};
use worker::kv::KvError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Worker error: {0}")]
    Worker(#[from] worker::Error),
    #[error("KV error: {0}")]
    KvError(#[from] KvError),
    #[error("KV key not found: {0}")]
    KvKeyNotFound(String),
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("API response error: {0} - {1}")]
    ApiResponse(u16, String),
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenericResponse {
    pub code: i32,
    pub data: Option<serde_json::Value>,
}

impl From<AppError> for worker::Error {
    fn from(err: AppError) -> Self {
        worker::Error::RustError(format!("Application error: {:?}", err))
    }
}

#[event(scheduled)]
pub async fn scheduled(evt: ScheduledEvent, env: Env, _: ScheduleContext) {
    let cron = evt.cron();
    match cron.as_str() {
        "30 2 * * sun" => {
            if let Err(e) = log_erase(&env).await {
                console_error!("Error in log_erase: {:?}", e);
            }
        }
        "0/5 0-15 * * *" => {
            if let Err(e) = zombie_task_stop(&env).await {
                console_error!("Error in zombie_task_stop: {:?}", e);
            }
        }
        "0/15 0-15 * * *" => {
            if let Err(e) = pending_task_notify(&env).await {
                console_error!("Error in pending_task_notify: {:?}", e);
            }
        }
        _ => console_log!("Unknown cron schedule: {}", cron),
    }
}

async fn get_kv_value(env: &Env, key: &str) -> std::result::Result<String, AppError> {
    let kv = env.kv("EMAR_BORING")?;
    kv.get(key)
        .text()
        .await?
        .ok_or_else(|| AppError::KvKeyNotFound(format!("KV key '{}' not found or empty", key)))
}

pub async fn log_erase(env: &Env) -> std::result::Result<(), AppError> {
    let api = get_kv_value(env, "dp_log_erase").await?;
    emar_get_api(&api).await
}

pub async fn zombie_task_stop(env: &Env) -> std::result::Result<(), AppError> {
    let api = get_kv_value(env, "dp_zombie_task_stop").await?;
    emar_get_api(&api).await
}

pub async fn pending_task_notify(env: &Env) -> std::result::Result<(), AppError> {
    let api = get_kv_value(env, "dp_pending_task_notify").await?;
    emar_get_api(&api).await
}

async fn emar_get_api(api: &str) -> std::result::Result<(), AppError> {
    let res = reqwest::get(api).await?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_else(|_| "N/A".to_string());
        console_error!("API call to {} failed with status: {} - Response: {}", api, status, text);
        return Err(AppError::ApiResponse(
            status.as_u16(),
            format!("API call to {} failed with status: {}", api, status),
        ));
    }

    let response_body = res.json::<GenericResponse>().await?;

    if response_body.code == 0 {
        console_log!("{} successful", api);
        Ok(())
    } else {
        console_error!(
            "API call to {} returned non-zero code: {}",
            api,
            response_body.code
        );
        Err(AppError::ApiResponse(
            response_body.code as u16,
            format!("API call to {} returned non-zero code: {}", api, response_body.code),
        ))
    }
}
