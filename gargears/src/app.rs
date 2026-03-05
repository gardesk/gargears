//! Main application state and event loop

use crate::daemon::{check_command, DaemonCommand};
use crate::ipc::discovery::DaemonStatus;
use crate::ipc::{ConnectionManager, IpcEvent};
use crate::panels::{
    Component, GarPanel, GarbarPanel, GarbgPanel, GarclipPanel, GarfieldPanel, GarlaunchPanel,
    GarlockPanel, GarnotifyPanel, GarshotPanel, GartermPanel, GartrayPanel, PlaceholderPanel,
};
use crate::ui::{Layout, Sidebar};
use crate::ui::{Panel, PanelAction};
use anyhow::Result;
use gartk_core::{InputEvent, Key, Point, Rect, Theme};
use gartk_render::Renderer;
use gartk_x11::{Connection, EventLoop, EventLoopConfig, Window, WindowConfig};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};
use x11rb::protocol::xproto::{ConnectionExt, ImageFormat};

/// How often to refresh connections (in seconds)
const REFRESH_INTERVAL_SECS: u64 = 5;

/// How often to poll IPC events (in milliseconds)
const IPC_POLL_INTERVAL_MS: u64 = 100;

/// Main application state
pub struct App {
    window: Window,
    renderer: Renderer,
    theme: Theme,
    gc: u32,
    sidebar: Sidebar,
    layout: Layout,
    selected_component: Component,
    panels: HashMap<Component, Box<dyn Panel>>,
    connection_manager: ConnectionManager,
    daemon_status: Vec<DaemonStatus>,
    last_refresh: Instant,
    last_ipc_poll: Instant,
    status_message: Option<(String, Instant)>,
    should_quit: bool,
    visible: Arc<AtomicBool>,
    instant_apply: bool,
    /// Persistent back buffer for blitting (avoids per-frame allocation)
    blit_buffer: Option<gartk_render::Surface>,
}

/// Duration to show status messages before fading
const STATUS_MESSAGE_DURATION: Duration = Duration::from_secs(3);

impl App {
    /// Create a new application
    pub fn new(initial_panel: Option<&str>) -> Result<Self> {
        // Connect to X11
        let conn = Connection::connect(None)?;

        // Get monitor of active window for centering
        let monitor = gartk_x11::monitor_of_active_window(&conn)?;

        // Window size
        let width = 900;
        let height = 600;
        let x = monitor.rect.x + (monitor.rect.width as i32 - width as i32) / 2;
        let y = monitor.rect.y + (monitor.rect.height as i32 - height as i32) / 2;

        // Create window
        let window = Window::create(
            conn.clone(),
            WindowConfig::new()
                .title("gargears")
                .class("gargears")
                .position(x, y)
                .size(width, height)
                .transparent(true),
        )?;

        window.focus()?;

        // Create renderer
        let theme = Theme::dark();
        let renderer = Renderer::with_theme(width, height, theme.clone())?;

        // Create GC for blitting
        let gc = conn.generate_id()?;
        conn.inner()
            .create_gc(gc, window.id(), &Default::default())?;
        conn.flush()?;

        // Initialize components
        let sidebar = Sidebar::new();
        let layout = Layout::new(width, height);

        // Parse initial panel
        let selected_component = initial_panel
            .and_then(|name| Component::from_name(name))
            .unwrap_or(Component::Gar);

        // Initialize connection manager and connect to running daemons
        let mut connection_manager = ConnectionManager::new();
        let daemon_status = connection_manager.connect_all();

        // Subscribe to events
        connection_manager.subscribe_to_events();

        // Create panels
        let mut panels: HashMap<Component, Box<dyn Panel>> = HashMap::new();

        // Create all panels
        panels.insert(
            Component::Gar,
            Box::new(GarPanel::new(connection_manager.take_gar_adapter())),
        );
        panels.insert(
            Component::Garbar,
            Box::new(GarbarPanel::new(connection_manager.take_garbar_adapter())),
        );
        panels.insert(
            Component::Garbg,
            Box::new(GarbgPanel::new(connection_manager.take_garbg_adapter())),
        );
        panels.insert(
            Component::Garterm,
            Box::new(GartermPanel::new(connection_manager.take_garterm_adapter())),
        );
        panels.insert(
            Component::Gartray,
            Box::new(GartrayPanel::new(connection_manager.take_gartray_adapter())),
        );
        panels.insert(
            Component::Garshot,
            Box::new(GarshotPanel::new(connection_manager.take_garshot_adapter())),
        );
        panels.insert(
            Component::Garlock,
            Box::new(GarlockPanel::new(connection_manager.take_garlock_adapter())),
        );
        panels.insert(
            Component::Garfield,
            Box::new(GarfieldPanel::new(connection_manager.take_garfield_adapter())),
        );
        panels.insert(
            Component::Garclip,
            Box::new(GarclipPanel::new(connection_manager.take_garclip_adapter())),
        );
        panels.insert(
            Component::Garlaunch,
            Box::new(GarlaunchPanel::new(connection_manager.take_garlaunch_adapter())),
        );
        panels.insert(
            Component::Garnotify,
            Box::new(GarnotifyPanel::new(connection_manager.take_garnotify_adapter())),
        );
        panels.insert(
            Component::Garcard,
            Box::new(PlaceholderPanel::new(
                "garcard",
                "Polkit authentication agent",
                connection_manager.is_connected(Component::Garcard),
            )),
        );

        Ok(Self {
            window,
            renderer,
            theme,
            gc,
            sidebar,
            layout,
            selected_component,
            panels,
            connection_manager,
            daemon_status,
            last_refresh: Instant::now(),
            last_ipc_poll: Instant::now(),
            status_message: None,
            should_quit: false,
            visible: Arc::new(AtomicBool::new(true)),
            instant_apply: true, // Enable instant-apply by default
            blit_buffer: None,
        })
    }

