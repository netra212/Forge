use leptos::prelude::*;

// Import our extracted Activity Bar and navigation enum.
// Both are defined in src/components/activity_bar.rs.
use crate::components::activity_bar::{ActivePanel, ActivityBar};
use crate::components::file_explorer::FileExplorer;

// ============================================================
// COMPONENT 01 — FORGE APPLICATION SHELL
//
// Responsibility:
// - Own the current navigation selection
// - Assemble the application's main regions
// - Pass state to child components
// - Display temporary placeholder content
//
// Component 02 (ActivityBar) now lives in its own file.
// ============================================================

// Keep the working styles local during this refactor.
// We can migrate them to styles.css in a separate step.
const APP_STYLES: &str = r#"

    /* Reset default browser spacing. */
    html, body {
        margin: 0 !important;
        padding: 0 !important;
        width: 100%;
        height: 100%;
        overflow: hidden;
    }

    /* Apply Forge's dark desktop appearance. */
    body {
        display: block !important;
        background: #1b1b1b;
        color: #cccccc;
        font-family: system-ui, sans-serif;
    }

    /* Include padding and borders in dimensions. */
    * {
        box-sizing: border-box;
    }

    /* ROOT: three application rows. */
    .forge-shell {
        position: fixed;
        inset: 0;
        display: grid;

        /* Header, flexible workspace, bottom status bar. */
        grid-template-rows: 44px minmax(0, 1fr) 26px;

        width: 100%;
        height: 100%;
        overflow: hidden;
        background: #1b1b1b;
    }

    /* Application header. */
    .forge-titlebar {
        display: flex;
        align-items: center;
        justify-content: center;
        background: #252525;
        border-bottom: 1px solid #353535;
        font-size: 13px;
    }

    /* Activity bar, sidebar and workspace columns. */
    .forge-body {
        display: grid;
        grid-template-columns: 52px 240px minmax(0, 1fr);
        min-width: 0;
        min-height: 0;
        overflow: hidden;
    }

    /* COMPONENT 02: activity bar layout. */
    .forge-activity {
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        align-items: center;
        padding: 8px 0;
        background: #202020;
        border-right: 1px solid #333333;
    }

    /* Stack the top navigation buttons. */
    .forge-activity-top {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 4px;
        width: 100%;
    }

    /* Shared navigation button styling. */
    .forge-activity-button {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
        height: 46px;
        min-height: 46px;

        padding: 0;
        border: none;
        border-left: 2px solid transparent;
        border-radius: 0;
        background: transparent;

        color: #858585;
        cursor: pointer;

        transition: color 120ms ease,
                    background 120ms ease;
    }

    /* Show hover feedback. */
    .forge-activity-button:hover {
        color: #eeeeee;
        background: #2b2b2b;
    }

    /* Highlight the selected navigation icon. */
    .forge-activity-button.active {
        color: #f0f0f0;
        border-left-color: #dedede;
        background: #292929;
    }

    /* Show keyboard focus. */
    .forge-activity-button:focus-visible {
        outline: 2px solid #78aaff;
        outline-offset: -3px;
    }

    /* Give every navigation SVG the same size. */
    .forge-activity-button svg {
        width: 22px;
        height: 22px;
        flex-shrink: 0;
    }

    /* Selected panel's sidebar. */
    .forge-explorer {
        padding: 16px;
        background: #252525;
        border-right: 1px solid #353535;
        min-width: 0;
        min-height: 0;
        overflow: hidden;
    }

    /* Sidebar heading. */
    .forge-explorer-heading {
        font-size: 13px;
        font-weight: 500;
        color: #dddddd;
    }

    /* Main welcome workspace. */
    .forge-workspace {
        display: flex;
        align-items: center;
        justify-content: center;
        min-width: 0;
        min-height: 0;
        overflow: hidden;
        background: #1b1b1b;
    }

    /* Center the welcome information. */
    .forge-welcome {
        text-align: center;
        padding: 20px;
    }

    /* Forge application heading. */
    .forge-welcome h1 {
        margin: 0 0 12px;
        font-size: 36px;
        font-weight: 600;
        color: #dedede;
    }

    /* Secondary text styling. */
    .forge-muted {
        color: #898989;
        font-size: 13px;
    }

    /* Footer stays in the bottom grid row. */
    .forge-statusbar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 12px;
        min-height: 0;
        background: #252525;
        border-top: 1px solid #353535;
        font-size: 11px;
        color: #aaaaaa;
    }


    /* ======================================================
    COMPONENT 03 — FILE EXPLORER
    ====================================================== */

    /* Arrange Explorer content vertically. */
    .forge-explorer {
        display: flex;
        flex-direction: column;
    }

    /* Center the empty-project interface. */
    .forge-explorer-empty {
        display: flex;
        flex: 1;
        min-height: 0;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 12px;
        text-align: center;
    }

    /* Folder illustration. */
    .forge-folder-icon {
        width: 38px;
        height: 38px;
        color: #989898;
    }

    /* Heading shown when no folder has been selected. */
    .forge-explorer-empty-title {
        color: #dedede;
        font-size: 13px;
        font-weight: 600;
    }

    /* Avoid excessive margins around the description. */
    .forge-explorer-empty p {
        margin: 0;
        line-height: 1.5;
    }

    /* Shared style for Open Project and Change Project. */
    .forge-open-project-button {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
        min-height: 35px;
        padding: 8px 12px;

        border: 1px solid #477bbd;
        border-radius: 5px;
        background: #3267a6;

        color: #ffffff;
        font-family: inherit;
        font-size: 12px;
        font-weight: 500;

        cursor: pointer;
    }

    /* Provide visual feedback on hover. */
    .forge-open-project-button:hover {
        background: #3d79be;
    }

    /* Make keyboard navigation accessible. */
    .forge-open-project-button:focus-visible {
        outline: 2px solid #a1c5ff;
        outline-offset: 2px;
    }

    /* Prevent repeated interaction during selection. */
    .forge-open-project-button:disabled {
        opacity: 0.55;
        cursor: not-allowed;
    }

    /* Display the current project information. */
    .forge-project-summary {
        display: flex;
        flex-direction: column;
        align-items: stretch;
        gap: 12px;
        margin-top: 22px;
    }

    /* Highlight the selected project name. */
    .forge-project-name {
        color: #eeeeee;
        font-size: 13px;
        font-weight: 600;
        overflow-wrap: anywhere;
    }

    /* Show the selected local folder path. */
    .forge-project-path {
        margin: 0;
        color: #9b9b9b;
        font-size: 11px;
        line-height: 1.5;
        overflow-wrap: anywhere;
    }

    /* Display folder-selection errors. */
    .forge-explorer-error {
        padding: 10px;
        border: 1px solid #853f46;
        border-radius: 5px;
        background: #3a2529;
        color: #ffb3bc;
        font-size: 11px;
        overflow-wrap: anywhere;
    }


