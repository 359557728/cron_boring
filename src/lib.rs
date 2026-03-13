use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct GenericResponse {
    pub code: i32,
    pub data: Option<serde_json::Value>,
}

#[event(scheduled)]
pub async fn scheduled(evt: ScheduledEvent, env: Env, _: ScheduleContext) {
    let cron = evt.cron();
    match cron.as_str() {
        "30 2 * * sun" => {
            log_erase(env).await;
        }
        "0/5 0-15 * * *" => {
            zombie_task_stop(env).await;
        }
        "0/15 0-15 * * *" => {
            pending_task_notify(env).await;
        }
        _ => {}
    }
}

pub async fn log_erase(env: Env) {
    let kv = env.kv("EMAR_BORING").unwrap();
    let api = kv.get("dp_log_erase").text().await.unwrap().unwrap();
    emar_get_api(&api).await;
}

pub async fn zombie_task_stop(env: Env) {
    let kv = env.kv("EMAR_BORING").unwrap();
    let api = kv.get("dp_zombie_task_stop").text().await.unwrap().unwrap();
    emar_get_api(&api).await;
}

pub async fn pending_task_notify(env: Env) {
    let kv = env.kv("EMAR_BORING").unwrap();
    let api = kv
        .get("dp_pending_task_notify")
        .text()
        .await
        .unwrap()
        .unwrap();
    emar_get_api(&api).await;
}

async fn emar_get_api(api: &str) {
    let rt = reqwest::get(api).await;
    if let Ok(res) = rt {
        if res.status() == reqwest::StatusCode::OK {
            let rt = res.json::<GenericResponse>().await;
            if let Ok(GenericResponse { code: 0, .. }) = rt {
                console_log!("{} successful", api);
            }
        }
    }
}
