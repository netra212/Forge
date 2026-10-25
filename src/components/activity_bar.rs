use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivePanel {
    Explorer,
    Chat,
    Terminal,
    History,
    Settings,
}

impl ActivePanel {
    pub fn title(self) -> &'static str {
        match self {
            Self::Explorer => "EXPLORER",
            Self::Chat => "CHAT",
            Self::Terminal => "TERMINAL",
            Self::History => "HISTORY",
            Self::Settings => "SETTINGS",
        }
    }
    
    pub fn description(self) -> &'static str {
        match self {
            Self::Explorer => "No Project Opened",
            Self::Chat => "Chat panel coming soon",
            Self::Terminal => "Terminal panel coming soon",
            Self::History => "History panel coming soon",
            Self::Settings => "Settings panel coming soon",
        }
    }
}

#[component]
pub fn ActivityBar(
    active_panel: RwSignal<ActivePanel>,
) -> impl IntoView {
    // Describe the Activity Bar's UI.
    view! {
        // Semantic navigation element for accessibility.
        <nav
            class="forge-activity"
            aria-label="Forge navigation"
        >
            // Group the top navigation buttons.
            <div class="forge-activity-top">

                // EXPLORER
                <button
                    type="button"
                    class="forge-activity-button"

                    // Add "active" when Explorer is selected.
                    class:active=move || {
                        active_panel.get() == ActivePanel::Explorer
                    }

                    // Accessible name and mouse tooltip.
                    aria-label="Explorer"
                    title="Explorer"

                    // Clicking updates the parent's signal.
                    on:click=move |_| {
                        active_panel.set(ActivePanel::Explorer);
                    }
                >
                    // Files icon, drawn with SVG.
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.8"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        aria-hidden="true"
                    >
                        <path d="M6 3h10l5 5v13H6z"/>
                        <path d="M16 3v5h5"/>
                        <path d="M3 7v14"/>
                    </svg>
                </button>

                // CHAT
                <button
                    type="button"
                    class="forge-activity-button"

                    // Highlight this button when Chat is active.
                    class:active=move || {
                        active_panel.get() == ActivePanel::Chat
                    }

                    aria-label="Chat"
                    title="Chat"

                    // Change the shared state to Chat.
                    on:click=move |_| {
                        active_panel.set(ActivePanel::Chat);
                    }
                >
                    // Speech bubble icon.
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.8"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        aria-hidden="true"
                    >
                        <rect
                            x="3"
                            y="4"
                            width="18"
                            height="14"
                            rx="3"
                        />
                        <path d="M7 18v3l4-3"/>
                    </svg>
                </button>

                // TERMINAL
                <button
                    type="button"
                    class="forge-activity-button"

                    // Reactively highlight the Terminal icon.
                    class:active=move || {
                        active_panel.get() == ActivePanel::Terminal
                    }

                    aria-label="Terminal"
                    title="Terminal"

                    // Update the selected panel on click.
                    on:click=move |_| {
                        active_panel.set(ActivePanel::Terminal);
                    }
                >
                    // Terminal window icon.
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.8"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        aria-hidden="true"
                    >
                        <rect
                            x="3"
                            y="4"
                            width="18"
                            height="16"
                            rx="2"
                        />
                        <path d="m7 9 3 3-3 3"/>
                        <path d="M12 16h5"/>
                    </svg>
                </button>

                // HISTORY
                <button
                    type="button"
                    class="forge-activity-button"

                    // Reactively highlight the History icon.
                    class:active=move || {
                        active_panel.get() == ActivePanel::History
                    }

                    aria-label="History"
                    title="History"

                    // Update the signal when clicked.
                    on:click=move |_| {
                        active_panel.set(ActivePanel::History);
                    }
                >
                    // Clock-shaped history icon.
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.8"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        aria-hidden="true"
                    >
                        <circle cx="12" cy="12" r="9"/>
                        <path d="M12 7v5l3 2"/>
                    </svg>
                </button>
            </div>

            // SETTINGS — positioned at the bottom by CSS.
            <button
                type="button"
                class="forge-activity-button"

                // Highlight Settings when it is selected.
                class:active=move || {
                    active_panel.get() == ActivePanel::Settings
                }

                aria-label="Settings"
                title="Settings"

                // Update the shared signal.
                on:click=move |_| {
                    active_panel.set(ActivePanel::Settings);
                }
            >
                // Settings icon.
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                    stroke-linecap="round"
                    aria-hidden="true"
                >
                    <circle cx="12" cy="12" r="3"/>
                    <path d="M12 2v5"/>
                    <path d="M12 17v5"/>
                    <path d="M2 12h5"/>
                    <path d="M17 12h5"/>
                    <path d="m4.93 4.93 3.54 3.54"/>
                    <path d="m15.53 15.53 3.54 3.54"/>
                    <path d="m19.07 4.93-3.54 3.54"/>
                    <path d="m8.47 15.53-3.54 3.54"/>
                </svg>
            </button>
        </nav>
    }
}