"#;


#[component]
pub fn App() -> impl IntoView {

    let active_panel = RwSignal::new(ActivePanel::Explorer);

    let selected_project = RwSignal::new(None::<String>);

    view! {

        <style>{APP_STYLES}</style>


        <div class="forge-shell">


            <header class="forge-titlebar">
                "FORGE - AI Developer Agent"
            </header>

            // Second row: application body.
            <div class="forge-body">

                // Render the separate ActivityBar component.
                //
                // Pass our shared RwSignal as a component prop.
                // ActivityBar will read and update this signal.
                <ActivityBar active_panel=active_panel />

                // Placeholder sidebar for the selected section.
                //
                // Component 03 will begin extracting the actual
                // File Explorer into its dedicated existing file.

                // Display the real FileExplorer only when
                // the Explorer navigation icon is selected.
                <Show
                    when=move || active_panel.get() == ActivePanel::Explorer

                    // Other activity-bar sections retain their placeholders.
                    fallback=move || view! {
                        <aside class="forge-explorer">

                            // Heading for the selected section.
                            <div class="forge-explorer-heading">
                                {move || active_panel.get().title()}
                            </div>

                            // Temporary text for Chat, Terminal, etc.
                            <p class="forge-muted">
                                {move || active_panel.get().description()}
                            </p>

                        </aside>
                    }
                >
                    // This component handles the Explorer interface.
                    //
                    // The selected folder signal is passed down
                    // so FileExplorer can update the parent-owned state.
                    <FileExplorer selected_project=selected_project />
                </Show>

                // The existing central workspace is unchanged.
                <main class="forge-workspace">
                    <div class="forge-welcome">

                        // Main application heading.
                        <h1>"Forge"</h1>

                        // Application subtitle.
                        <p class="forge-muted">
                            "AI Developer Agent"
                        </p>

                        // Empty workspace information.
                        <p class="forge-muted">
                            "Your workspace will appear here"
                        </p>

                    </div>
                </main>

            </div>

            // Third row: existing bottom status bar.
            <footer class="forge-statusbar">

                // Application version.
                <span>"Forge v0.1.0"</span>

                <span>
                // Reactively display the current project name.
                {move || {
                    // Read the selected folder, if one exists.
                    selected_project
                        .get()
                        .map(|path| {
                            // Extract the final directory name.
                            let name = std::path::Path::new(&path)
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or(&path);

                            // Build the status message.
                            format!("Project: {name}")
                        })
                        // Show the original message before selection.
                        .unwrap_or_else(|| "No project opened".to_string())
                    }}
                </span>

            </footer>

        </div>
    }
}
