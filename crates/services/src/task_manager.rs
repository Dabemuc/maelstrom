use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use io::catalog::EditGraph;
use previews::preview_generation::{generate_preview_for_image, generate_preview_for_image_with_graph};
use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use crate::catalog::CatalogService;
use crate::event_bus::EventBus;
use crate::events::{ServiceEvent, TaskId};
use crate::ImportStrategy;

#[derive(Debug)]
pub struct TaskManager {
    bus: Arc<EventBus>,
    /// Handle to the tokio runtime captured at construction time (inside an async context).
    /// Used to spawn tasks from non-async contexts (e.g. the iced update function on the
    /// macOS main thread, which runs outside the tokio runtime).
    rt: Handle,
    next_id: AtomicU64,
    handles: Mutex<HashMap<TaskId, JoinHandle<()>>>,
}

impl TaskManager {
    /// Must be called from within a tokio runtime (e.g. inside a `Task::perform` closure)
    /// so that `Handle::current()` succeeds.
    pub fn new(bus: Arc<EventBus>) -> Arc<Self> {
        Arc::new(Self {
            bus,
            rt: Handle::current(),
            next_id: AtomicU64::new(1),
            handles: Mutex::new(HashMap::new()),
        })
    }

    pub fn cancel(&self, task_id: TaskId) {
        if let Some(handle) = self.handles.lock().unwrap().remove(&task_id) {
            handle.abort();
        }
    }

    /// Imports fotos into a managed directory and generates previews for each imported image
    /// in parallel. Progress is emitted as [`ServiceEvent::PreviewGenerated`] per image and
    /// [`ServiceEvent::ImportCompleted`] once all previews are done.
    pub fn spawn_import_with_previews(
        self: &Arc<Self>,
        catalog_service: Arc<CatalogService>,
        strategy: ImportStrategy,
        import_path: PathBuf,
        managed_dir: PathBuf,
    ) -> TaskId {
        let task_id = TaskId::new(self.next_id.fetch_add(1, Ordering::Relaxed));
        let bus = self.bus.clone();

        let handle = self.rt.spawn(async move {
            // 1. Import files (sequential — moves/copies files on disk)
            let payload = catalog_service
                .import_fotos_into_managed_dir_with_strategy(strategy, import_path, managed_dir)
                .await;

            // 2. Fan out: one tokio::spawn per image for true parallelism
            let preview_handles: Vec<JoinHandle<()>> = payload
                .imported_items
                .iter()
                .filter(|item| !item.hash.is_empty() && item.dest_path.is_file())
                .map(|item| {
                    let bus = bus.clone();
                    let dest_path = item.dest_path.clone();
                    let hash = item.hash.clone();
                    let catalog_clone = catalog_service.get_catalog_ref().clone();

                    tokio::spawn(async move {
                        let result = generate_preview_for_image_with_graph(
                            dest_path,
                            hash,
                            EditGraph::default(),
                            &catalog_clone,
                        )
                        .await;
                        bus.emit(ServiceEvent::PreviewGenerated { task_id, result });
                    })
                })
                .collect();

            // 3. Wait for all previews before signalling overall completion
            for handle in preview_handles {
                let _ = handle.await;
            }

            bus.emit(ServiceEvent::ImportCompleted { task_id, payload });
        });

        self.handles.lock().unwrap().insert(task_id, handle);
        task_id
    }

    /// Syncs the catalog with the filesystem for a directory and generates previews for any
    /// new images in parallel. Emits [`ServiceEvent::SyncCompleted`] immediately after the
    /// catalog diff (so the UI can populate existing previews), then emits
    /// [`ServiceEvent::PreviewGenerated`] as each new preview finishes.
    pub fn spawn_sync_with_previews(
        self: &Arc<Self>,
        catalog_service: Arc<CatalogService>,
        request_id: u64,
        path: PathBuf,
    ) -> TaskId {
        let task_id = TaskId::new(self.next_id.fetch_add(1, Ordering::Relaxed));
        let bus = self.bus.clone();

        let handle = self.rt.spawn(async move {
            // 1. Sync catalog with filesystem (fast — no preview gen inside anymore)
            let result = catalog_service
                .sync_catalog_with_fs_for_dir(request_id, path)
                .await;

            let result = match result {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[TaskManager] Sync failed: {}", e);
                    return;
                }
            };

            // 2. Emit sync result immediately so the UI can show existing previews
            let images_to_generate = result.images_to_add_to_catalog.clone();
            bus.emit(ServiceEvent::SyncCompleted { task_id, result });

            // 3. Fan out: one tokio::spawn per new image for true parallelism
            let preview_handles: Vec<JoinHandle<()>> = images_to_generate
                .into_iter()
                .map(|path| {
                    let bus = bus.clone();
                    let catalog_clone = catalog_service.get_catalog_ref().clone();

                    tokio::spawn(async move {
                        let result =
                            generate_preview_for_image(path, &catalog_clone, false).await;
                        bus.emit(ServiceEvent::PreviewGenerated { task_id, result });
                    })
                })
                .collect();

            for handle in preview_handles {
                let _ = handle.await;
            }
        });

        self.handles.lock().unwrap().insert(task_id, handle);
        task_id
    }
}
