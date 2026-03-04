use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Resolves module paths to .opl files and tracks loading state
pub struct ModuleLoader {
    search_paths: Vec<PathBuf>,
    loading: HashSet<String>,
    loaded: HashSet<String>,
}

impl ModuleLoader {
    pub fn new(base_dir: &Path) -> Self {
        let mut search_paths = vec![base_dir.to_path_buf()];
        if let Ok(opal_path) = std::env::var("OPAL_PATH") {
            for p in opal_path.split(':') {
                if !p.is_empty() {
                    search_paths.push(PathBuf::from(p));
                }
            }
        }
        Self {
            search_paths,
            loading: HashSet::new(),
            loaded: HashSet::new(),
        }
    }

    /// Resolve a module path like ["Math", "Vector"] to a .opl file
    pub fn resolve(&self, module_path: &[String]) -> Option<PathBuf> {
        let filename = module_path.last()?.to_lowercase();
        let dir_parts: Vec<String> = module_path[..module_path.len().saturating_sub(1)]
            .iter()
            .map(|s| s.to_lowercase())
            .collect();

        for search_dir in &self.search_paths {
            // Try: dir/subdir/file.opl
            let mut path = search_dir.clone();
            for part in &dir_parts {
                path.push(part);
            }
            path.push(format!("{}.opl", filename));
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    /// Returns false if circular dependency detected
    pub fn mark_loading(&mut self, key: &str) -> bool {
        self.loading.insert(key.to_string())
    }

    pub fn mark_loaded(&mut self, key: &str) {
        self.loading.remove(key);
        self.loaded.insert(key.to_string());
    }

    pub fn is_loaded(&self, key: &str) -> bool {
        self.loaded.contains(key)
    }
}
