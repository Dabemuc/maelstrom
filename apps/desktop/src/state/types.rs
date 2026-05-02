#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewMode {
    Library,
    #[allow(dead_code)]
    Develop,
    NoCatalog,
}
