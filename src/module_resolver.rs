//! # Module Resolver
//!
//! Resolves module imports, loads module files, builds dependency graph,
//! and detects circular dependencies.
//!
//! ## Design Philosophy
//!
//! The module resolver is responsible for:
//! - Resolving import paths (relative, absolute, standard library)
//! - Loading and parsing module files
//! - Building a dependency graph
//! - Detecting circular dependencies
//! - Caching loaded modules
//!
//! ## Path Resolution Order
//!
//! 1. **Relative paths**: `./math.gw` or `../lib/utils.gw` - resolved relative to importing file
//! 2. **Absolute paths**: `std/math.gw` - resolved from project root
//! 3. **Standard library**: `std/math.gw` - resolved from standard library directory

use crate::ast::AstNode;
use crate::lexer::Lexer;
use crate::parser::Parser;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;

/// Result type for module resolution operations
pub type ResolverResult<T> = Result<T, ResolverError>;

/// Errors that can occur during module resolution
#[derive(Debug, Clone, PartialEq)]
pub enum ResolverError {
    /// Module file not found at specified path
    ModuleNotFound {
        path: String,
        searched_paths: Vec<String>,
    },

    /// Circular dependency detected
    CircularDependency {
        cycle: Vec<String>,
    },

    /// Parse error while loading module
    ParseError {
        path: String,
        error: String,
    },

    /// I/O error reading module file
    IoError {
        path: String,
        message: String,
    },

    /// Invalid module path
    InvalidPath {
        path: String,
        reason: String,
    },
}

/// Information about a loaded module
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// Canonical path to the module file
    pub path: String,

    /// Module name (extracted from grove declaration or filename)
    pub name: String,

    /// Parsed AST of the module
    pub ast: Vec<AstNode>,

    /// List of modules this module imports (paths)
    pub dependencies: Vec<String>,

    /// List of exported symbols
    pub exports: Vec<String>,
}

/// Module resolver - loads and resolves module dependencies
pub struct ModuleResolver {
    /// Project root directory (where main.gw is located)
    project_root: String,

    /// Standard library directory path
    stdlib_path: String,

    /// Cache of loaded modules (path -> ModuleInfo)
    module_cache: BTreeMap<String, ModuleInfo>,

    /// Dependency graph (module_path -> list of dependency paths)
    dependency_graph: BTreeMap<String, Vec<String>>,

    /// Currently being loaded (for cycle detection)
    loading_stack: Vec<String>,
}

impl ModuleResolver {
    /// Create a new module resolver
    ///
    /// # Arguments
    /// * `project_root` - Root directory of the project
    /// * `stdlib_path` - Path to standard library directory
    pub fn new(project_root: String, stdlib_path: String) -> Self {
        ModuleResolver {
            project_root,
            stdlib_path,
            module_cache: BTreeMap::new(),
            dependency_graph: BTreeMap::new(),
            loading_stack: Vec::new(),
        }
    }

    /// Resolve an import path to a canonical file path
    ///
    /// Resolution order:
    /// 1. If path starts with `./` or `../`: relative to importer
    /// 2. Otherwise: try project root, then stdlib
    ///
    /// # Arguments
    /// * `import_path` - Path from import statement (e.g., "std/math.gw")
    /// * `importer_path` - Path of the file doing the importing (for relative resolution)
    pub fn resolve_path(&self, import_path: &str, importer_path: Option<&str>) -> ResolverResult<String> {
        let mut searched = Vec::new();

        // 1. Check for relative path
        if import_path.starts_with("./") || import_path.starts_with("../") {
            if let Some(importer) = importer_path {
                let resolved = Self::resolve_relative(import_path, importer);
                searched.push(resolved.clone());

                // In real implementation, would check if file exists
                // For now, just return the resolved path
                return Ok(resolved);
            } else {
                return Err(ResolverError::InvalidPath {
                    path: import_path.to_string(),
                    reason: "Relative path used but no importer specified".to_string(),
                });
            }
        }

        // 2. Try project root
        let project_path = format!("{}/{}", self.project_root, import_path);
        searched.push(project_path.clone());

        // In real implementation, would check if file exists
        // For Phase 2, we'll just simulate this

        // 3. Try stdlib
        let stdlib_path = format!("{}/{}", self.stdlib_path, import_path);
        searched.push(stdlib_path.clone());

        // For now, prefer stdlib if path starts with "std/"
        if import_path.starts_with("std/") {
            return Ok(stdlib_path);
        }

        // Otherwise prefer project root
        Ok(project_path)
    }

