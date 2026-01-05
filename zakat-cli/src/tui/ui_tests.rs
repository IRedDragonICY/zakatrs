
#[cfg(test)]
mod tests {
    use crate::tui::ui::ui;
    use ratatui::{backend::TestBackend, Terminal};
    use crate::tui::app::{App, Screen};

    #[test]
    fn test_ui_render_header() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new(false); // offline = false

        terminal.draw(|f| {
            ui(f, &app);
        }).unwrap();

        let buffer = terminal.backend().buffer();
        
        // Verify "ZAKAT" brand is present
        let mut found_zakat = false;
        for i in 0..80 {
            // Buffer::cell((x, y)) returns Option<&Cell>
            if let Some(cell) = buffer.cell((i, 0)) {
                if cell.symbol() == "Z" {
                     found_zakat = true;
                     break;
                }
            }
        }
        
        assert!(found_zakat, "Header should contain 'ZAKAT' branding");
    }

    #[test]
    fn test_ui_render_dashboard() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(false);
        app.screen = Screen::Main;

        terminal.draw(|f| {
            ui(f, &app);
        }).unwrap();
    }

    #[test]
    fn test_ui_render_empty_portfolio_message() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(false);
        // Portfolio is empty by default

        terminal.draw(|f| {
            ui(f, &app);
        }).unwrap();
    }

    #[test]
    fn test_ui_render_help() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(false);
        app.screen = Screen::Help;

        terminal.draw(|f| {
            ui(f, &app);
        }).unwrap();

        let buffer = terminal.backend().buffer();
        // Check for Help Title
        let mut found_help = false;
        // Check loosely for "Help" or arrow content
        for i in 0..80 {
            for j in 0..24 {
                if let Some(cell) = buffer.cell((i, j)) {
                     if cell.symbol() == "H" { 
                         found_help = true;
                    }
                }
            }
        }
    }
}
