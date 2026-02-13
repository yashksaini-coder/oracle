//! Oracle - Rust Code Inspector
//!
//! A terminal-based Rust code inspector with beautiful TUI.

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use oracle_lib::{
    app::App,
    ui::{app::Focus, AnimationState, OracleUi},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io, path::PathBuf, time::Duration};

fn main() -> Result<()> {
    // Parse command line args
    let args: Vec<String> = env::args().collect();
    let project_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        env::current_dir()?
    };

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create and run app
    let mut app = App::new();

    // Try to load settings (ignore errors, use defaults)
    let _ = app.load_settings();

    // Analyze the project
    if let Err(e) = app.analyze_project(&project_path) {
        app.status_message = format!("Analysis failed: {}", e);
    }

    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    let mut animation = AnimationState::new();
    let mut inspector_scroll: usize = 0;
    let mut last_selected: Option<usize> = None;
    
    loop {
        // Update animations
        animation.update();
        
        // Reset inspector scroll on selection change
        let current_selected = app.list_state.selected();
        if current_selected != last_selected {
            inspector_scroll = 0;
            animation.on_selection_change();
            last_selected = current_selected;
        }

        // Draw UI
        terminal.draw(|frame| {
            let filtered = app.get_filtered_items();
            let selected = app.list_state.selected();
            
            // Get installed crate items if viewing a crate
            let installed_items: Vec<&oracle_lib::analyzer::AnalyzedItem> = 
                app.installed_crate_filtered
                    .iter()
                    .filter_map(|&i| app.installed_crate_items.get(i))
                    .collect();

            let ui = OracleUi::new(&app.theme)
                .items(&app.items)
                .filtered_items(&filtered)
                .list_selected(selected)
                .candidates(&app.filtered_candidates)
                .crate_info(app.crate_info.as_ref())
                .dependency_tree(&app.dependency_tree)
                .installed_crates(&app.installed_crates_list)
                .selected_installed_crate(app.selected_installed_crate.as_ref())
                .installed_crate_items(&installed_items)
                .search_input(&app.search_input)
                .current_tab(app.current_tab)
                .focus(app.focus)
                .selected_item(app.selected_item())
                .completion_selected(app.completion_selected)
                .show_completion(app.show_completion)
                .show_help(app.show_help)
                .status_message(&app.status_message)
                .inspector_scroll(inspector_scroll)
                .animation_state(&animation);

            frame.render_widget(ui, frame.area());
        })?;

        if app.should_quit {
            break;
        }

        // Handle events with shorter poll time when animating
        let poll_duration = if animation.is_animating() {
            Duration::from_millis(16) // ~60fps when animating
        } else {
            Duration::from_millis(50)
        };

        if event::poll(poll_duration)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_event(app, key.code, key.modifiers, &mut inspector_scroll, &mut animation);
                }
            }
        }
    }

    Ok(())
}

fn handle_key_event(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    inspector_scroll: &mut usize,
    animation: &mut AnimationState,
) {
    use oracle_lib::ui::app::Tab;
    
    // Global shortcuts
    match code {
        KeyCode::Char('q') if modifiers.is_empty() && app.focus != Focus::Search => {
            app.should_quit = true;
            return;
        }
        KeyCode::Char('?') if modifiers.is_empty() && app.focus != Focus::Search => {
            app.toggle_help();
            return;
        }
        KeyCode::Esc => {
            if app.show_help {
                app.show_help = false;
            } else if app.show_completion {
                app.show_completion = false;
            } else if app.current_tab == Tab::InstalledCrates && app.selected_installed_crate.is_some() {
                // Go back to crate list
                app.clear_installed_crate();
            } else if !app.search_input.is_empty() {
                app.clear_search();
            } else {
                app.should_quit = true;
            }
            return;
        }
        _ => {}
    }

    // Help is open - any key closes it
    if app.show_help {
        app.show_help = false;
        return;
    }

    // Tab switching with number keys
    match code {
        KeyCode::Char('1') if modifiers.is_empty() && app.focus != Focus::Search => {
            app.current_tab = Tab::Types;
            app.list_state.select(Some(0));
            app.filter_items();
            animation.on_tab_change();
            return;
        }
        KeyCode::Char('2') if modifiers.is_empty() && app.focus != Focus::Search => {
            app.current_tab = Tab::Functions;
            app.list_state.select(Some(0));
            app.filter_items();
            animation.on_tab_change();
            return;
        }
        KeyCode::Char('3') if modifiers.is_empty() && app.focus != Focus::Search => {
            app.current_tab = Tab::Modules;
            app.list_state.select(Some(0));
            app.filter_items();
            animation.on_tab_change();
            return;
        }
        KeyCode::Char('4') if modifiers.is_empty() && app.focus != Focus::Search => {
            app.current_tab = Tab::Dependencies;
            app.list_state.select(Some(0));
            app.filter_items();
            animation.on_tab_change();
            return;
        }
        KeyCode::Char('5') if modifiers.is_empty() && app.focus != Focus::Search => {
            app.current_tab = Tab::InstalledCrates;
            app.list_state.select(Some(0));
            if app.installed_crates_list.is_empty() {
                let _ = app.scan_installed_crates();
            }
            app.filter_items();
            animation.on_tab_change();
            return;
        }
        _ => {}
    }

    // Focus-specific handling
    match app.focus {
        Focus::Search => handle_search_input(app, code, modifiers),
        Focus::List => handle_list_input(app, code, modifiers),
        Focus::Inspector => handle_inspector_input(app, code, modifiers, inspector_scroll),
    }
}