    /// Resolve a relative path
    ///
    /// # Arguments
    /// * `relative_path` - Relative path (e.g., "../lib/utils.gw")
    /// * `base_path` - Base path to resolve from
    fn resolve_relative(relative_path: &str, base_path: &str) -> String {
        // Get directory of base path
        let base_dir = if let Some(pos) = base_path.rfind('/') {
            &base_path[..pos]
        } else {
            "."
        };

        // Combine paths (simplified for Phase 2)
        format!("{}/{}", base_dir, relative_path)
    }

    /// Load a module from a file path
    ///
    /// This reads the file, parses it, and extracts module information.
    ///
    /// # Arguments
    /// * `path` - Canonical path to the module file
    pub fn load_module(&mut self, path: &str) -> ResolverResult<&ModuleInfo> {
        // Check if already cached
        if self.module_cache.contains_key(path) {
            return Ok(&self.module_cache[path]);
        }

        // Check for circular dependency
        if self.loading_stack.contains(&path.to_string()) {
            // Build cycle path
            let mut cycle = Vec::new();
            let mut found = false;
            for p in &self.loading_stack {
                if found || p == path {
                    cycle.push(p.clone());
                    found = true;
                }
            }
            cycle.push(path.to_string());

            return Err(ResolverError::CircularDependency { cycle });
        }

        // Add to loading stack
        self.loading_stack.push(path.to_string());

        // In real implementation, would read file here
        // For Phase 2, we'll use a placeholder
        // let source = std::fs::read_to_string(path).map_err(|e| ...)?;

        // For now, create a placeholder for testing
        let source = format!("grove {} with end", Self::module_name_from_path(path));

        // Parse the source
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize_positioned();
        let mut parser = Parser::new(tokens);

        let ast = parser.parse().map_err(|e| ResolverError::ParseError {
            path: path.to_string(),
            error: format!("{:?}", e),
        })?;

        // Extract module information
        let info = Self::extract_module_info(path, &ast)?;

        // Remove from loading stack
        self.loading_stack.pop();

        // Add dependencies to graph
        self.dependency_graph.insert(path.to_string(), info.dependencies.clone());

        // Cache the module
        self.module_cache.insert(path.to_string(), info);

        Ok(&self.module_cache[path])
    }

    /// Extract module name from file path
    ///
    /// # Arguments
    /// * `path` - File path (e.g., "std/math.gw")
    ///
    /// # Returns
    /// Module name (e.g., "math")
    fn module_name_from_path(path: &str) -> String {
        // Extract filename without extension
        let filename = if let Some(pos) = path.rfind('/') {
            &path[pos + 1..]
        } else {
            path
        };

        // Remove .gw extension
        filename.trim_end_matches(".gw").to_string()
    }

    /// Extract module information from parsed AST
    ///
    /// # Arguments
    /// * `path` - Module file path
    /// * `ast` - Parsed AST
    fn extract_module_info(path: &str, ast: &[AstNode]) -> ResolverResult<ModuleInfo> {
        let mut name = Self::module_name_from_path(path);
        let mut dependencies = Vec::new();
        let mut exports = Vec::new();

        for node in ast {
            match node {
                // Extract module name from grove declaration
                AstNode::ModuleDecl { name: module_name, body: _, exports: module_exports, .. } => {
                    name = module_name.clone();
                    exports.extend(module_exports.clone());
                }

                // Extract dependencies from imports
                AstNode::Import { path: import_path, .. } => {
                    dependencies.push(import_path.clone());
                }

                // Extract exports
                AstNode::Export { items, .. } => {
                    exports.extend(items.clone());
                }

                _ => {}
            }
        }

        Ok(ModuleInfo {
            path: path.to_string(),
            name,
            ast: ast.to_vec(),
            dependencies,
            exports,
        })
    }

    /// Check for circular dependencies in the dependency graph
    ///
    /// Returns an error if a cycle is detected.
    pub fn check_circular_dependencies(&self) -> ResolverResult<()> {
        for module_path in self.dependency_graph.keys() {
            let mut visited = Vec::new();
            self.check_cycle_from(module_path, &mut visited)?
        }
        Ok(())
    }

