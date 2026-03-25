use io::catalog::ImageDO;
use previews::preview_generation::PreviewGenerationError;

use crate::types::{CatalogSyncResult, ImportCompletedPayload};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone)]
pub enum ServiceEvent {
    PreviewGenerated {
        task_id: TaskId,
        result: Result<ImageDO, PreviewGenerationError>,
    },
    ImportCompleted {
        task_id: TaskId,
        payload: ImportCompletedPayload,
    },
    SyncCompleted {
        task_id: TaskId,
        result: CatalogSyncResult,
    },
}
