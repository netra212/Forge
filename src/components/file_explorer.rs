// ============================================================
// FORGE — COMPONENT 03C-2: INTERACTIVE FILE EXPLORER
//
// This is the Leptos/WASM frontend implementation.
//
// Responsibilities:
// 1. Preserve the existing Explorer UI.
// 2. Open projects through the native Tauri backend.
// 3. Display the selected project's name and path.
// 4. Load directory entries on demand.
// 5. Expand and collapse folders.
// 6. Notify the Code Viewer when a file is selected.
// ============================================================

// Leptos components, signals, effects, and view macros.
use leptos::prelude::*;

// Run async tasks in the WebAssembly frontend.
use leptos::task::spawn_local;

// Convert data between Rust and JavaScript.
use serde::{Deserialize, Serialize, de::DeserializeOwned};

// JavaScript/WebAssembly interoperability.
use wasm_bindgen::prelude::*;

// ============================================================
// 1. FRONTEND FILE ENTRY MODEL
// ============================================================

/// One file or directory returned by the native backend.
///
/// This mirrors the native FileEntry data model, but uses
/// Deserialize because the frontend receives the data.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub relative_path: String,
    pub is_directory: bool,
}

// ============================================================
// 2. TAURI IPC BINDING
// ============================================================

// The original code used:
//
// window.__TAURI__.dialog.open(...)
//
// We now use:
//
// window.__TAURI__.core.invoke(...)
//
// This allows the native backend to register the selected
// project root before we request directory contents.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(
        catch,
        js_namespace = ["window", "__TAURI__", "core"],
        js_name = invoke
    )]
    async fn invoke_native(command: &str, arguments: JsValue) -> Result<JsValue, JsValue>;
}

// ============================================================
// 3. GENERIC TAURI COMMAND HELPER
// ============================================================

/// Invoke a native Tauri command and deserialize its result.
///
/// T may be Option<String>, Vec<FileEntry>, or String.
pub async fn invoke_command<T: DeserializeOwned>(
    command: &str,
    arguments: JsValue,
) -> Result<T, String> {
    // Execute the native command.
    let result = invoke_native(command, arguments)
        .await
        .map_err(|error| error.as_string().unwrap_or_else(|| format!("{error:?}")))?;

    // Convert JavaScript data into the expected Rust type.
    serde_wasm_bindgen::from_value(result).map_err(|error| error.to_string())
}

// ============================================================
// 4. SERIALIZABLE COMMAND ARGUMENTS
// ============================================================

/// Arguments for native commands that accept a relative path.
///
/// The Rust field relative_path becomes the JavaScript
/// property relativePath through rename_all.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RelativePathArgs<'a> {
    relative_path: &'a str,
}

/// Empty argument object for commands without parameters.
#[derive(Serialize)]
struct NoArgs {}

// ============================================================
// 5. HELPER FOR PATH-BASED COMMANDS
// ============================================================

/// Invoke a Tauri command with a project-relative path.
pub async fn invoke_with_path<T: DeserializeOwned>(
    command: &str,
    relative_path: &str,
) -> Result<T, String> {
    // Serialize our request arguments into a JsValue.
    let arguments = serde_wasm_bindgen::to_value(&RelativePathArgs { relative_path })
        .map_err(|error| error.to_string())?;

    // Reuse the generic IPC helper.
    invoke_command(command, arguments).await
}

// ============================================================
// 6. SELECT PROJECT USING THE NATIVE BACKEND
// ============================================================

/// Open the native folder picker.
///
/// Unlike the previous frontend-only implementation,
/// this command also updates native ProjectState.
async fn choose_project_folder() -> Result<Option<String>, String> {
    // open_project requires no frontend arguments.
    let arguments = serde_wasm_bindgen::to_value(&NoArgs {}).map_err(|error| error.to_string())?;

    // The native backend handles the folder picker,
    // canonicalizes the path, and stores the project root.
    invoke_command::<Option<String>>("open_project", arguments).await
}

// ============================================================
// 7. OPEN OR CHANGE THE SELECTED PROJECT
// ============================================================

