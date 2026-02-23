pub fn load_imported_directories() {
    if let Some(catalog) = &self.catalog {
        let catalog_clone = catalog.clone();
        return Task::perform(
            async move { catalog_clone.get_imported_directories().await },
            Message::ImportedDirectoriesLoadAttempted,
        );
    }
}