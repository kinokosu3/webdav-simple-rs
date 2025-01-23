use crate::{config::Config, error::WebDavError};
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use serde_json::Value;

const API_URL: &str = "https://drive-pc.quark.cn/1/clouddrive";

pub async fn base_request<T, F>(
    path: &str,
    method: reqwest::Method,
    callback: F,
) -> Result<T, WebDavError>
where
    T: DeserializeOwned,
    F: FnOnce(reqwest::RequestBuilder) -> reqwest::RequestBuilder,
{
    let url = format!("{}{}", API_URL, path);
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    // 判断是否为空
    let config = Config::get();
    if config.storage.quark.cookie.is_empty() {
        return Err(WebDavError::InvalidInput("Cookie 为空".into()));
    }

    let cookie = &config.storage.quark.cookie;
    headers.insert(
        "Cookie",
        cookie
            .parse()
            .map_err(|e| WebDavError::InvalidInput(format!("无效的 Cookie: {}", e)))?,
    );

    headers.insert(
        "Accept",
        "application/json, text/plain, */*".parse().unwrap(),
    );
    headers.insert("Referer", "https://pan.quark.cn/".parse().unwrap());
    headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".parse().unwrap());

    let query = [("pr", "ucpro"), ("fr", "pc")];
    let req = client.request(method, url).headers(headers).query(&query);
    let res = callback(req).send().await?;

    let status = res.status();
    let body = res.text().await?;

    if status.is_success() {
        // 先解析为 Value 以检查 API 返回的错误信息
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| WebDavError::Internal(format!("解析响应失败: {}", e)))?;

        if let Some(code) = value.get("code").and_then(|c| c.as_i64()) {
            if code != 0 {
                let msg = value
                    .get("msg")
                    .and_then(|m| m.as_str())
                    .unwrap_or("未知错误")
                    .to_string();
                tracing::error!("API 错误, code: {}, msg: {}", code, msg);
                return Err(WebDavError::Internal(format!("API 错误: {}", msg)));
            }
        }

        serde_json::from_value(value)
            .map_err(|e| WebDavError::Internal(format!("解析响应数据失败: {}", e)))
    } else {
        match status.as_u16() {
            404 => Err(WebDavError::NotFound(path.into())),
            403 => Err(WebDavError::PermissionDenied(path.into())),
            401 => Err(WebDavError::InvalidInput("认证失败".into())),
            _ => Err(WebDavError::Internal(format!(
                "请求失败: {} - {}",
                status, body
            ))),
        }
    }
}