/// Start the project picker from either Explorer button.
///
/// The selected_project signal is shared with App.
/// The selected_file signal is shared with CodeViewer.
/// project_revision invalidates older async results.
fn start_project_picker(
    selected_project: RwSignal<Option<String>>,
    selected_file: RwSignal<Option<String>>,
    project_revision: RwSignal<u64>,
    is_opening: RwSignal<bool>,
    error_message: RwSignal<Option<String>>,
) {
    // Prevent opening the folder picker multiple times.
    if is_opening.get_untracked() {
        return;
    }

    // Tell Leptos that the picker is currently active.
    is_opening.set(true);

    // Clear any previous error.
    error_message.set(None);

    // WebView-related futures run on the local executor.
    spawn_local(async move {
        // Wait for native project selection.
        match choose_project_folder().await {
            // User selected a valid project directory.
            Ok(Some(path)) => {
                // Increment the project revision to invalidate
                // directory and source requests from older projects.
                project_revision.update(|revision| {
                    *revision = revision.wrapping_add(1);
                });

                // Close the previously opened source file.
                selected_file.set(None);

                // Store the newly selected project path.
                // The status bar can continue reading this signal.
                selected_project.set(Some(path));
            }

            // Cancellation is normal, not an error.
            Ok(None) => {
                // Preserve the current project and selected file.
            }

            // Report real dialog/backend errors.
            Err(message) => {
                error_message.set(Some(message));
            }
        }

        // Re-enable both project-selection buttons.
        is_opening.set(false);
    });
}

// ============================================================
// 8. RECURSIVE FILE TREE NODE
// ============================================================

/// Display one file or directory in the Explorer.
///
/// Clicking a file selects it for CodeViewer.
/// Clicking a directory expands or collapses its children.
#[component]
fn FileTreeNode(
    entry: FileEntry,
    depth: usize,
    selected_file: RwSignal<Option<String>>,
    project_revision: RwSignal<u64>,
) -> impl IntoView {
    // Extract metadata from the FileEntry.
    let is_directory = entry.is_directory;
    let name = entry.name;
    let relative_path = entry.relative_path;

    // Different closures need owned copies of the path.
    let click_path = relative_path.clone();
    let selected_path = relative_path.clone();

    // Whether this directory is currently expanded.
    let expanded = RwSignal::new(false);

    // None means the directory hasn't been loaded yet.
    // Some(vec![]) means it was loaded but is empty.
    let children = RwSignal::new(None::<Vec<FileEntry>>);

    // Track asynchronous directory loading.
    let loading = RwSignal::new(false);

    // Store an error for this particular directory.
    let error = RwSignal::new(None::<String>);

    // Indent nested folders by an additional 16 pixels.
    let indentation = format!("{}px", 10 + depth * 16);

    view! {
        <div class="forge-tree-node">

            // ---------------------------------------------
            // FILE OR DIRECTORY ROW
            // ---------------------------------------------

            <button
                type="button"

                // Highlight the selected source file.
                class=move || {
                    if !is_directory
                        && selected_file.get().as_deref()
                            == Some(selected_path.as_str())
                    {
                        "forge-tree-row forge-tree-selected"
                    } else {
                        "forge-tree-row"
                    }
                }

                // Indent the row according to directory depth.
                style=format!("padding-left: {indentation};")

                // Make folder expansion state available to
                // assistive technologies.
                aria-expanded=move || {
                    if is_directory {
                        Some(expanded.get().to_string())
                    } else {
                        None
                    }
                }

                on:click=move |_| {
                    // A FILE WAS CLICKED:
                    // Tell the Code Viewer which file to open.
                    if !is_directory {
                        selected_file.set(Some(click_path.clone()));
                        return;
                    }

                    // AN EXPANDED FOLDER WAS CLICKED:
                    // Collapse it without discarding its data.
                    if expanded.get_untracked() {
                        expanded.set(false);
                        return;
                    }

                    // A COLLAPSED FOLDER WAS CLICKED:
                    // Expand it.
                    expanded.set(true);

                    // Do not reload children that we already have.
                    // Also avoid duplicate requests while loading.
                    if children.get_untracked().is_some()
                        || loading.get_untracked()
                    {
                        return;
                    }

                    // Show a loading state for this directory.
                    loading.set(true);
                    error.set(None);

                    // Own the folder path for the async closure.
                    let folder = click_path.clone();

                    // Remember which project initiated this request.
                    let expected_revision =
                        project_revision.get_untracked();

                    // Ask the native backend for this directory's
                    // immediate children.
                    spawn_local(async move {
                        let result =
                            invoke_with_path::<Vec<FileEntry>>(
                                "list_project_directory",
                                &folder,
                            )
                            .await;

                        // Ignore results from an older project.
                        if project_revision.get_untracked()
                            != expected_revision
                        {
                            return;
                        }

                        // Update this folder's reactive state.
                        match result {
                            Ok(entries) => {
                                children.set(Some(entries));
                            }

                            Err(message) => {
                                error.set(Some(message));
                            }
                        }

                        // The current request is finished.
                        loading.set(false);
                    });
                }
            >
                // Chevron for expanded/collapsed directories.
                <span
                    class="forge-tree-chevron"
                    aria-hidden="true"
                >
                    {move || {
                        if !is_directory {
                            " "
                        } else if expanded.get() {
                            "▾"
                        } else {
                            "▸"
                        }
                    }}
                </span>

                // Folder or file symbol.
                <span
                    class="forge-tree-symbol"
                    aria-hidden="true"
                >
                    {if is_directory { "📁" } else { "📄" }}
                </span>

                // Display the actual filename.
                <span class="forge-tree-label">
                    {name}
                </span>
            </button>

            // ---------------------------------------------
            // NESTED DIRECTORY CONTENT
            // ---------------------------------------------

            <Show when=move || is_directory && expanded.get()>

                // Display a loading message.
                <Show when=move || loading.get()>
                    <p class="forge-tree-hint">
                        "Loading..."
                    </p>
                </Show>

                // Display an I/O or authorization error.
                <Show when=move || error.get().is_some()>
                    <p class="forge-tree-error" role="alert">
                        {move || error.get().unwrap_or_default()}
                    </p>
                </Show>

                // Display an empty-directory message.
                <Show when=move || {
                    !loading.get()
                        && children
                            .get()
                            .as_ref()
                            .is_some_and(Vec::is_empty)
                }>
                    <p class="forge-tree-hint">
                        "Empty folder"
                    </p>
                </Show>

                // Render children recursively using FileTreeNode.
                <For
                    each=move || {
                        children.get().unwrap_or_default()
                    }

                    // Stable identity for each child.
                    key=|child| child.relative_path.clone()

                    children=move |child| {
                        view! {
                            <FileTreeNode
                                entry=child
                                depth=depth + 1
                                selected_file=selected_file
                                project_revision=project_revision
                            />
                        }
                        .into_any()
                    }
                />

            </Show>
        </div>
    }
}

