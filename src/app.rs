use leptos::prelude::*;

// Import our extracted Activity Bar and navigation enum.
// Both are defined in src/components/activity_bar.rs.
use crate::components::activity_bar::{ActivePanel, ActivityBar};
use crate::components::code_viewer::CodeViewer;
use crate::components::file_explorer::FileExplorer;

// ============================================================
// COMPONENT 01 — FORGE APPLICATION SHELL
//
// Responsibility:
// - Own the current navigation selection
// - Assemble the application's main regions
// - Pass state to child components
// - Render the interactive File Explorer and Code Viewer
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
    
    /* ========================================================
    COMPONENT 03C-2 — FILE TREE
    ======================================================== */
    /* Sidebar layout and spacing. */
    .forge-explorer {
        padding: 12px 0 0;
    }
    .forge-explorer-heading {
        padding: 0 14px 12px;
    }
    .forge-explorer-empty {
        padding: 18px;
    }
    /* Selected project summary. */
    .forge-project-summary {
        margin: 0;
        padding: 6px 12px 10px;
        gap: 8px;
    }
    .forge-project-name {
        text-transform: uppercase;
        font-size: 12px;
    }
    .forge-change-project-button {
        min-height: 28px;
        padding: 4px 8px;
        font-size: 11px;
    }
    /* Scrollable file-tree area. */
    .forge-tree-scroll {
        flex: 1;
        min-height: 0;
        overflow: auto;
        padding: 4px 0 12px;
    }
    .forge-tree-node {
        min-width: 0;
    }
    /* Each file/folder occupies one clickable row. */
    .forge-tree-row {
        display: flex;
        align-items: center;
        gap: 5px;
        width: 100%;
        min-width: 0;
        min-height: 25px;
        padding: 3px 8px;
        border: 0;
        border-radius: 0;
        background: transparent;
        color: #cdcdcd;
        text-align: left;
        font: 12px system-ui, sans-serif;
        cursor: pointer;
    }
    /* Hover and selected-file feedback. */
    .forge-tree-row:hover {
        background: #333333;
    }
    .forge-tree-row.forge-tree-selected {
        background: #373e47;
        color: #ffffff;
    }
    .forge-tree-row:focus-visible {
        outline: 1px solid #77aaff;
        outline-offset: -1px;
    }
    /* Chevron, icon and filename layout. */
    .forge-tree-chevron {
        flex: 0 0 10px;
        font-size: 11px;
        color: #b0b0b0;
    }
    .forge-tree-symbol {
        flex: 0 0 18px;
        font-size: 13px;
    }
    .forge-tree-label {
        overflow: hidden;
        white-space: nowrap;
        text-overflow: ellipsis;
    }
    /* Loading, empty-folder and error feedback. */
    .forge-tree-hint {
        padding: 5px 18px;
        color: #929292;
        font-size: 11px;
    }
    .forge-tree-error,
    .forge-viewer-error {
        margin: 8px 12px;
        color: #ffb3bc;
        font-size: 12px;
        overflow-wrap: anywhere;
    }
    /* ========================================================
    COMPONENT 04 — READ-ONLY CODE VIEWER
    ======================================================== */
    /* Fill the central workspace vertically. */
    .forge-editor {
        display: flex;
        flex-direction: column;
        width: 100%;
        height: 100%;
        min-width: 0;
    }
    /* File tab at the top of the viewer. */
    .forge-editor-tab {
        display: flex;
        align-items: center;
        gap: 9px;
        width: max-content;
        max-width: 100%;
        min-height: 36px;
        padding: 0 12px;
        background: #222222;
        border-right: 1px solid #353535;
        border-bottom: 1px solid #202020;
        color: #e2e2e2;
        font-size: 12px;
    }
    .forge-editor-tab-icon {
        color: #df965e;
    }
    /* Close-file button. */
    .forge-editor-close {
        border: none;
        background: transparent;
        color: #aaaaaa;
        font-size: 18px;
        padding: 0 3px;
        margin-left: 14px;
        cursor: pointer;
    }
    .forge-editor-close:hover {
        color: #ffffff;
    }
    /* Show the project-relative source path. */
    .forge-editor-path {
        display: flex;
        justify-content: space-between;
        gap: 12px;
        padding: 8px 14px;
        background: #1b1b1b;
        color: #8e8e8e;
        border-bottom: 1px solid #2c2c2c;
        font-size: 11px;
        overflow-wrap: anywhere;
    }
    .forge-readonly-label {
        white-space: nowrap;
    }
    /* Both vertical and horizontal source scrolling. */
    .forge-editor-scroll {
        overflow: auto;
        min-height: 0;
        flex: 1;
    }
    /* Loading message. */
    .forge-viewer-notice {
        padding: 18px;
        color: #999999;
        font-size: 12px;
    }
    /* Use a monospaced font for source code. */
    .forge-source-lines {
        width: max-content;
        min-width: 100%;
        padding: 12px 0 24px;
        font: 13px/1.65 ui-monospace, SFMono-Regular,
            Menlo, Monaco, Consolas, monospace;
    }
    /* Individual source-code lines. */
    .forge-code-line {
        display: flex;
        white-space: pre;
        min-height: 21px;
    }
    .forge-code-line:hover {
        background: #242424;
    }
    /* Line numbers. */
    .forge-line-number {
        flex: 0 0 60px;
        padding-right: 17px;
        color: #717171;
        text-align: right;
        user-select: none;
    }
    /* File contents. */
    .forge-line-text {
        flex: 1;
        padding-right: 25px;
        color: #d3d3d3;
        white-space: pre;
    }
