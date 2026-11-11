use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Padding, Paragraph, Row, Table, TableState, Wrap},
};
use std::{io, time::Duration};

#[derive(PartialEq)]
enum ActivePopup {
    None,
    Menu,
    Help,
}

pub struct TuiApp {
    popup: ActivePopup,
    menu_index: usize,
    logs: Vec<String>,
    border_color: Color,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            popup: ActivePopup::None,
            menu_index: 0,
            logs: Vec::new(),
            border_color: Color::LightCyan,
        }
    }

    pub fn push_log(&mut self, message: impl Into<String>) {
        self.logs.push(message.into());
    }

    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.main_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        res
    }

    fn main_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| {
                self.render_ui(f);
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match self.popup {
                        ActivePopup::None => match key.code {
                            KeyCode::Esc => {
                                self.popup = ActivePopup::Menu;
                                self.menu_index = 0;
                            }
                            KeyCode::Char('h') | KeyCode::Char('H') => {
                                self.popup = ActivePopup::Help;
                            }
                            KeyCode::Char('q') | KeyCode::Char('Q') => {
                                break;
                            }
                            _ => {}
                        },
                        ActivePopup::Menu => match key.code {
                            KeyCode::Up => {
                                if self.menu_index > 0 {
                                    self.menu_index -= 1;
                                } else {
                                    self.menu_index = 2;
                                }
                            }
                            KeyCode::Down => {
                                if self.menu_index < 2 {
                                    self.menu_index += 1;
                                } else {
                                    self.menu_index = 0;
                                }
                            }
                            KeyCode::Enter => match self.menu_index {
                                0 => self.popup = ActivePopup::None,
                                1 => self.popup = ActivePopup::Help,
                                2 => break,
                                _ => {}
                            },
                            KeyCode::Esc => {
                                self.popup = ActivePopup::None;
                            }
                            _ => {}
                        },
                        ActivePopup::Help => match key.code {
                            KeyCode::Esc | KeyCode::Char('h') | KeyCode::Char('H') => {
                                self.popup = ActivePopup::None;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
        Ok(())
    }

    fn render_ui(&self, f: &mut Frame) {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Aumentado levemente para acomodar o respiro dos metadados
                Constraint::Min(10),    // Caixa Central de Logs (Sem Título)
                Constraint::Length(3),  // Footer
            ])
            .split(size);

        let header_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // 0: Banner
                Constraint::Length(1), // 1: Leve espaço pequeno
                Constraint::Length(1), // 2: Texto de criador e versão
                Constraint::Length(1), // 3: Leve espaço pequeno antes da box do centro
            ])
            .split(chunks[0]);

        render_banner(f, header_layout[0]);

        let version_str = env!("CARGO_PKG_VERSION");
        let meta_info = format!("v{}", version_str);
        let meta_paragraph = Paragraph::new(meta_info)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(meta_paragraph, header_layout[2]);

        let log_content = self.logs.join("\n");
        render_custom_box(
            f,
            chunks[1],
            "",
            &log_content,
            Some(self.border_color),
            Some(Color::White),
            false,
        );

        let footer = Paragraph::new(" [ESC] Menu | [H] Ajuda | [Q] Sair Direto ")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::LightCyan)),
            );
        f.render_widget(footer, chunks[2]);

        match self.popup {
            ActivePopup::Menu => {
                let popup_area = centered_rect(90, 90, size);
                f.render_widget(Clear, popup_area);

                let opt0 = if self.menu_index == 0 {
                    " > Retomar Execução"
                } else {
                    "   Retomar Execução"
                };
                let opt1 = if self.menu_index == 1 {
                    " > Central de Ajuda"
                } else {
                    "   Central de Ajuda"
                };
                let opt2 = if self.menu_index == 2 {
                    " > Sair do Sistema"
                } else {
                    "   Sair do Sistema"
                };

                let menu_content = format!(
                    "\n{}\n{}\n{}\n\n [▲/▼] Navegar | [ENTER] Selecionar",
                    opt0, opt1, opt2
                );

                render_custom_box(
                    f,
                    popup_area,
                    " MENU PRINCIPAL ",
                    &menu_content,
                    Some(Color::Cyan),
                    Some(Color::White),
                    true,
                );
            }
            ActivePopup::Help => {
                let popup_area = centered_rect(90, 90, size);
                f.render_widget(Clear, popup_area);

                let help_text = "\n • ESC       : Abre o menu de opções interativo\n • H         : Exibe este painel de ajuda\n • Q         : Encerra imediatamente o programa\n • Setas     : Navega pelas opções do menu\n • ENTER     : Confirma a seleção no menu\n\nPressione ESC para fechar.";

                render_custom_box(
                    f,
                    popup_area,
                    " CENTRAL DE AJUDA ",
                    help_text,
                    Some(Color::Cyan),
                    Some(Color::White),
                    true,
                );
            }
            ActivePopup::None => {}
        }
    }

    pub fn table_selector(
        &mut self,
        table_title: &str,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    ) -> Result<Option<Vec<String>>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut table_state = TableState::default();
        if !rows.is_empty() {
            table_state.select(Some(0));
        }

        let mut selected_row_data: Option<Vec<String>> = None;
        let tick_rate = Duration::from_millis(100);

        loop {
            terminal.draw(|f| {
                let size = f.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(9), // Header e Banner
                        Constraint::Min(10),   // Tabela Central
                        Constraint::Length(3), // Footer
                    ])
                    .split(size);

                // Renderiza o topo (Banner + Versão)
                let header_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(6),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ])
                    .split(chunks[0]);

                render_banner(f, header_layout[0]);
                let version_str = env!("CARGO_PKG_VERSION");
                let meta_info = format!("v{}", version_str);
                let meta_paragraph = Paragraph::new(meta_info)
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center);
                f.render_widget(meta_paragraph, header_layout[2]);

                // Constrói o cabeçalho da tabela
                let header_cells = headers.iter().map(|h| {
                    Cell::from(h.clone()).style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                });
                let header_row = Row::new(header_cells).style(Style::default().bg(Color::DarkGray));

                // Constrói as linhas da tabela
                let table_rows: Vec<Row> = rows
                    .iter()
                    .map(|row_data| {
                        let cells = row_data
                            .iter()
                            .map(|cell_text| Cell::from(cell_text.clone()));
                        Row::new(cells)
                    })
                    .collect();

                // Define larguras proporcionais para as colunas
                let col_count = headers.len().max(1) as u16;
                let percentage = 100 / col_count;
                let constraints: Vec<Constraint> = (0..headers.len())
                    .map(|_| Constraint::Percentage(percentage))
                    .collect();

                let table = Table::new(table_rows, constraints)
                    .header(header_row)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(self.border_color))
                            .title(format!(" {} ({}) ", table_title, rows.len())),
                    )
                    .row_highlight_style(
                        Style::default()
                            .bg(Color::Blue)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(">> ");

                f.render_stateful_widget(table, chunks[1], &mut table_state);

                // Footer de navegação
                let footer = Paragraph::new(" [▲/▼] Navegar | [ENTER] Selecionar Alvo ")
                    .style(Style::default().fg(Color::Cyan))
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::LightCyan)),
                    );
                f.render_widget(footer, chunks[2]);
            })?;

            // Eventos do teclado
            if event::poll(tick_rate)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Down => {
                            if !rows.is_empty() {
                                let i = match table_state.selected() {
                                    Some(i) => {
                                        if i >= rows.len() - 1 {
                                            0
                                        } else {
                                            i + 1
                                        }
                                    }
                                    None => 0,
                                };
                                table_state.select(Some(i));
                            }
                        }
                        KeyCode::Up => {
                            if !rows.is_empty() {
                                let i = match table_state.selected() {
                                    Some(i) => {
                                        if i == 0 {
                                            rows.len() - 1
                                        } else {
                                            i - 1
                                        }
                                    }
                                    None => 0,
                                };
                                table_state.select(Some(i));
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(i) = table_state.selected() {
                                if i < rows.len() {
                                    selected_row_data = Some(rows[i].clone());
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(selected_row_data)
    }
}

pub fn render_banner(f: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(Span::styled(
            "    ███████╗██╗██╗     ██╗   ██╗███╗   ███╗██╗    ██╗ █████╗ ██████╗ ███████╗",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "    ██╔════╝██║██║     ██║   ██║████╗ ████║██║    ██║██╔══██╗██╔══██╗██╔════╝",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "    █████╗  ██║██║     ██║   ██║██╔████╔██║██║ █╗ ██║███████║██████╔╝█████╗  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "    ██╔══╝  ██║██║     ██║   ██║██║╚██╔╝██║██║███╗██║██╔══██║██╔══██╗██╔══╝  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "    ██║     ██║███████╗╚██████╔╝██║ ╚═╝ ██║╚███╔███╔╝██║  ██║██║  ██║███████╗",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "    ╚═╝     ╚═╝╚══════╝ ╚═════╝ ╚═╝     ╚═╝ ╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    let paragraph = Paragraph::new(text).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

pub fn render_custom_box(
    f: &mut Frame,
    area: Rect,
    title: &str,
    content: &str,
    border_color: Option<Color>,
    text_color: Option<Color>,
    solid_bg: bool,
) {
    let mut block = Block::default();

    if !title.is_empty() {
        block = block.title(title);
    }

    if let Some(bc) = border_color {
        block = block
            .borders(Borders::ALL)
            .border_style(Style::default().fg(bc));
    } else {
        block = block.borders(Borders::NONE);
    }

    block = block.padding(Padding::new(2, 2, 2, 1));

    if solid_bg {
        block = block.style(Style::default().bg(Color::Black));
    }

    let mut paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: true });

    if let Some(tc) = text_color {
        paragraph = paragraph.style(Style::default().fg(tc));
    }

    f.render_widget(paragraph, area);
}

pub fn render_success(msg: &str) -> String {
    format!("\n[✔ SUCESSO] {}\n", msg)
}

pub fn render_error(msg: &str) -> String {
    format!("\n[✖ ERRO] {}\n", msg)
}

pub fn render_alert(msg: &str) -> String {
    format!("\n[! ALERTA] {}\n", msg)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