// ============================================================
// 9. MAIN FILE EXPLORER COMPONENT
// ============================================================

/// Display the Explorer sidebar.
///
/// App owns the shared project/file signals and passes
/// them to this component.
#[component]
pub fn FileExplorer(
    selected_project: RwSignal<Option<String>>,
    selected_file: RwSignal<Option<String>>,
    project_revision: RwSignal<u64>,
) -> impl IntoView {
    // Whether the native project picker is running.
    let is_opening = RwSignal::new(false);

    // Error when opening or changing a project.
    let error_message = RwSignal::new(None::<String>);

    // Top-level directory entries.
    let root_entries = RwSignal::new(None::<Vec<FileEntry>>);

    // State for loading the project's root directory.
    let root_loading = RwSignal::new(false);
    let root_error = RwSignal::new(None::<String>);

    // Reload the root directory whenever the project changes.
    Effect::new(move |_| {
        // Track both the selected project and its revision.
        let revision = project_revision.get();
        let has_project = selected_project.get().is_some();

        // Clear entries and errors from an older project.
        root_entries.set(None);
        root_error.set(None);
        root_loading.set(has_project);

        // No project is selected: nothing to load.
        if !has_project {
            return;
        }

        // Load the selected project's top-level entries.
        spawn_local(async move {
            // An empty relative path means project root.
            let result = invoke_with_path::<Vec<FileEntry>>("list_project_directory", "").await;

            // Discard results from an older project.
            if project_revision.get_untracked() != revision {
                return;
            }

            match result {
                Ok(entries) => {
                    root_entries.set(Some(entries));
                }

                Err(message) => {
                    root_error.set(Some(message));
                }
            }

            root_loading.set(false);
        });
    });

    // Preserve the original Explorer interface.
    view! {
        <aside class="forge-explorer">

            // ---------------------------------------------
            // EXPLORER HEADING
            // ---------------------------------------------
            <div class="forge-explorer-heading">
                "EXPLORER"
            </div>

            // ---------------------------------------------
            // NO PROJECT SELECTED
            // ---------------------------------------------
            <Show when=move || selected_project.get().is_none()>
                <div class="forge-explorer-empty">

                    // Original folder SVG: preserved.
                    <svg
                        class="forge-folder-icon"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        aria-hidden="true"
                    >
                        <path d="M3 7a2 2 0 0 1 2-2h5l2 3h7a2 2 0 0 1 2 2v9H3z"/>
                    </svg>

                    <div class="forge-explorer-empty-title">
                        "No Folder Opened"
                    </div>

                    // Original empty-state description.
                    <p class="forge-muted">
                        "Open a Rust project to get started."
                    </p>

                    // Original Open Project button.
                    <button
                        type="button"
                        class="forge-open-project-button"

                        disabled=move || is_opening.get()

                        on:click=move |_| {
                            start_project_picker(
                                selected_project,
                                selected_file,
                                project_revision,
                                is_opening,
                                error_message,
                            );
                        }
                    >
                        {move || {
                            if is_opening.get() {
                                "Opening..."
                            } else {
                                "Open Project"
                            }
                        }}
                    </button>
                </div>
            </Show>

            // ---------------------------------------------
            // PROJECT SELECTED
            // ---------------------------------------------
            <Show when=move || selected_project.get().is_some()>
                <div class="forge-project-summary">

                    // Original selected project name.
                    <div class="forge-project-name">
                        {move || {
                            selected_project
                                .get()
                                .map(|path| {
                                    std::path::Path::new(&path)
                                        .file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or(&path)
                                        .to_string()
                                })
                                .unwrap_or_default()
                        }}
                    </div>

                    // Original full project path: preserved.
                    <p class="forge-project-path">
                        {move || {
                            selected_project
                                .get()
                                .unwrap_or_default()
                        }}
                    </p>

                    // Original Change Project button.
                    <button
                        type="button"
                        class="forge-open-project-button forge-change-project-button"

                        disabled=move || is_opening.get()

                        on:click=move |_| {
                            start_project_picker(
                                selected_project,
                                selected_file,
                                project_revision,
                                is_opening,
                                error_message,
                            );
                        }
                    >
                        {move || {
                            if is_opening.get() {
                                "Opening..."
                            } else {
                                "Change Project"
                            }
                        }}
                    </button>
                </div>

                // -----------------------------------------
                // REAL PROJECT FILE TREE — NEW
                // -----------------------------------------
                <div
                    class="forge-tree-scroll"
                    aria-label="Project files"
                >
                    // Root directory is loading.
                    <Show when=move || root_loading.get()>
                        <p class="forge-tree-hint">
                            "Loading project files..."
                        </p>
                    </Show>

                    // Error while reading the project root.
                    <Show when=move || root_error.get().is_some()>
                        <p class="forge-tree-error" role="alert">
                            {move || root_error.get().unwrap_or_default()}
                        </p>
                    </Show>

                    // The selected project contains no entries.
                    <Show when=move || {
                        !root_loading.get()
                            && root_entries
                                .get()
                                .as_ref()
                                .is_some_and(Vec::is_empty)
                    }>
                        <p class="forge-tree-hint">
                            "This project folder is empty."
                        </p>
                    </Show>

                    // Render real top-level files and directories.
                    <For
                        each=move || {
                            root_entries.get().unwrap_or_default()
                        }

                        key=|entry| entry.relative_path.clone()

                        children=move |entry| {
                            view! {
                                <FileTreeNode
                                    entry=entry
                                    depth=0
                                    selected_file=selected_file
                                    project_revision=project_revision
                                />
                            }
                        }
                    />
                </div>
            </Show>

            // ---------------------------------------------
            // ERROR FEEDBACK — ORIGINAL BEHAVIOR
            // ---------------------------------------------
            <Show when=move || error_message.get().is_some()>
                <p class="forge-explorer-error" role="alert">
                    {move || error_message.get().unwrap_or_default()}
                </p>
            </Show>

        </aside>
    }
}