    /// Check for circular dependencies starting from a module
    ///
    /// # Arguments
    /// * `current` - Current module path
    /// * `visited` - Stack of visited modules (for cycle detection)
    fn check_cycle_from(&self, current: &str, visited: &mut Vec<String>) -> ResolverResult<()> {
        // Check if we've seen this module before
        if let Some(pos) = visited.iter().position(|p| p == current) {
            // Circular dependency found - build cycle path
            let cycle = visited[pos..].to_vec();
            return Err(ResolverError::CircularDependency { cycle });
        }

        // Add to visited
        visited.push(current.to_string());

        // Check dependencies
        if let Some(deps) = self.dependency_graph.get(current) {
            for dep in deps {
                self.check_cycle_from(dep, visited)?;
            }
        }

        // Remove from visited (backtrack)
        visited.pop();

        Ok(())
    }

    /// Get a cached module
    ///
    /// # Arguments
    /// * `path` - Module path
    pub fn get_module(&self, path: &str) -> Option<&ModuleInfo> {
        self.module_cache.get(path)
    }

    /// Get all loaded modules
    pub fn loaded_modules(&self) -> impl Iterator<Item = (&String, &ModuleInfo)> {
        self.module_cache.iter()
    }

    /// Clear the module cache (for testing)
    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        self.module_cache.clear();
        self.dependency_graph.clear();
        self.loading_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_resolver() {
        let resolver = ModuleResolver::new(
            "/project".to_string(),
            "/usr/lib/glimmer-weave/std".to_string(),
        );

        assert_eq!(resolver.project_root, "/project");
        assert_eq!(resolver.stdlib_path, "/usr/lib/glimmer-weave/std");
        assert_eq!(resolver.module_cache.len(), 0);
    }

    #[test]
    fn test_resolve_stdlib_path() {
        let resolver = ModuleResolver::new(
            "/project".to_string(),
            "/usr/lib/glimmer-weave/std".to_string(),
        );

        let resolved = resolver.resolve_path("std/math.gw", None).unwrap();
        assert_eq!(resolved, "/usr/lib/glimmer-weave/std/std/math.gw");
    }

    #[test]
    fn test_resolve_project_path() {
        let resolver = ModuleResolver::new(
            "/project".to_string(),
            "/usr/lib/glimmer-weave/std".to_string(),
        );

        let resolved = resolver.resolve_path("lib/utils.gw", None).unwrap();
        assert_eq!(resolved, "/project/lib/utils.gw");
    }

    #[test]
    fn test_resolve_relative_path() {
        let resolver = ModuleResolver::new(
            "/project".to_string(),
            "/usr/lib/glimmer-weave/std".to_string(),
        );

        let resolved = resolver.resolve_path("./utils.gw", Some("/project/lib/main.gw")).unwrap();
        assert_eq!(resolved, "/project/lib/./utils.gw");
    }

    #[test]
    fn test_module_name_from_path() {
        assert_eq!(ModuleResolver::module_name_from_path("std/math.gw"), "math");
        assert_eq!(ModuleResolver::module_name_from_path("math.gw"), "math");
        assert_eq!(ModuleResolver::module_name_from_path("lib/utils/helpers.gw"), "helpers");
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut resolver = ModuleResolver::new(
            "/project".to_string(),
            "/usr/lib/glimmer-weave/std".to_string(),
        );

        // Manually create a circular dependency in the graph
        resolver.dependency_graph.insert("a.gw".to_string(), vec!["b.gw".to_string()]);
        resolver.dependency_graph.insert("b.gw".to_string(), vec!["c.gw".to_string()]);
        resolver.dependency_graph.insert("c.gw".to_string(), vec!["a.gw".to_string()]);

        let result = resolver.check_circular_dependencies();
        assert!(result.is_err());

        if let Err(ResolverError::CircularDependency { cycle }) = result {
            assert!(cycle.len() >= 3);
            // Cycle should contain a.gw, b.gw, c.gw
            assert!(cycle.contains(&"a.gw".to_string()));
        } else {
            panic!("Expected CircularDependency error");
        }
    }

    #[test]
    fn test_no_circular_dependency() {
        let mut resolver = ModuleResolver::new(
            "/project".to_string(),
            "/usr/lib/glimmer-weave/std".to_string(),
        );

        // Create a non-circular dependency graph
        resolver.dependency_graph.insert("a.gw".to_string(), vec!["b.gw".to_string()]);
        resolver.dependency_graph.insert("b.gw".to_string(), vec!["c.gw".to_string()]);
        resolver.dependency_graph.insert("c.gw".to_string(), vec![]);

        let result = resolver.check_circular_dependencies();
        assert!(result.is_ok());
    }
}