    /// Create a new application for daemon mode (starts hidden)
    pub fn new_daemon(initial_panel: Option<&str>) -> Result<Self> {
        // Connect to X11
        let conn = Connection::connect(None)?;

        // Get monitor of active window for centering
        let monitor = gartk_x11::monitor_of_active_window(&conn)?;

        // Window size
        let width = 900;
        let height = 600;
        let x = monitor.rect.x + (monitor.rect.width as i32 - width as i32) / 2;
        let y = monitor.rect.y + (monitor.rect.height as i32 - height as i32) / 2;

        // Create window but don't map it yet
        let window = Window::create(
            conn.clone(),
            WindowConfig::new()
                .title("gargears")
                .class("gargears")
                .position(x, y)
                .size(width, height)
                .transparent(true)
                .map_on_create(false), // Don't show immediately
        )?;

        // Create renderer
        let theme = Theme::dark();
        let renderer = Renderer::with_theme(width, height, theme.clone())?;

        // Create GC for blitting
        let gc = conn.generate_id()?;
        conn.inner()
            .create_gc(gc, window.id(), &Default::default())?;
        conn.flush()?;

        // Initialize components
        let sidebar = Sidebar::new();
        let layout = Layout::new(width, height);

        // Parse initial panel
        let selected_component = initial_panel
            .and_then(|name| Component::from_name(name))
            .unwrap_or(Component::Gar);

        // Initialize connection manager
        let mut connection_manager = ConnectionManager::new();
        let daemon_status = connection_manager.connect_all();
        connection_manager.subscribe_to_events();

        // Create panels
        let mut panels: HashMap<Component, Box<dyn Panel>> = HashMap::new();
        panels.insert(
            Component::Gar,
            Box::new(GarPanel::new(connection_manager.take_gar_adapter())),
        );
        panels.insert(
            Component::Garbar,
            Box::new(GarbarPanel::new(connection_manager.take_garbar_adapter())),
        );
        panels.insert(
            Component::Garbg,
            Box::new(GarbgPanel::new(connection_manager.take_garbg_adapter())),
        );
        panels.insert(
            Component::Garterm,
            Box::new(GartermPanel::new(connection_manager.take_garterm_adapter())),
        );
        panels.insert(
            Component::Gartray,
            Box::new(GartrayPanel::new(connection_manager.take_gartray_adapter())),
        );
        panels.insert(
            Component::Garshot,
            Box::new(GarshotPanel::new(connection_manager.take_garshot_adapter())),
        );
        panels.insert(
            Component::Garlock,
            Box::new(GarlockPanel::new(connection_manager.take_garlock_adapter())),
        );
        panels.insert(
            Component::Garfield,
            Box::new(GarfieldPanel::new(connection_manager.take_garfield_adapter())),
        );
        panels.insert(
            Component::Garclip,
            Box::new(GarclipPanel::new(connection_manager.take_garclip_adapter())),
        );
        panels.insert(
            Component::Garlaunch,
            Box::new(GarlaunchPanel::new(connection_manager.take_garlaunch_adapter())),
        );
        panels.insert(
            Component::Garnotify,
            Box::new(GarnotifyPanel::new(connection_manager.take_garnotify_adapter())),
        );
        panels.insert(
            Component::Garcard,
            Box::new(PlaceholderPanel::new(
                "garcard",
                "Polkit authentication agent",
                connection_manager.is_connected(Component::Garcard),
            )),
        );

        Ok(Self {
            window,
            renderer,
            theme,
            gc,
            sidebar,
            layout,
            selected_component,
            panels,
            connection_manager,
            daemon_status,
            last_refresh: Instant::now(),
            last_ipc_poll: Instant::now(),
            status_message: None,
            should_quit: false,
            visible: Arc::new(AtomicBool::new(false)), // Start hidden
            instant_apply: true, // Enable instant-apply by default
            blit_buffer: None,
        })
    }

