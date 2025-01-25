use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Resp {
    pub status: i32,
    pub code: i32,
    pub message: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SortResp {
    #[serde(flatten)]
    pub resp: Resp,
    pub data: SortData,
    pub metadata: SortMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SortData {
    pub list: Vec<List>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SortMetadata {
    #[serde(rename = "_size")]
    pub size: i32,
    #[serde(rename = "_page")]
    pub page: i32,
    #[serde(rename = "_count")]
    pub count: i32,
    #[serde(rename = "_total")]
    pub total: i32,
    pub way: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct List {
    pub fid: String,
    pub file_name: String,
    pub size: i64,
    pub file: bool,
    pub l_updated_at: Option<i64>,
    pub updated_at: i64,
    pub created_at: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DownResp {
    #[serde(flatten)]
    pub resp: Resp,
    pub data: Vec<DownData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownData {
    pub download_url: String,
    pub range_size: i64,
    pub size: i64,
}