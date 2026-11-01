// ==========================================================
// FORGE — NATIVE FILESYSTEM EXPLORER
//
// Location:
// src-tauri/src/fs/explorer.rs
//
// Responsibilities:
// 1. Manage the selected project root.
// 2. Validate filesystem paths.
// 3. List files and directories.
// 4. Read source-code files safely for preview.
// ==========================================================

use serde::Serialize;
use std::fs;
use std::io::{self, ErrorKind, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

// ----------------------------------------------------------
// 1. APPLICATION PROJECT STATE
// ----------------------------------------------------------

// The currently selected project is stored in native state.
//
// None means no project has been selected.
//
// Mutex allows concurrent Tauri commands to safely access
// and update the selected project path.
#[derive(Default)]
pub struct ProjectState {
    pub root: Mutex<Option<PathBuf>>,
}

// ----------------------------------------------------------
// 2. FILE ENTRY DATA MODEL
// ----------------------------------------------------------

// Represents one file or folder returned by the filesystem.
//
// Serialize lets Tauri send this information to Leptos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub relative_path: String,
    pub is_directory: bool,
}

// ----------------------------------------------------------
// 3. CANONICALIZE PROJECT ROOT
// ----------------------------------------------------------

// Resolve the selected project path and verify that it
// refers to an existing directory.
pub fn canonical_project_root(path: &Path) -> io::Result<PathBuf> {
    let canonical = fs::canonicalize(path)?;

    // A regular file cannot be selected as a project root.
    if !canonical.is_dir() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "the selected project root is not a directory",
        ));
    }

    Ok(canonical)
}

// ----------------------------------------------------------
// 4. VALIDATE RELATIVE PATH
// ----------------------------------------------------------

// Reject absolute paths and parent-directory traversal.
//
// Empty paths are allowed because they represent the
// project's root directory when listing files.
fn validate_relative_path(relative: &Path) -> io::Result<()> {
    if relative.as_os_str().is_empty() {
        return Ok(());
    }

    // Inspect the structural components of the path.
    for component in relative.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "only project-relative directory paths are allowed",
            ));
        }
    }

    Ok(())
}

// ----------------------------------------------------------
// 5. LIST PROJECT DIRECTORY
// ----------------------------------------------------------

// Read one directory level under an authorized project root.
//
// This is lazy directory enumeration, not recursive scanning.
pub fn list_directory(root: &Path, relative: &Path) -> io::Result<Vec<FileEntry>> {
    // Reject invalid relative path components.
    validate_relative_path(relative)?;

    // Resolve the authorized project root.
    let root = canonical_project_root(root)?;

    // Build the full path to the requested directory.
    let requested = root.join(relative);

    // Resolve the destination, including symbolic links.
    let resolved = fs::canonicalize(&requested)?;

    // Prevent access outside the selected project root.
    if !resolved.starts_with(&root) {
        return Err(io::Error::new(
            ErrorKind::PermissionDenied,
            "requested directory is outside the selected project",
        ));
    }

    // Ensure the destination is a directory.
    if !resolved.is_dir() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "requested path is not a directory",
        ));
    }

    // Collect the directory's immediate children.
    let mut entries = Vec::new();

    for child in fs::read_dir(&resolved)? {
        // Each iterator item can independently fail.
        let child = child?;

        // Identify the filesystem entry type.
        let kind = child.file_type()?;

        // Skip symbolic links in this first implementation.
        if kind.is_symlink() {
            continue;
        }

        // Show only regular files and directories.
        if !kind.is_file() && !kind.is_dir() {
            continue;
        }

        // Convert the filename into UTF-8 text.
        // Skip names that cannot be represented as a String.
        let Ok(name) = child.file_name().into_string() else {
            continue;
        };

        // Create a path relative to the project root.
        let entry_relative = relative.join(&name);

        // Use forward slashes for the frontend's path model.
        let relative_path = entry_relative
            .to_str()
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "invalid filename"))?
            .replace(std::path::MAIN_SEPARATOR, "/");

        // Construct our own application data model.
        entries.push(FileEntry {
            name,
            relative_path,
            is_directory: kind.is_dir(),
        });
    }

    // Directories first, followed by filenames in order.
    entries.sort_by(|left, right| {
        right
            .is_directory
            .cmp(&left.is_directory)
            .then_with(|| left.name.cmp(&right.name))
    });

    Ok(entries)
}

// ----------------------------------------------------------
// 6. READ-ONLY SOURCE FILE READER
// ----------------------------------------------------------

// Limit a preview to one mebibyte (1 MiB).
const MAX_PREVIEW_BYTES: u64 = 1024 * 1024;

// Read one UTF-8 source-code file inside the project.
//
// Unlike list_directory(), this returns the file contents
// as String, rather than returning filesystem metadata.
pub fn read_project_file(root: &Path, relative: &Path) -> io::Result<String> {
    // Check that the file path has valid relative components.
    validate_relative_path(relative)?;

    // An empty path represents the project root, not a file.
    if relative.as_os_str().is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "choose a file to preview",
        ));
    }

    // Canonicalize the selected project root.
    let root = canonical_project_root(root)?;

    // Construct the requested file path.
    let requested = root.join(relative);

    // Reject a symbolic link as the final requested file.
    if fs::symlink_metadata(&requested)?.file_type().is_symlink() {
        return Err(io::Error::new(
            ErrorKind::PermissionDenied,
            "symbolic links cannot be previewed",
        ));
    }

    // Resolve the actual destination.
    let resolved = fs::canonicalize(&requested)?;

    // Reject files outside the current project root.
    if !resolved.starts_with(&root) {
        return Err(io::Error::new(
            ErrorKind::PermissionDenied,
            "file is outside the selected project",
        ));
    }

    // The Code Viewer expects a regular file.
    if !resolved.is_file() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "requested path is not a regular file",
        ));
    }

    // Open the file for reading.
    let file = fs::File::open(&resolved)?;

    // Reject oversized files before reading their contents.
    if file.metadata()?.len() > MAX_PREVIEW_BYTES {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "file is too large to preview",
        ));
    }

    // Store the raw bytes in a vector.
    let mut bytes = Vec::new();

    // Read at most one extra byte to detect size-limit overflow.
    file.take(MAX_PREVIEW_BYTES + 1).read_to_end(&mut bytes)?;

    // Check the actual number of bytes read.
    if bytes.len() as u64 > MAX_PREVIEW_BYTES {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "file is too large to preview",
        ));
    }

    // Treat NUL-containing data as binary for this viewer.
    if bytes.contains(&0) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "binary files cannot be previewed",
        ));
    }

    // Convert the bytes to valid UTF-8 text.
    String::from_utf8(bytes)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "file is not valid UTF-8 text"))
}