    /// Get visibility state (for IPC server)
    pub fn visible_state(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.visible)
    }

    /// Show the window
    pub fn show(&mut self) -> Result<()> {
        if !self.visible.load(Ordering::Relaxed) {
            self.window.map()?;
            self.window.focus()?;
            self.visible.store(true, Ordering::Relaxed);
            tracing::info!("Window shown");
        }
        Ok(())
    }

    /// Hide the window
    pub fn hide(&mut self) -> Result<()> {
        if self.visible.load(Ordering::Relaxed) {
            self.window.unmap()?;
            self.visible.store(false, Ordering::Relaxed);
            tracing::info!("Window hidden");
        }
        Ok(())
    }

    /// Toggle window visibility
    pub fn toggle(&mut self) -> Result<()> {
        if self.visible.load(Ordering::Relaxed) {
            self.hide()
        } else {
            self.show()
        }
    }

    /// Run the application event loop
    pub fn run(&mut self) -> Result<()> {
        let mut event_loop = EventLoop::new(&self.window, EventLoopConfig::default())?;

        // Initial render
        self.render()?;

        event_loop.run(|ev, event| {
            match event.clone() {
                InputEvent::Key(key_event) if key_event.pressed => {
                    // First check global keys
                    if self.handle_key(&key_event.key) {
                        ev.request_redraw();
                    } else {
                        // Pass to current panel
                        if let Some(panel) = self.panels.get_mut(&self.selected_component) {
                            match panel.handle_event(&event) {
                                PanelAction::Apply => {
                                    if let Err(e) = panel.apply() {
                                        self.set_status(format!("Apply failed: {}", e));
                                    } else {
                                        self.set_status("Changes applied");
                                    }
                                }
                                PanelAction::InstantApply => {
                                    if let Err(e) = panel.apply_instant() {
                                        self.set_status(format!("Apply failed: {}", e));
                                    }
                                }
                                PanelAction::Reset => {
                                    panel.reset();
                                    self.set_status("Reset to original values");
                                }
                                PanelAction::Save => {
                                    if panel.has_config_file() {
                                        if let Err(e) = panel.save_to_config() {
                                            self.set_status(format!("Save failed: {}", e));
                                        } else {
                                            self.set_status("Saved to config file");
                                        }
                                    } else {
                                        self.set_status("No config file for this component");
                                    }
                                }
                                PanelAction::Redraw | PanelAction::None => {}
                            }
                            ev.request_redraw();
                        }
                    }
                }
                InputEvent::MousePress(mouse_event) => {
                    let x = mouse_event.position.x;
                    let y = mouse_event.position.y;

                    // Check if click is in sidebar
                    let sidebar_bounds = self.layout.sidebar_bounds();
                    if x >= sidebar_bounds.x
                        && x < sidebar_bounds.x + sidebar_bounds.width as i32
                        && y >= sidebar_bounds.y
                        && y < sidebar_bounds.y + sidebar_bounds.height as i32
                    {
                        if let Some(component) = self.sidebar.component_at_y(y, &self.layout, &self.theme) {
                            self.selected_component = component;
                        }
                    } else {
                        // Pass to current panel
                        if let Some(panel) = self.panels.get_mut(&self.selected_component) {
                            match panel.handle_event(&event) {
                                PanelAction::Apply => {
                                    if let Err(e) = panel.apply() {
                                        self.set_status(format!("Apply failed: {}", e));
                                    } else {
                                        self.set_status("Changes applied");
                                    }
                                }
                                PanelAction::InstantApply => {
                                    if let Err(e) = panel.apply_instant() {
                                        self.set_status(format!("Apply failed: {}", e));
                                    }
                                }
                                PanelAction::Reset => {
                                    panel.reset();
                                    self.set_status("Reset to original values");
                                }
                                PanelAction::Save => {
                                    if panel.has_config_file() {
                                        if let Err(e) = panel.save_to_config() {
                                            self.set_status(format!("Save failed: {}", e));
                                        } else {
                                            self.set_status("Saved to config file");
                                        }
                                    } else {
                                        self.set_status("No config file for this component");
                                    }
                                }
                                PanelAction::Redraw | PanelAction::None => {}
                            }
                        }
                    }
                    ev.request_redraw();
                }
                InputEvent::MouseMove(mouse_event) => {
                    // Update hover states - only redraw if needed
                    if let Some(panel) = self.panels.get_mut(&self.selected_component) {
                        if panel.on_mouse_move(mouse_event.position.x, mouse_event.position.y) {
                            ev.request_redraw();
                        }
                    }
                }
                InputEvent::Scroll(_) => {
                    if let Some(panel) = self.panels.get_mut(&self.selected_component) {
                        panel.handle_event(&event);
                    }
                    ev.request_redraw();
                }
                InputEvent::Expose => {
                    ev.request_redraw();
                }
                InputEvent::CloseRequested => {
                    self.should_quit = true;
                }
                InputEvent::FocusOut => {
                    // Don't close on focus out - this is a config app, not a popup
                }
                InputEvent::Resize { width, height } => {
                    self.layout = Layout::new(width, height);
                    if let Ok(new_renderer) =
                        Renderer::with_theme(width, height, self.theme.clone())
                    {
                        self.renderer = new_renderer;
                    }
                    self.blit_buffer = None; // Recreate on next render
                    ev.request_redraw();
                }
                _ => {}
            }

            // Poll for IPC events (throttled)
            if self.last_ipc_poll.elapsed().as_millis() >= IPC_POLL_INTERVAL_MS as u128 {
                self.poll_ipc_events();
                self.last_ipc_poll = Instant::now();
            }

            // Periodically refresh connections
            if self.last_refresh.elapsed().as_secs() >= REFRESH_INTERVAL_SECS {
                self.refresh_connections();
                ev.request_redraw();
            }

            if ev.needs_redraw() {
                let _ = self.render();
                ev.redraw_done();
            }

            Ok(!self.should_quit)
        })?;

        Ok(())
    }

    /// Run the application in daemon mode (handles IPC commands)
    pub fn run_daemon(&mut self, command_rx: Receiver<DaemonCommand>) -> Result<()> {
        let mut event_loop = EventLoop::new(&self.window, EventLoopConfig::default())?;

        tracing::info!("Daemon mode started (window hidden, use gargearsctl to show)");

        event_loop.run(|ev, event| {
            // Check for daemon commands first
            while let Some(cmd) = check_command(&command_rx) {
                match cmd {
                    DaemonCommand::Show => {
                        let _ = self.show();
                        ev.request_redraw();
                    }
                    DaemonCommand::Hide => {
                        let _ = self.hide();
                    }
                    DaemonCommand::Toggle => {
                        let _ = self.toggle();
                        if self.visible.load(Ordering::Relaxed) {
                            ev.request_redraw();
                        }
                    }
                    DaemonCommand::Status => {
                        // Handled by IPC server directly
                    }
                    DaemonCommand::Quit => {
                        self.should_quit = true;
                    }
                }
            }

            // Only process events when visible
            if self.visible.load(Ordering::Relaxed) {
                match event.clone() {
                    InputEvent::Key(key_event) if key_event.pressed => {
                        // Escape hides in daemon mode instead of quitting
                        if matches!(key_event.key, Key::Escape) {
                            let _ = self.hide();
                        } else if self.handle_key(&key_event.key) {
                            ev.request_redraw();
                        } else if let Some(panel) = self.panels.get_mut(&self.selected_component) {
                            match panel.handle_event(&event) {
                                PanelAction::Apply => {
                                    if let Err(e) = panel.apply() {
                                        self.set_status(format!("Apply failed: {}", e));
                                    } else {
                                        self.set_status("Changes applied");
                                    }
                                }
                                PanelAction::InstantApply => {
                                    if let Err(e) = panel.apply_instant() {
                                        self.set_status(format!("Apply failed: {}", e));
                                    }
                                }
                                PanelAction::Reset => {
                                    panel.reset();
                                    self.set_status("Reset to original values");
                                }
                                PanelAction::Save => {
                                    if panel.has_config_file() {
                                        if let Err(e) = panel.save_to_config() {
                                            self.set_status(format!("Save failed: {}", e));
                                        } else {
                                            self.set_status("Saved to config file");
                                        }
                                    }
                                }
                                PanelAction::Redraw | PanelAction::None => {}
                            }
                            ev.request_redraw();
                        }
                    }
                    InputEvent::MousePress(mouse_event) => {
                        let x = mouse_event.position.x;
                        let y = mouse_event.position.y;

                        let sidebar_bounds = self.layout.sidebar_bounds();
                        if x >= sidebar_bounds.x
                            && x < sidebar_bounds.x + sidebar_bounds.width as i32
                            && y >= sidebar_bounds.y
                            && y < sidebar_bounds.y + sidebar_bounds.height as i32
                        {
                            if let Some(component) =
                                self.sidebar.component_at_y(y, &self.layout, &self.theme)
                            {
                                self.selected_component = component;
                            }
                        } else if let Some(panel) = self.panels.get_mut(&self.selected_component) {
                            match panel.handle_event(&event) {
                                PanelAction::Apply => {
                                    if let Err(e) = panel.apply() {
                                        self.set_status(format!("Apply failed: {}", e));
                                    } else {
                                        self.set_status("Changes applied");
                                    }
                                }
                                PanelAction::InstantApply => {
                                    if let Err(e) = panel.apply_instant() {
                                        self.set_status(format!("Apply failed: {}", e));
                                    }
                                }
                                PanelAction::Reset => {
                                    panel.reset();
                                    self.set_status("Reset to original values");
                                }
                                PanelAction::Save => {
                                    if panel.has_config_file() {
                                        if let Err(e) = panel.save_to_config() {
                                            self.set_status(format!("Save failed: {}", e));
                                        } else {
                                            self.set_status("Saved to config file");
                                        }
                                    }
                                }
                                PanelAction::Redraw | PanelAction::None => {}
                            }
                        }
                        ev.request_redraw();
                    }
                    InputEvent::MouseMove(mouse_event) => {
                        if let Some(panel) = self.panels.get_mut(&self.selected_component) {
                            if panel.on_mouse_move(mouse_event.position.x, mouse_event.position.y) {
                                ev.request_redraw();
                            }
                        }
                    }
                    InputEvent::Scroll(_) => {
                        if let Some(panel) = self.panels.get_mut(&self.selected_component) {
                            panel.handle_event(&event);
                        }
                        ev.request_redraw();
                    }
                    InputEvent::Expose => {
                        ev.request_redraw();
                    }
                    InputEvent::CloseRequested => {
                        // In daemon mode, hide instead of quit
                        let _ = self.hide();
                    }
                    InputEvent::Resize { width, height } => {
                        self.layout = Layout::new(width, height);
                        if let Ok(new_renderer) =
                            Renderer::with_theme(width, height, self.theme.clone())
                        {
                            self.renderer = new_renderer;
                        }
                        self.blit_buffer = None; // Recreate on next render
                        ev.request_redraw();
                    }
                    _ => {}
                }

                if ev.needs_redraw() {
                    let _ = self.render();
                    ev.redraw_done();
                }
            }

            // Poll IPC events (throttled) and refresh connections
            if self.last_ipc_poll.elapsed().as_millis() >= IPC_POLL_INTERVAL_MS as u128 {
                self.poll_ipc_events();
                self.last_ipc_poll = Instant::now();
            }
            if self.last_refresh.elapsed().as_secs() >= REFRESH_INTERVAL_SECS {
                self.refresh_connections();
            }

            Ok(!self.should_quit)
        })?;

        Ok(())
    }

    /// Poll for IPC events from connected daemons
    fn poll_ipc_events(&mut self) {
        let events = self.connection_manager.poll_events();

        for event in events {
            match event {
                IpcEvent::Connected(component) => {
                    tracing::info!("Connected to {:?}", component);
                    self.set_status(format!("Connected to {}", component.display_name()));
                    self.daemon_status = self.connection_manager.get_statuses();
                }
                IpcEvent::Disconnected(component) => {
                    tracing::info!("Disconnected from {:?}", component);
                    self.set_status(format!("Disconnected from {}", component.display_name()));
                    self.daemon_status = self.connection_manager.get_statuses();
                }
                IpcEvent::WorkspaceChanged { new, .. } => {
                    tracing::debug!("Workspace changed to {}", new);
                }
                IpcEvent::WallpaperChanged { monitor, source } => {
                    tracing::debug!("Wallpaper changed on {}: {}", monitor, source);
                }
                IpcEvent::StatusUpdate { component, .. } => {
                    tracing::debug!("Status update from {:?}", component);
                }
                IpcEvent::Error { component, message } => {
                    tracing::warn!("Error from {:?}: {}", component, message);
                    self.set_status(format!("{}: {}", component.display_name(), message));
                }
            }
        }
    }

    /// Refresh connections to daemons
    fn refresh_connections(&mut self) {
        self.connection_manager.refresh_connections();
        self.daemon_status = self.connection_manager.get_statuses();
        self.last_refresh = Instant::now();
    }

    /// Handle a key press. Returns true if handled as a global key.
    fn handle_key(&mut self, key: &Key) -> bool {
        match key {
            Key::Escape => {
                self.should_quit = true;
                true
            }
            Key::Char('q') => {
                self.should_quit = true;
                true
            }
            Key::Char('r') => {
                // Manual refresh
                self.refresh_connections();
                self.set_status("Refreshed connections");
                true
            }
            Key::Char('i') => {
                // Toggle instant-apply mode
                self.instant_apply = !self.instant_apply;
                // Update all panels
                for panel in self.panels.values_mut() {
                    panel.set_instant_apply(self.instant_apply);
                }
                self.set_status(if self.instant_apply {
                    "Instant apply enabled"
                } else {
                    "Instant apply disabled"
                });
                true
            }
            Key::Tab => {
                self.select_next_component();
                true
            }
            _ => false,
        }
    }

    /// Set a status message that will auto-expire
    fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some((message.into(), Instant::now()));
    }
    /// Clear expired status messages
    fn clear_expired_status(&mut self) {
        if let Some((_, timestamp)) = &self.status_message {
            if timestamp.elapsed() > STATUS_MESSAGE_DURATION {
                self.status_message = None;
            }
        }
    }

    /// Select the previous component
    fn select_prev_component(&mut self) {
        let components = Component::all();
        if let Some(idx) = components.iter().position(|c| *c == self.selected_component) {
            if idx > 0 {
                self.selected_component = components[idx - 1];
            }
        }
    }

    /// Select the next component
    fn select_next_component(&mut self) {
        let components = Component::all();
        if let Some(idx) = components.iter().position(|c| *c == self.selected_component) {
            if idx + 1 < components.len() {
                self.selected_component = components[idx + 1];
            }
        }
    }

    /// Render the application
    fn render(&mut self) -> Result<()> {
        // Clear background
        self.renderer.clear()?;

        // Draw sidebar
        self.sidebar.render(
            &mut self.renderer,
            &self.layout,
            &self.theme,
            self.selected_component,
            &self.daemon_status,
        )?;

        // Draw content panel
        self.render_content_panel()?;

        // Render status message if present
        self.render_status_message()?;

        // Flush and blit
        self.renderer.flush();
        self.blit_surface()?;

        Ok(())
    }

    /// Render the content panel
    fn render_content_panel(&mut self) -> Result<()> {
        let bounds = self.layout.content_bounds();

        // Draw panel background
        self.renderer.fill_rounded_rect(
            bounds,
            self.theme.border_radius,
            self.theme.item_background,
        )?;

        // Render the current panel
        if let Some(panel) = self.panels.get_mut(&self.selected_component) {
            // Panel content area (inset slightly from background)
            let content_bounds = Rect::new(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            );

            panel.render(&mut self.renderer, content_bounds, &self.theme)?;
        }

        Ok(())
    }

    /// Render status message at the bottom of the window
    fn render_status_message(&mut self) -> Result<()> {
        // Clear expired messages
        self.clear_expired_status();

        if let Some((message, timestamp)) = &self.status_message {
            let size = self.renderer.size();
            let elapsed = timestamp.elapsed();

            // Calculate fade opacity (fade out during last second)
            let opacity = if elapsed > STATUS_MESSAGE_DURATION - Duration::from_secs(1) {
                let fade_elapsed = (elapsed - (STATUS_MESSAGE_DURATION - Duration::from_secs(1))).as_secs_f64();
                (1.0 - fade_elapsed).max(0.0)
            } else {
                1.0
            };

            // Determine color based on message content
            let is_error = message.contains("failed") || message.contains("Error");
            let base_color = if is_error {
                gartk_core::Color::from_u8(0xff, 0x55, 0x55, 0xff) // Red for errors
            } else {
                gartk_core::Color::from_u8(0x50, 0xfa, 0x7b, 0xff) // Green for success
            };

            let color = base_color.with_alpha(opacity);

            let style = gartk_render::TextStyle::new()
                .font_family(&self.theme.font_family)
                .font_size(self.theme.font_size * 0.85)
                .color(color);

            // Render at bottom center
            let center = Point {
                x: (size.width / 2) as i32,
                y: (size.height - 24) as i32,
            };

            self.renderer.text_centered(message, center, &style)?;
        }

        Ok(())
    }

    /// Blit the rendered surface to the window
    fn blit_surface(&mut self) -> Result<()> {
        let size = self.renderer.size();

        // Ensure blit buffer exists and is the right size
        let needs_new_buffer = match &self.blit_buffer {
            Some(buf) => buf.width() != size.width || buf.height() != size.height,
            None => true,
        };

        if needs_new_buffer {
            self.blit_buffer = Some(gartk_render::Surface::new(size.width, size.height)?);
        }

        // Copy renderer surface to blit buffer
        {
            let ctx = self.renderer.context()?;
            ctx.target().flush();
        }

        let blit_buffer = self.blit_buffer.as_mut().unwrap();
        {
            let blit_ctx = blit_buffer.context()?;
            blit_ctx.set_source_surface(self.renderer.surface().cairo_surface(), 0.0, 0.0)?;
            blit_ctx.paint()?;
        }

        // Blit to X11 using with_data to avoid Vec allocation
        let conn = self.window.connection();
        let window_id = self.window.id();
        let gc = self.gc;
        let depth = self.window.depth();

        blit_buffer.with_data(|data| {
            let _ = conn.inner().put_image(
                ImageFormat::Z_PIXMAP,
                window_id,
                gc,
                size.width as u16,
                size.height as u16,
                0,
                0,
                0,
                depth,
                data,
            );
        })?;

        conn.flush()?;

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = self.window.connection().inner().free_gc(self.gc);
    }
}
