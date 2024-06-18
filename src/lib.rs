use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct GenericResponse<'a> {
    pub code: i32,
    pub message: Option<&'a str>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenericResponseSim {
    pub code: i32,
    pub data: Option<serde_json::Value>,
}

pub const API_ACCESS_FAIL: GenericResponse = GenericResponse {
    code: -1,
    message: Some("接口暂不可用"),
    data: None,
};

pub const API_FATAL_ERROR: GenericResponse = GenericResponse {
    code: -1,
    message: Some("接口无法正常响应"),
    data: None,
};

pub const API_RESPONSE_PARSE_ERROR: GenericResponse = GenericResponse {
    code: -1,
    message: Some("接口响应无法正常解析"),
    data: None,
};

#[event(scheduled)]
pub async fn scheduled(evt: ScheduledEvent, env: Env, _: ScheduleContext) {
    let cron = evt.cron();
    match cron.as_str() {
        "30 2 * * sun" => {
            log_erase(env).await;
        }
        "0/5 * * * *" => {
            zombie_task_stop(env).await;
        }
        _ => {}
    }
}

pub async fn log_erase(env: Env) {
    let kv = env.kv("EMAR_BORING").unwrap();
    let api = kv.get("dp_log_erase").text().await.unwrap().unwrap();
    let rt = emar_get_api(&api).await;
    if let GenericResponse { code: 0, .. } = rt {
        console_log!("dolphin log erase success");
    }
}

pub async fn zombie_task_stop(env: Env) {
    let kv = env.kv("EMAR_BORING").unwrap();
    let api = kv.get("dp_zombie_task_stop").text().await.unwrap().unwrap();
    let rt = emar_get_api(&api).await;
    if let GenericResponse { code: 0, .. } = rt {
        console_log!("dolphin zombie task stop success");
    }
}

async fn emar_get_api(api: &str) -> GenericResponse {
    let rt = reqwest::get(api).await;
    if let Ok(res) = rt {
        if res.status() == reqwest::StatusCode::OK {
            let rt = res.json::<GenericResponseSim>().await;
            if let Ok(GenericResponseSim { code: 0, data }) = rt {
                GenericResponse {
                    code: 0,
                    data,
                    message: Some("接口请求成功"),
                }
            } else {
                API_RESPONSE_PARSE_ERROR
            }
        } else {
            API_FATAL_ERROR
        }
    } else {
        API_ACCESS_FAIL
    }
}
