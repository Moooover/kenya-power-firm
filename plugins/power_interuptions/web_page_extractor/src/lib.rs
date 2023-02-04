pub mod text_extractor;

use crate::text_extractor::TextExtractor;
use async_trait::async_trait;
use std::sync::Arc;
use use_cases::import_planned_blackouts::ImportPlannedBlackoutsInteractor;

struct WebPageExtractor {
    importer: Arc<dyn ImportPlannedBlackoutsInteractor>,
    extractor: Arc<dyn TextExtractor>,
}
