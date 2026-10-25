
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Serialize;
use wasm_bindgen::prelude::*;

// ============================================================
// COMPONENT 03 — FILE EXPLORER
//
// Responsibilities:
// - Show the empty Explorer interface.
// - Open the macOS folder selection dialog.
// - Store the selected project path in shared state.
// - Show the selected project's name and location.
// - Display errors without crashing the application.
//
// Reading project files will be a separate step.
// ============================================================


// Define the options passed to Tauri's dialog API.
//
// Serialize converts this Rust struct into the JavaScript
// object expected by the Tauri dialog plugin.
#[derive(Serialize)]
struct FolderDialogOptions {
    // Only allow selecting directories, not individual files.
    directory: bool,

    // Only allow one project folder at a time.
    multiple: bool,
}


// Declare a function provided by Tauri's JavaScript API.
//
// This calls:
// window.__TAURI__.dialog.open(options)
//
// "catch" converts JavaScript exceptions or rejected
// Promises into a Rust Result instead of silently failing.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(
        catch,
        js_namespace = ["window", "__TAURI__", "dialog"],
        js_name = open
    )]
    async fn open_directory_dialog(
        options: JsValue,
    ) -> Result<JsValue, JsValue>;
}


// Open the native folder picker.
//
// Result<Option<String>, String> has three outcomes:
//
// Ok(Some(path)) = user selected a directory.
// Ok(None)       = user cancelled the dialog.
// Err(message)   = the dialog failed.
async fn choose_project_folder() -> Result<Option<String>, String> {
    // Define the native dialog options.
    let options = FolderDialogOptions {
        directory: true,
        multiple: false,
    };

    // Convert the Rust options into a JavaScript value.
    let args = serde_wasm_bindgen::to_value(&options)
        .map_err(|error| error.to_string())?;

    // Ask Tauri to open the native folder picker.
    let result = open_directory_dialog(args)
        .await
        .map_err(|error| {
            error
                .as_string()
                .unwrap_or_else(|| format!("{error:?}"))
        })?;

    // Closing the picker without selecting anything
    // normally returns JavaScript null.
    if result.is_null() || result.is_undefined() {
        return Ok(None);
    }

    // Convert the selected JavaScript path to Rust String.
    let path = result
        .as_string()
        .ok_or("The dialog did not return a folder path.")?;

    // Return the successfully selected directory.
    Ok(Some(path))
}


// Start a folder-selection operation.
//
// This helper is shared by the initial Open Project button
// and the Change Project button.
fn start_project_picker(
    selected_project: RwSignal<Option<String>>,
    is_opening: RwSignal<bool>,
    error_message: RwSignal<Option<String>>,
) {
    // Do not start another picker while one is already open.
    if is_opening.get_untracked() {
        return;
    }

    // Tell the interface that an operation is in progress.
    is_opening.set(true);

    // Clear any error from a previous attempt.
    error_message.set(None);

    // Run the asynchronous operation on the WASM UI thread.
    //
    // spawn_local does not require a Send future,
    // which is appropriate for browser/WebView APIs.
    spawn_local(async move {
        // Wait for the user to select or cancel a folder.
        match choose_project_folder().await {
            // A directory was selected successfully.
            Ok(Some(path)) => {
                // Store it in the signal owned by App.
                selected_project.set(Some(path));
            }

            // The user pressed Cancel.
            Ok(None) => {
                // Keep the previously selected project unchanged.
            }

            // An actual error occurred.
            Err(message) => {
                // Display the error inside the Explorer.
                error_message.set(Some(message));
            }
        }

        // Enable the button again after the operation finishes.
        is_opening.set(false);
    });
}


// Register FileExplorer as a reusable Leptos component.
#[component]
pub fn FileExplorer(
    // This signal is owned by App and passed down as a prop.
    //
    // The parent can read the selected path later when
    // displaying the file tree or updating the status bar.
    selected_project: RwSignal<Option<String>>,
) -> impl IntoView {
    // Store whether the native picker is currently running.
    let is_opening = RwSignal::new(false);

    // Store a possible error message for the interface.
    let error_message = RwSignal::new(None::<String>);

    // Build the Explorer interface.
    view! {
        // Use the sidebar class from Component 01.
        <aside class="forge-explorer">

            // Explorer heading at the top.
            <div class="forge-explorer-heading">
                "EXPLORER"
            </div>

            // ---------------------------------------------
            // EMPTY STATE
            // ---------------------------------------------

            // Show this section when no project is selected.
            <Show when=move || selected_project.get().is_none()>
                <div class="forge-explorer-empty">

                    // Decorative folder icon.
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

                    // Explain why the Explorer is empty.
                    <div class="forge-explorer-empty-title">
                        "No Folder Opened"
                    </div>

                    // Short instruction for the user.
                    <p class="forge-muted">
                        "Open a Rust project to get started."
                    </p>

                    // Open the native macOS folder picker.
                    <button
                        type="button"
                        class="forge-open-project-button"

                        // Disable repeated clicks while opening.
                        disabled=move || is_opening.get()

                        // Start the asynchronous picker.
                        on:click=move |_| {
                            start_project_picker(
                                selected_project,
                                is_opening,
                                error_message,
                            );
                        }
                    >
                        // Change the label while the picker is active.
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
            // SELECTED PROJECT STATE
            // ---------------------------------------------

            // Show this section after selecting a folder.
            <Show when=move || selected_project.get().is_some()>
                <div class="forge-project-summary">

                    // Display the folder name instead of the full path.
                    <div class="forge-project-name">
                        {move || {
                            selected_project
                                .get()
                                .map(|path| {
                                    // Extract the last folder name.
                                    std::path::Path::new(&path)
                                        .file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or(&path)
                                        .to_string()
                                })
                                .unwrap_or_default()
                        }}
                    </div>

                    // Display the actual local directory path.
                    <p class="forge-project-path">
                        {move || {
                            selected_project.get().unwrap_or_default()
                        }}
                    </p>

                    // Explain the current milestone.
                    <p class="forge-muted">
                        "Project selected. File tree coming next."
                    </p>

                    // Allow the user to choose another project.
                    <button
                        type="button"
                        class="forge-open-project-button"
                        disabled=move || is_opening.get()
                        on:click=move |_| {
                            start_project_picker(
                                selected_project,
                                is_opening,
                                error_message,
                            );
                        }
                    >
                        "Change Project"
                    </button>

                </div>
            </Show>

            // ---------------------------------------------
            // ERROR FEEDBACK
            // ---------------------------------------------

            // Only display errors when one exists.
            <Show when=move || error_message.get().is_some()>

                // role="alert" helps assistive technologies
                // announce errors to the user.
                <p class="forge-explorer-error" role="alert">
                    {move || error_message.get().unwrap_or_default()}
                </p>

            </Show>

        </aside>
    }
}