fn handle_search_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        KeyCode::Char(c) => {
            app.on_char(c);
        }
        KeyCode::Backspace => {
            app.on_backspace();
        }
        KeyCode::Down => {
            if app.show_completion {
                app.next_completion();
            } else {
                app.focus = Focus::List;
            }
        }
        KeyCode::Up => {
            if app.show_completion {
                app.prev_completion();
            }
        }
        KeyCode::Tab if modifiers.is_empty() => {
            if app.show_completion {
                app.select_completion();
            } else {
                app.next_focus();
            }
        }
        KeyCode::BackTab => {
            app.prev_focus();
        }
        KeyCode::Enter => {
            if app.show_completion {
                app.select_completion();
            } else {
                app.filter_items();
                app.focus = Focus::List;
            }
        }
        _ => {}
    }
}

fn handle_list_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    use oracle_lib::ui::app::Tab;
    
    match code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_item();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.prev_item();
        }
        KeyCode::Tab if modifiers.is_empty() => {
            app.next_focus();
        }
        KeyCode::BackTab => {
            app.prev_focus();
        }
        KeyCode::Char('/') => {
            app.focus = Focus::Search;
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            // Handle installed crates specially
            if app.current_tab == Tab::InstalledCrates && app.selected_installed_crate.is_none() {
                // Select a crate from the list
                if let Some(idx) = app.list_state.selected() {
                    // Filter the list like we do in UI
                    let query = app.search_input.to_lowercase();
                    let crate_name: Option<String> = app.installed_crates_list.iter()
                        .filter(|name| query.is_empty() || name.to_lowercase().contains(&query))
                        .nth(idx)
                        .cloned();
                    
                    if let Some(name) = crate_name {
                        let _ = app.select_installed_crate(&name);
                        app.list_state.select(Some(0));
                    }
                }
            } else {
                app.focus = Focus::Inspector;
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            // Go back in installed crates
            if app.current_tab == Tab::InstalledCrates && app.selected_installed_crate.is_some() {
                app.clear_installed_crate();
            } else {
                app.focus = Focus::Search;
            }
        }
        KeyCode::Home | KeyCode::Char('g') => {
            let len = app.get_current_list_len();
            if len > 0 {
                app.list_state.select(Some(0));
            }
        }
        KeyCode::End | KeyCode::Char('G') => {
            let len = app.get_current_list_len();
            if len > 0 {
                app.list_state.select(Some(len - 1));
            }
        }
        KeyCode::PageDown => {
            // Jump 10 items
            for _ in 0..10 {
                app.next_item();
            }
        }
        KeyCode::PageUp => {
            for _ in 0..10 {
                app.prev_item();
            }
        }
        _ => {}
    }
}

fn handle_inspector_input(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    inspector_scroll: &mut usize,
) {
    match code {
        KeyCode::Tab if modifiers.is_empty() => {
            app.next_focus();
        }
        KeyCode::BackTab => {
            app.prev_focus();
        }
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
            app.focus = Focus::List;
        }
        KeyCode::Char('/') => {
            app.focus = Focus::Search;
        }
        // Scroll the inspector content
        KeyCode::Down | KeyCode::Char('j') => {
            *inspector_scroll = inspector_scroll.saturating_add(1);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            *inspector_scroll = inspector_scroll.saturating_sub(1);
        }
        KeyCode::PageDown => {
            *inspector_scroll = inspector_scroll.saturating_add(10);
        }
        KeyCode::PageUp => {
            *inspector_scroll = inspector_scroll.saturating_sub(10);
        }
        KeyCode::Home | KeyCode::Char('g') => {
            *inspector_scroll = 0;
        }
        _ => {}
    }
}
