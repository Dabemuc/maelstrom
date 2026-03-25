use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use io::{
    catalog::{
        EditGraph, ImageDO,
        catalog::{CATALOG_FILE_NAME, CATALOG_FOLDER_NAME, Catalog},
        catalog_error::CatalogError,
    },
    image_files::helpers::scan_folder_images,
    import::{ImportDecision, create_import_plan, execute_import_plan},
};
use previews::preview_generation::{
    generate_preview_for_image, generate_preview_for_image_with_graph,
};
use tokio::task::JoinHandle;

use crate::{ImportStrategy, types::ImportCompletedPayload};

use crate::{
    catalog::{
        catalog_sync::{compare_catalog_to_fs, sync_catalog_with_fs_for_dir},
        preview_data::preview_data_from_image_do,
    },
    error::ServiceError,
    events::{ServiceEvent, TaskId},
    task_manager::TaskManager,
    types::{CatalogSyncResult, PreviewData},
};

#[derive(Debug)]
pub struct CatalogService {
    catalog: Catalog,
    task_manager: OnceLock<Arc<TaskManager>>,
}

impl CatalogService {
    pub async fn init(config_base: PathBuf) -> Result<Self, ServiceError> {
        // Load catalog file
        if !config_base.exists() {
            println!("User config dir doesnt exists at {:?}", config_base);
            return Err(ServiceError::Catalog(
                CatalogError::MissingCatalogBaseDirectory(config_base.to_str().unwrap().to_owned()),
            ));
        }
        let catalog_root = config_base.join(CATALOG_FOLDER_NAME);

        let catalog = if catalog_root.join(CATALOG_FILE_NAME).exists() {
            println!("Catalog file exists; loading catalog");
            Catalog::load(catalog_root.clone()).await?
        } else {
            println!("default catalog not found, creating at: {:?}", catalog_root);
            Catalog::create(config_base.clone()).await?
        };

        Ok(Self {
            catalog,
            task_manager: OnceLock::new(),
        })
    }

    /// Called by [`Services`] after construction to wire up the shared async runtime.
    pub(crate) fn inject_task_manager(&self, tm: Arc<TaskManager>) {
        let _ = self.task_manager.set(tm);
    }

    fn tm(&self) -> &Arc<TaskManager> {
        self.task_manager
            .get()
            .expect("task_manager not injected — call Services::new")
    }

    // TODO: Should eventually be removed
    pub fn get_catalog_ref(&self) -> &Catalog {
        &self.catalog
    }

    pub async fn get_managed_directories(&self) -> Result<Vec<PathBuf>, ServiceError> {
        Ok(self.catalog.get_managed_directories().await?)
    }

    pub async fn add_managed_directory(&self, path: impl AsRef<Path>) -> Result<(), ServiceError> {
        Ok(self.catalog.add_managed_directory(path).await?)
    }

    pub async fn get_edit_graph(&self, content_hash: &str) -> Result<EditGraph, ServiceError> {
        Ok(self.catalog.get_edit_graph(content_hash).await?)
    }

    pub async fn get_all_image_dos_for_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Vec<ImageDO>, ServiceError> {
        Ok(self.catalog.get_all_image_dos_for_path(path).await?)
    }

    pub async fn set_edit_graph(
        &self,
        content_hash: &str,
        graph: &EditGraph,
    ) -> Result<(), ServiceError> {
        Ok(self.catalog.set_edit_graph(content_hash, graph).await?)
    }

    // TODO: Should eventually be removed
    pub fn root(&self) -> &Path {
        self.catalog.root()
    }

    // TODO: Should eventually be removed
    pub fn cache_dir(&self) -> &Path {
        self.catalog.cache_dir()
    }

    // TODO: Should eventually be removed
    pub fn preview_cache_dir(&self) -> &Path {
        self.catalog.preview_cache_dir()
    }

    // TODO: Should eventually be removed
    pub fn develop_cache_dir(&self) -> &Path {
        self.catalog.develop_cache_dir()
    }

    // TODO: Should be renamed (whole process, aka all involved files/methods)
    /// Compares catalog to fs for path and adds new images to catalog
    pub async fn sync_catalog_with_fs_for_dir(
        &self,
        request_id: u64,
        selected_path: PathBuf,
    ) -> Result<CatalogSyncResult, ServiceError> {
        sync_catalog_with_fs_for_dir(&self.catalog, request_id, selected_path).await
    }

    /// Builds [`PreviewData`] for a slice of image DOs by reading metadata and checking the
    /// preview cache directory. Used by the async task manager for the fast first-phase emit.
    pub fn build_preview_data(&self, image_dos: &[ImageDO]) -> Vec<PreviewData> {
        image_dos
            .iter()
            .map(|image_do| preview_data_from_image_do(&self.catalog, image_do))
            .collect()
    }

    /// Scans the filesystem for `path` and returns which image files are not yet in the catalog
    /// and which catalog entries no longer exist on disk. CPU-bound; intended to be called from
    /// a background task.
    pub fn diff_dir_with_catalog(
        &self,
        path: PathBuf,
        image_dos: Vec<ImageDO>,
    ) -> (Vec<PathBuf>, Vec<ImageDO>) {
        let scan = scan_folder_images(path);
        compare_catalog_to_fs(scan.all_image_paths, image_dos)
    }

