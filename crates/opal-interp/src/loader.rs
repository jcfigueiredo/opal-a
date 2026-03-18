use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct SearchRoot {
    source_root: PathBuf,
    package_prefix: Option<String>,
    package_root_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolveError {
    pub module_path: Vec<String>,
    pub searched_paths: Vec<PathBuf>,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "module '{}' not found", self.module_path.join("."))
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub file_path: PathBuf,
    pub canonical_name: String,
    pub searched_paths: Vec<PathBuf>,
}

/// Resolves module paths to .opl files using script or package roots.
pub struct ModuleLoader {
    search_roots: Vec<SearchRoot>,
}

impl ModuleLoader {
    pub fn new(base_dir: &Path) -> Self {
        let mut search_roots = Vec::new();
        push_search_root(&mut search_roots, SearchRoot::discover(base_dir, true));

        if let Ok(opal_path) = std::env::var("OPAL_PATH") {
            for entry in opal_path.split(':') {
                if entry.is_empty() {
                    continue;
                }
                let path = PathBuf::from(entry);
                push_search_root(&mut search_roots, SearchRoot::discover(&path, false));
            }
        }

        Self { search_roots }
    }

    /// Resolve a module path like ["MyWebApp", "Routes"] to a .opl file.
    pub fn resolve(&self, module_path: &[String]) -> Result<ResolvedModule, ResolveError> {
        let mut searched_paths = Vec::new();

        for search_root in &self.search_roots {
            let Some(candidates) = search_root.candidates(module_path) else {
                continue;
            };

            for candidate in candidates {
                searched_paths.push(candidate.clone());
                if candidate.exists() {
                    return Ok(ResolvedModule {
                        file_path: normalize_existing_path(&candidate),
                        canonical_name: module_path.join("."),
                        searched_paths,
                    });
                }
            }
        }

        Err(ResolveError {
            module_path: module_path.to_vec(),
            searched_paths,
        })
    }
}

impl SearchRoot {
    fn discover(base_dir: &Path, walk_ancestors: bool) -> Option<Self> {
        if let Some(package_root) = find_package_root(base_dir, walk_ancestors) {
            let source_root = if package_root.join("src").is_dir() {
                package_root.join("src")
            } else {
                package_root.clone()
            };

            let package_name = parse_package_name(&package_root.join("opal.toml"));
            let package_prefix = package_name
                .as_deref()
                .map(package_name_to_module_prefix)
                .filter(|s| !s.is_empty());
            let package_root_file = package_name
                .as_deref()
                .map(package_name_to_file_stem)
                .filter(|s| !s.is_empty());

            return Some(Self {
                source_root,
                package_prefix,
                package_root_file,
            });
        }

        if base_dir.is_dir() {
            Some(Self {
                source_root: base_dir.to_path_buf(),
                package_prefix: None,
                package_root_file: None,
            })
        } else {
            None
        }
    }

    fn candidates(&self, module_path: &[String]) -> Option<Vec<PathBuf>> {
        let relative_segments = match &self.package_prefix {
            Some(prefix) => {
                let first = module_path.first()?;
                if first != prefix {
                    return None;
                }
                &module_path[1..]
            }
            None => module_path,
        };

        if relative_segments.is_empty() {
            let root_file = self.package_root_file.as_ref()?;
            return Some(vec![self.source_root.join(format!("{}.opl", root_file))]);
        }

        let file_segments: Vec<String> = relative_segments
            .iter()
            .map(|segment| module_segment_to_file_segment(segment))
            .collect();

        let mut base = self.source_root.clone();
        for segment in &file_segments[..file_segments.len().saturating_sub(1)] {
            base.push(segment);
        }

        let leaf = file_segments.last()?;
        let mut candidates = Vec::with_capacity(2);
        candidates.push(base.join(format!("{}.opl", leaf)));
        candidates.push(base.join(leaf).join("index.opl"));
        Some(candidates)
    }
}

fn push_search_root(search_roots: &mut Vec<SearchRoot>, candidate: Option<SearchRoot>) {
    let Some(candidate) = candidate else {
        return;
    };

    if search_roots.iter().any(|existing| {
        existing.source_root == candidate.source_root
            && existing.package_prefix == candidate.package_prefix
            && existing.package_root_file == candidate.package_root_file
    }) {
        return;
    }

    search_roots.push(candidate);
}

fn find_package_root(base_dir: &Path, walk_ancestors: bool) -> Option<PathBuf> {
    let mut current = if base_dir.is_dir() {
        base_dir.to_path_buf()
    } else {
        base_dir.parent()?.to_path_buf()
    };

    loop {
        if current.join("opal.toml").is_file() {
            return Some(current);
        }

        if !walk_ancestors {
            break;
        }

        if !current.pop() {
            break;
        }
    }

    None
}

fn parse_package_name(manifest_path: &Path) -> Option<String> {
    let manifest = fs::read_to_string(manifest_path).ok()?;
    let mut in_package_section = false;

    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_package_section = trimmed == "[package]";
            continue;
        }

        if !in_package_section {
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if key.trim() != "name" {
            continue;
        }

        let value = value.trim().trim_matches('"').trim_matches('\'');
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }

    None
}

