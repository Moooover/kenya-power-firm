
use serde::Deserialize;
use serde::Serialize;
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StatusCode {
    OK,
    #[serde(rename = "ZERO_RESULTS")]
    ZERORESULTS,
    #[serde(rename = "INVALID_REQUEST")]
    INVALIDREQUEST,
    #[serde(rename = "OVER_QUERY_LIMIT")]
    OVERQUERYLIMIT,
    #[serde(rename = "REQUEST_DENIED")]
    REQUESTDENIED,
    #[serde(rename = "UNKNOWN_ERROR")]
    UNKNOWNERROR,
}

impl StatusCode {
    pub fn is_cacheable(&self) -> bool {
        matches!(self, StatusCode::OK | StatusCode::ZERORESULTS)
    }
}
