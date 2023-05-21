pub mod strategy;

use crate::power_interruptions::location::{AffectedLine, NairobiTZDateTime};
use crate::subscriptions::AffectedSubscriber;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Notification {
    pub url: Url,
    pub subscriber: AffectedSubscriber,
    pub lines: Vec<AffectedLine<NairobiTZDateTime>>,
}

impl Notification {
    pub fn already_sent(&self) -> bool {
        self.lines.is_empty()
    }
}
