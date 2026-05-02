Phase 1 — Thin service wrappers (lowest risk, quick wins)

- [ ] Move preview file resolution + metadata/size extraction into a PreviewDataService that returns PreviewData (currently in apps/desktop/src/update/helpers.rs and used by apps/desktop/src/update/preview.rs).
- [ ] Add a PreviewService::load_preview_bytes(hash) (or similar) to centralize fs::read and path construction (currently in apps/desktop/src/update/preview.rs).
- [ ] Create WorkspaceService::scan_folder(path) that wraps io::image_files::helpers::scan_folder_images and returns the scan payload (currently in apps/desktop/src/update/managed.rs and apps/desktop/src/update/workspace.rs).

  Phase 2 — Orchestration extraction (still straightforward)

- [ ] Create WorkspaceService::refresh_root(root) to unify the thread-spawn + scan + message payload logic now repeated in managed.rs and workspace.rs.
- [ ] Move selection “refresh from cache” logic to WorkspaceService::apply_selection_cache (currently in apps/desktop/src/update/helpers.rs::refresh_selected_previews_from_cache) so both CLI and desktop can reuse it.
- [ ] Add PreviewService::generate_preview_for_image(path, graph) to encapsulate previews::preview_generation calls currently in apps/desktop/src/update/import.rs and apps/desktop/src/update/develop.rs.

  Phase 3 — Import flow

- [ ] Introduce ImportService::plan_import(root, source_path) wrapping scan_folder_images + create_import_plan.
- [ ] Add ImportService::execute_import(plan) that wraps execute_import_plan and returns a report.
- [ ] Expose a single ImportService::import_into_root(root, source_path) that returns (report, imported_items); desktop can keep the file dialog, CLI can pass paths directly.

  Phase 4 — Develop workflow

- [ ] Add DevelopService::load_state(hash) that fetches edit graph + loads LinearImage (currently split across apps/desktop/src/update/workspace.rs and apps/desktop/src/state/develop.rs).
- [ ] Add DevelopService::apply_graph(state) that executes the graph (currently in apps/desktop/src/update/develop.rs).
- [ ] Add DevelopService::save_edit_graph(hash, graph) + DevelopService::regenerate_preview(hash, graph) to unify the “save + preview” flow.

  Phase 5 — Consolidation and cleanup

- [ ] Remove direct io, previews, graph, image dependencies from apps/desktop (only services should remain).
- [ ] Move any remaining orchestration (thread spawning, batching, cache diffing) into services.
- [ ] Add CLI adapters to call the same services for import/scan/preview/develop.