"#;

// NOTE:
// Keep the complete APP_STYLES constant from the corrected
// source file linked above. It includes all your existing
// shell, Explorer, tree, and editor styles.

// ============================================================
// FORGE — MAIN APPLICATION COMPONENT
// ============================================================

#[component]
pub fn App() -> impl IntoView {
    // --------------------------------------------------------
    // 1. ACTIVE NAVIGATION PANEL
    // --------------------------------------------------------

    // Controls which Activity Bar panel is selected.
    //
    // Explorer is active when Forge starts.
    let active_panel = RwSignal::new(ActivePanel::Explorer);

    // --------------------------------------------------------
    // 2. SELECTED PROJECT
    // --------------------------------------------------------

    // Stores the path of the currently selected project.
    //
    // None = no project selected.
    // Some(path) = project opened.
    let selected_project = RwSignal::new(None::<String>);

    // --------------------------------------------------------
    // 3. SELECTED SOURCE FILE — NEW
    // --------------------------------------------------------

    // Shared state between FileExplorer and CodeViewer.
    //
    // FileExplorer updates this signal when a user clicks
    // a source file such as src/main.rs.
    //
    // CodeViewer observes the signal and loads the file.
    let selected_file = RwSignal::new(None::<String>);

    // --------------------------------------------------------
    // 4. PROJECT REVISION — NEW
    // --------------------------------------------------------

    // Incremented whenever a different project is opened.
    //
    // Helps child components reject asynchronous results
    // belonging to a previously selected project.
    let project_revision = RwSignal::new(0_u64);

    // --------------------------------------------------------
    // 5. APPLICATION UI
    // --------------------------------------------------------

    view! {

        // Inject our application's existing CSS.
        <style>{APP_STYLES}</style>


        // ----------------------------------------------------
        // ROOT APPLICATION SHELL
        // ----------------------------------------------------

        <div class="forge-shell">


            // ------------------------------------------------
            // TOP HEADER
            // ------------------------------------------------

            <header class="forge-titlebar">
                "FORGE - AI Developer Agent"
            </header>


            // ------------------------------------------------
            // APPLICATION BODY
            // ------------------------------------------------

            <div class="forge-body">


                // --------------------------------------------
                // ACTIVITY BAR
                // --------------------------------------------

                // This component reads and updates active_panel.
                <ActivityBar
                    active_panel=active_panel
                />


                // --------------------------------------------
                // FILE EXPLORER SIDEBAR
                // --------------------------------------------

                // Show the Explorer when the Explorer icon
                // is selected in the Activity Bar.
                <Show

                    when=move || {
                        active_panel.get()
                            == ActivePanel::Explorer
                    }

                    // Other panels retain their placeholders.
                    fallback=move || view! {

                        <aside class="forge-explorer">

                            <div class="forge-explorer-heading">
                                {move || {
                                    active_panel.get().title()
                                }}
                            </div>

                            <p class="forge-muted">
                                {move || {
                                    active_panel.get().description()
                                }}
                            </p>

                        </aside>

                    }
                >

                    // Pass the shared application signals
                    // to the Interactive File Explorer.
                    //
                    // FIX: All three props are required.
                    <FileExplorer
                        selected_project=selected_project
                        selected_file=selected_file
                        project_revision=project_revision
                    />

                </Show>


                // --------------------------------------------
                // SOURCE CODE VIEWER — NEW
                // --------------------------------------------

                // Replaces the old static welcome workspace.
                //
                // CodeViewer displays the welcome state when
                // no file is selected, and source code when
                // selected_file contains a project-relative path.
                //
                // CodeViewer owns its own <main> element.
                <CodeViewer
                    selected_file=selected_file
                    project_revision=project_revision
                />


            </div>


            // ------------------------------------------------
            // BOTTOM STATUS BAR
            // ------------------------------------------------

            <footer class="forge-statusbar">

                // Keep the existing application version.
                <span>
                    "Forge v0.1.0"
                </span>


                // Display the selected project name.
                <span>

                    {move || {

                        selected_project
                            .get()

                            .map(|path| {

                                // Extract the last directory name.
                                let name =
                                    std::path::Path::new(&path)
                                        .file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or(&path);

                                // Build the footer message.
                                format!("Project: {name}")
                            })

                            // Original message when no project
                            // has been selected.
                            .unwrap_or_else(|| {
                                "No project opened".to_string()
                            })

                    }}

                </span>

            </footer>


        </div>
    }
}
