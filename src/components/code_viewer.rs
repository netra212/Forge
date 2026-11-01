//
// COMPONENT 04 — READ-ONLY CODE VIEWER
//

// Leptos reactivity and components.
use leptos::prelude::*;

// Execute async operations in Leptos CSR.
use leptos::task::spawn_local;

// Reuse the Tauri IPC helper from FileExplorer.
use super::file_explorer::invoke_with_path;

// ------------------------------------------------------------
// CODE VIEWER COMPONENT
// ------------------------------------------------------------

#[component]
pub fn CodeViewer(
    selected_file: RwSignal<Option<String>>,
    project_revision: RwSignal<u64>,
) -> impl IntoView {
    // Stores the actual source code returned by Tauri.
    let source = RwSignal::new(None::<String>);

    // Tracks whether a source request is running.
    let loading = RwSignal::new(false);

    // Stores errors returned by the backend.
    let error = RwSignal::new(None::<String>);

    // This effect runs whenever the selected file
    // or selected project revision changes.
    Effect::new(move |_| {
        // Remember the current project revision.
        let revision = project_revision.get();

        // Get the file selected in the Explorer.
        let requested_file = selected_file.get();

        // Clear source text and errors from previous files.
        source.set(None);
        error.set(None);

        // Show loading only when a file was selected.
        loading.set(requested_file.is_some());

        // If no file is selected, stop here.
        let Some(path) = requested_file else {
            return;
        };

        // Call the native backend asynchronously.
        spawn_local(async move {
            // Read the source text of this file.
            let result = invoke_with_path::<String>("read_project_source", &path).await;

            // Ignore responses from a previous project
            // or from a file that is no longer selected.
            if project_revision.get_untracked() != revision
                || selected_file.get_untracked().as_deref() != Some(path.as_str())
            {
                return;
            }

            // Store the returned source text or error.
            match result {
                Ok(text) => {
                    source.set(Some(text));
                }

                Err(message) => {
                    error.set(Some(message));
                }
            }

            // Source loading is finished.
            loading.set(false);
        });
    });

    // Render the main workspace.
    view! {
        <main class="forge-workspace">

            // Show editor only when a file is selected.
            <Show
                when=move || selected_file.get().is_some()

                // Otherwise display the existing welcome UI.
                fallback=|| view! {
                    <div class="forge-welcome">

                        <h1>"Forge"</h1>

                        <p class="forge-muted">
                            "AI Developer Agent"
                        </p>

                        <p class="forge-muted">
                            "Select a file in Explorer to preview its source."
                        </p>

                    </div>
                }
            >

                <div class="forge-editor">

                    // -----------------------------------
                    // FILE TAB
                    // -----------------------------------

                    <div class="forge-editor-tab">

                        <span class="forge-editor-tab-icon">
                            "◇"
                        </span>

                        // Extract only the filename for the tab.
                        <span>
                            {move || {
                                selected_file
                                    .get()
                                    .as_deref()
                                    .and_then(|path| {
                                        path.rsplit('/').next()
                                    })
                                    .unwrap_or_default()
                                    .to_string()
                            }}
                        </span>

                        // Close the currently selected file.
                        <button
                            type="button"
                            class="forge-editor-close"
                            aria-label="Close file"
                            on:click=move |_| {
                                selected_file.set(None);
                            }
                        >
                            "×"
                        </button>

                    </div>

                    // -----------------------------------
                    // FILE PATH / STATUS
                    // -----------------------------------

                    <div class="forge-editor-path">

                        {move || {
                            selected_file.get().unwrap_or_default()
                        }}

                        <span class="forge-readonly-label">
                            "Read-only"
                        </span>

                    </div>

                    // -----------------------------------
                    // SOURCE CODE CONTENT
                    // -----------------------------------

                    <div class="forge-editor-scroll">

                        // Display loading feedback.
                        <Show when=move || loading.get()>
                            <p class="forge-viewer-notice">
                                "Loading source..."
                            </p>
                        </Show>

                        // Display backend errors.
                        <Show when=move || error.get().is_some()>
                            <p
                                class="forge-viewer-error"
                                role="alert"
                            >
                                {move || {
                                    error.get().unwrap_or_default()
                                }}
                            </p>
                        </Show>

                        // Render source lines when available.
                        <Show when=move || source.get().is_some()>

                            <div class="forge-source-lines">

                                <For

                                    // Split source text into lines
                                    // and attach line numbers.
                                    each=move || {
                                        source
                                            .get()
                                            .unwrap_or_default()
                                            .lines()
                                            .enumerate()
                                            .map(|(number, line)| {
                                                (number, line.to_string())
                                            })
                                            .collect::<Vec<_>>()
                                    }

                                    // Use line number as the key
                                    // within this preview.
                                    key=|(number, _)| *number

                                    // Render each numbered line.
                                    children=|(number, line)| {
                                        view! {
                                            <div class="forge-code-line">

                                                <span class="forge-line-number">
                                                    {number + 1}
                                                </span>

                                                <span class="forge-line-text">
                                                    {
                                                        if line.is_empty() {
                                                            " ".to_string()
                                                        } else {
                                                            line
                                                        }
                                                    }
                                                </span>

                                            </div>
                                        }
                                    }
                                />

                            </div>

                        </Show>

                    </div>

                </div>

            </Show>

        </main>
    }
}