fn package_name_to_module_prefix(name: &str) -> String {
    let mut out = String::new();
    let mut capitalize_next = true;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if capitalize_next {
                out.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                out.push(ch.to_ascii_lowercase());
            }
        } else if ch.is_alphanumeric() {
            out.push(ch);
            capitalize_next = false;
        } else {
            capitalize_next = true;
        }
    }

    out
}

fn package_name_to_file_stem(name: &str) -> String {
    let mut out = String::new();

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch.is_alphanumeric() {
            out.push(ch);
        } else if !out.ends_with('_') && !out.is_empty() {
            out.push('_');
        }
    }

    out.trim_end_matches('_').to_string()
}

fn module_segment_to_file_segment(segment: &str) -> String {
    let chars: Vec<char> = segment.chars().collect();
    let mut out = String::new();

    for (idx, ch) in chars.iter().enumerate() {
        if ch.is_ascii_uppercase() {
            let prev = idx.checked_sub(1).and_then(|i| chars.get(i));
            let next = chars.get(idx + 1);
            let needs_separator = prev.is_some_and(|prev| {
                prev.is_ascii_lowercase()
                    || prev.is_ascii_digit()
                    || (prev.is_ascii_uppercase()
                        && next.is_some_and(|next| next.is_ascii_lowercase()))
            });

            if needs_separator && !out.ends_with('_') {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else if *ch == '-' || *ch == ' ' {
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
        } else {
            out.push(*ch);
        }
    }

    out
}

fn normalize_existing_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("opal-loader-{prefix}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn resolves_script_modules_from_base_dir() {
        let root = temp_dir("script");
        fs::write(root.join("rewards.opl"), "export { calculate_reward }\n").unwrap();

        let loader = ModuleLoader::new(&root);
        let resolved = loader.resolve(&["Rewards".to_string()]).unwrap();

        assert!(resolved.file_path.ends_with("rewards.opl"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolves_package_prefixed_modules_from_src() {
        let root = temp_dir("package");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("opal.toml"),
            "[package]\nname = \"my_web_app\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(root.join("src").join("routes.opl"), "export { router }\n").unwrap();

        let loader = ModuleLoader::new(&root.join("src"));
        let resolved = loader
            .resolve(&["MyWebApp".to_string(), "Routes".to_string()])
            .unwrap();

        assert!(resolved.file_path.ends_with("src/routes.opl"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolves_directory_modules_via_index_file() {
        let root = temp_dir("index");
        fs::create_dir_all(root.join("src").join("models")).unwrap();
        fs::write(
            root.join("opal.toml"),
            "[package]\nname = \"my_web_app\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            root.join("src").join("models").join("index.opl"),
            "export { User }\n",
        )
        .unwrap();

        let loader = ModuleLoader::new(&root.join("src"));
        let resolved = loader
            .resolve(&["MyWebApp".to_string(), "Models".to_string()])
            .unwrap();

        assert!(resolved.file_path.ends_with("src/models/index.opl"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn normalizes_pascal_case_module_segments_to_snake_case_files() {
        assert_eq!(module_segment_to_file_segment("MyWebApp"), "my_web_app");
        assert_eq!(module_segment_to_file_segment("HTTPServer"), "http_server");
        assert_eq!(module_segment_to_file_segment("数学"), "数学");
    }

    #[test]
    fn resolve_error_display() {
        let err = ResolveError {
            module_path: vec!["Math".to_string(), "Vector".to_string()],
            searched_paths: vec![PathBuf::from("/tmp/math/vector.opl")],
        };
        assert_eq!(err.to_string(), "module 'Math.Vector' not found");
    }

    #[test]
    fn resolve_nonexistent_module_returns_error() {
        let root = temp_dir("notfound");
        let loader = ModuleLoader::new(&root);
        let result = loader.resolve(&["Nonexistent".to_string()]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(!err.searched_paths.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn discover_nonexistent_dir_returns_none() {
        let result = SearchRoot::discover(Path::new("/nonexistent/path"), false);
        assert!(result.is_none());
    }

    #[test]
    fn discover_dir_without_package_file() {
        let root = temp_dir("nopackage");
        let result = SearchRoot::discover(&root, false);
        assert!(result.is_some());
        let sr = result.unwrap();
        assert!(sr.package_prefix.is_none());
        assert!(sr.package_root_file.is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn package_name_to_file_stem_cases() {
        assert_eq!(package_name_to_file_stem("my_web_app"), "my_web_app");
        assert_eq!(package_name_to_file_stem("opal-http"), "opal_http");
    }

    #[test]
    fn package_name_to_module_prefix_cases() {
        assert_eq!(package_name_to_module_prefix("my_web_app"), "MyWebApp");
        assert_eq!(package_name_to_module_prefix("opal-http"), "OpalHttp");
    }
}
