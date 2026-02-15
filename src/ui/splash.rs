//! Splash screen with cyberpunk-themed grid animation shown before the main TUI.

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, Instant};

use crate::ui::theme::Theme;

const SPLASH_DURATION: Duration = Duration::from_millis(2200);
const GRID_ROWS: usize = 12;

/// Draw a cyberpunk grid/matrix animation. `phase` is in 0.0..1.0 (or more for continuous).
fn draw_cyberpunk_grid(frame: &mut Frame, area: Rect, phase: f64, theme: &Theme) {
    let width = area.width as usize;
    let height = area.height.saturating_sub(2) as usize;
    if height == 0 || width == 0 {
        return;
    }

    // Animated scanline effect
    let scanline_offset = ((phase * 30.0) as usize) % 3;
    
    for row in 0..height.min(GRID_ROWS) {
        let y = area.y + 2 + row as u16;
        let mut line = String::with_capacity(width);
        
        for col in 0..width {
            // Create grid pattern with animated scanlines
            let is_grid_line = (col % 8 == 0) || (row % 4 == 0);
            let char_idx = ((col + row) as f64 * 0.1 + phase) as usize;
            
            if is_grid_line {
                // Animated grid characters
                let chars = ['─', '═', '│', '║', '┄', '┈'];
                let c = chars[char_idx % chars.len()];
                line.push(c);
            } else if (row + scanline_offset) % 3 == 0 {
                // Scanline effect
                line.push('·');
            } else {
                // Matrix-like falling characters
                let matrix_chars = ['·', '∿', '~', '≈', ' '];
                let c = matrix_chars[char_idx % matrix_chars.len()];
                line.push(c);
            }
        }
        
        // Alternate colors for grid lines
        let style = if is_grid_line || row % 2 == 0 {
            theme.style_accent()
        } else {
            theme.style_dim()
        };
        let span = Span::styled(line, style);
        frame.render_widget(Paragraph::new(span), Rect::new(area.x, y, area.width, 1));
    }
}

/// Run the splash screen: cyberpunk grid animation + title. Returns when duration elapsed or any key pressed.
pub fn run_splash(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    let theme = Theme::cyberpunk(); // Use the new cyberpunk theme for splash
    let start = Instant::now();

    loop {
        let elapsed = start.elapsed();
        if elapsed >= SPLASH_DURATION {
            break;
        }
        let phase = elapsed.as_secs_f64() * 2.0;

        terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),
                    Constraint::Min(10),
                    Constraint::Length(4),
                ])
                .split(area);

            // Cyberpunk header with double borders
            let title = Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("[ ", theme.style_border_focused()),
                    Span::styled("⚡ ORACLE ⚡", theme.style_accent_bold().add_modifier(Modifier::BOLD)),
                    Span::styled(" ]", theme.style_border_focused()),
                ]),
                Line::from(vec![
                    Span::styled("═══════════════════════════", theme.style_border()),
                ]),
                Line::from(vec![
                    Span::styled("RUST CODE INSPECTOR v∞", theme.style_dim()),
                ]),
            ])
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(theme.style_border()),
            );
            title.render(chunks[0], frame.buffer_mut());

            draw_cyberpunk_grid(frame, chunks[1], phase, &theme);

            // Cyberpunk-style progress indicator
            let progress_bar = {
                let progress = (elapsed.as_secs_f64() / SPLASH_DURATION.as_secs_f64()).min(1.0);
                let filled = (progress * 40.0) as usize;
                let bar = "█".repeat(filled) + &"░".repeat(40.saturating_sub(filled));
                format!("[{}]", bar)
            };

            let hint = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("◇ ", theme.style_border_focused()),
                    Span::styled(progress_bar, theme.style_accent()),
                ]),
                Line::from(vec![
                    Span::styled("Press any key to enter ", theme.style_muted()),
                    Span::styled(">>", theme.style_accent()),
                ]),
            ])
            .alignment(Alignment::Center);
            hint.render(chunks[2], frame.buffer_mut());
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    break;
                }
            }
        }
    }

    Ok(())
}