    /// Imports fotos into managed directory by copying them according to an import strategy and
    /// adding them to catalog afterwards.
    ///
    /// # Arguments
    ///
    /// * `import_strategy` - The copy strategy of the import
    /// * `import_path` - The path of the directory to import from
    /// * `managed_dir_path` - The path of the managed directory to import into
    pub async fn import_fotos_into_managed_dir_with_strategy(
        &self,
        import_strategy: ImportStrategy,
        import_path: PathBuf,
        managed_dir_path: PathBuf,
    ) -> ImportCompletedPayload {
        // Read all images to import
        let scan_result = scan_folder_images(import_path.clone());
        println!(
            "Importing {} images into {:?}",
            scan_result.all_image_paths.len(),
            import_path.to_str()
        );

        let already_imported_images = match self
            .catalog
            .get_all_image_dos_for_path(managed_dir_path.clone())
            .await
        {
            Ok(value) => value,
            Err(err) => {
                return ImportCompletedPayload {
                    summary: format!("Failed to load already imported images for path: {}", err),
                    imported_items: Vec::new(),
                    root: managed_dir_path.clone(),
                };
            }
        };
        let existing_hashes: HashSet<String> = already_imported_images
            .into_iter()
            .map(|img| img.hash)
            .collect();
        let plan = create_import_plan(
            managed_dir_path.clone(),
            &scan_result,
            &existing_hashes,
            import_strategy,
        );
        let imported_items = plan
            .items
            .iter()
            .filter(|item| item.decision == ImportDecision::Import)
            .cloned()
            .collect();
        let report = execute_import_plan(plan, &self.catalog).await;

        let summary = format!(
            "Imported {} new images out of {} total. Skipped {}, errors {}.",
            report.imported_count,
            scan_result.all_image_paths.len(),
            report.skipped_count,
            report.errors.len()
        );

        ImportCompletedPayload {
            summary,
            imported_items,
            root: managed_dir_path.clone(),
        }
    }

    /// Imports fotos into a managed directory and generates previews for each imported image
    /// in parallel. Progress is emitted as [`ServiceEvent::PreviewGenerated`] per image and
    /// [`ServiceEvent::ImportCompleted`] once all previews are done.
    pub fn spawn_import_with_previews(
        self: &Arc<Self>,
        strategy: ImportStrategy,
        import_path: PathBuf,
        managed_dir: PathBuf,
    ) -> TaskId {
        let bus = self.tm().bus.clone();
        let catalog_service = self.clone();

        self.tm().spawn(|task_id| async move {
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
        })
    }

    /// Syncs the catalog with the filesystem for a directory and generates previews for any
    /// new images in parallel.
    ///
    /// Emits two [`ServiceEvent::SyncCompleted`] events:
    /// 1. Immediately after the DB query — carries existing image/preview data so the UI can
    ///    populate right away without waiting for the filesystem scan.
    /// 2. After the FS scan — carries `catalog_image_dos_to_delete` so stale entries are marked.
    ///
    /// Then emits [`ServiceEvent::PreviewGenerated`] as each new preview finishes.
    pub fn spawn_sync_with_previews(self: &Arc<Self>, request_id: u64, path: PathBuf) -> TaskId {
        let bus = self.tm().bus.clone();
        let catalog_service = self.clone();

        self.tm().spawn(|task_id| async move {
            // Phase 1: DB query only — O(1) regardless of directory size, no disk reads per image
            let image_dos = match catalog_service.get_all_image_dos_for_path(&path).await {
                Ok(dos) => dos,
                Err(e) => {
                    eprintln!("[Sync] DB query failed: {}", e);
                    return;
                }
            };

            // Emit immediately with empty preview_data: the UI builds quick previews from
            // image_dos without reading EXIF or image dimensions (see handle_selection_synced).
            bus.emit(ServiceEvent::SyncCompleted {
                task_id,
                result: CatalogSyncResult {
                    request_id,
                    selected_path: path.clone(),
                    image_dos: image_dos.clone(),
                    preview_data: vec![],
                    images_to_add_to_catalog: vec![],
                    catalog_image_dos_to_delete: vec![],
                    generated: vec![],
                },
            });

            // Phase 2: FS scan + hash comparison — blocking I/O, must not run on an async
            // worker thread or it starves the executor and delays subscription message delivery.
            let catalog_for_scan = catalog_service.clone();
            let path_for_scan = path.clone();
            let (images_to_add, images_to_delete) = tokio::task::spawn_blocking(move || {
                catalog_for_scan.diff_dir_with_catalog(path_for_scan, image_dos)
            })
            .await
            .expect("diff_dir_with_catalog panicked");

            // Emit stale entries so the UI can mark them as missing
            if !images_to_delete.is_empty() {
                bus.emit(ServiceEvent::SyncCompleted {
                    task_id,
                    result: CatalogSyncResult {
                        request_id,
                        selected_path: path.clone(),
                        image_dos: vec![],
                        preview_data: vec![],
                        images_to_add_to_catalog: vec![],
                        catalog_image_dos_to_delete: images_to_delete,
                        generated: vec![],
                    },
                });
            }

            // Phase 3: Fan out preview generation for new images — one task per image
            let preview_handles: Vec<JoinHandle<()>> = images_to_add
                .into_iter()
                .map(|img_path| {
                    let bus = bus.clone();
                    let catalog_clone = catalog_service.get_catalog_ref().clone();

                    tokio::spawn(async move {
                        let result =
                            generate_preview_for_image(img_path, &catalog_clone, false).await;
                        bus.emit(ServiceEvent::PreviewGenerated { task_id, result });
                    })
                })
                .collect();

            for handle in preview_handles {
                let _ = handle.await;
            }
        })
    }
}
