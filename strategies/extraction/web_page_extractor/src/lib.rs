use std::sync::Arc;
use use_cases::import_planned_blackouts::ImportPlannedBlackoutsInteractor;

struct WebPageExtractor {
    importer: Arc<dyn ImportPlannedBlackoutsInteractor>,
}
