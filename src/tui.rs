use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use crate::parser::{Token, TokenType, process};

#[derive(PartialEq)]
enum Panel {
    Request,
    Response,
}

struct App {
    tokens: Vec<Token>,
    response_text: String,
    status_info: String,
    is_loading: bool,
    request_scroll: u16,
    response_scroll: u16,
    active_panel: Panel,
    should_quit: bool,
    client: reqwest::Client,
}

impl App {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            response_text: String::from("Press Enter to send request"),
            status_info: String::new(),
            is_loading: false,
            request_scroll: 0,
            response_scroll: 0,
            active_panel: Panel::Request,
            should_quit: false,
            client: reqwest::Client::new(),
        }
    }

    fn method_and_url(&self) -> (String, String) {
        let mut method = String::new();
        let mut url = String::new();
        for token in &self.tokens {
            match token.token_type {
                TokenType::Method => method = token.value.clone(),
                TokenType::URL => url = token.value.clone(),
                _ => {}
            }
        }
        (method, url)
    }

    fn request_lines(&self) -> Vec<Line<'_>> {
        let (method, url) = self.method_and_url();

        let method_color = match method.as_str() {
            "GET" => Color::Green,
            "POST" => Color::Yellow,
            "PUT" => Color::Blue,
            "PATCH" => Color::Magenta,
            "DELETE" => Color::Red,
            _ => Color::White,
        };

        let mut lines: Vec<Line> = vec![Line::from(vec![
            Span::styled(
                format!(" {method} "),
                Style::default()
                    .fg(Color::Black)
                    .bg(method_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(url, Style::default().fg(Color::Cyan)),
        ])];

        lines.push(Line::raw(""));

        // Headers
        let mut i = 0;
        while i < self.tokens.len() {
            if matches!(self.tokens[i].token_type, TokenType::Header) {
                let header_name = &self.tokens[i].value;
                let header_value = if i + 1 < self.tokens.len() {
                    if matches!(self.tokens[i + 1].token_type, TokenType::HeaderValue) {
                        i += 1;
                        &self.tokens[i].value
                    } else {
                        ""
                    }
                } else {
                    ""
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{header_name}: "),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(header_value, Style::default().fg(Color::White)),
                ]));
            }
            i += 1;
        }

        // Body
        for token in &self.tokens {
            if matches!(token.token_type, TokenType::Body) {
                lines.push(Line::raw(""));
                // Pretty-print the body JSON if possible
                let body_str = match formatjson::format_json(&token.value) {
                    Ok(formatted) => formatted,
                    Err(_) => token.value.clone(),
                };
                for line in body_str.lines() {
                    lines.push(Line::styled(
                        line.to_string(),
                        Style::default().fg(Color::White),
                    ));
                }
            }
        }

        lines
    }

    fn response_lines(&self) -> Vec<Line<'_>> {
        self.response_text
            .lines()
            .map(|l| Line::raw(l.to_string()))
            .collect()
    }

    fn scroll_up(&mut self) {
        match self.active_panel {
            Panel::Request => self.request_scroll = self.request_scroll.saturating_sub(1),
            Panel::Response => self.response_scroll = self.response_scroll.saturating_sub(1),
        }
    }

    fn scroll_down(&mut self) {
        match self.active_panel {
            Panel::Request => self.request_scroll = self.request_scroll.saturating_add(1),
            Panel::Response => self.response_scroll = self.response_scroll.saturating_add(1),
        }
    }
}

pub async fn run(tokens: Vec<Token>) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal, tokens).await;
    ratatui::restore();
    result
}

async fn run_app(
    terminal: &mut DefaultTerminal,
    tokens: Vec<Token>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(tokens);

    loop {
        terminal.draw(|frame| draw(frame, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Tab => {
                        app.active_panel = match app.active_panel {
                            Panel::Request => Panel::Response,
                            Panel::Response => Panel::Request,
                        };
                    }
                    KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                    KeyCode::Enter => {
                        if !app.is_loading {
                            app.is_loading = true;
                            app.response_text = String::from("Sending request...");
                            app.status_info = String::new();

                            // Redraw to show loading state
                            terminal.draw(|frame| draw(frame, &app))?;

                            let start = Instant::now();
                            match process(app.client.clone(), &app.tokens).await {
                                Ok(body) => {
                                    let elapsed = start.elapsed();
                                    app.response_text = match formatjson::format_json(&body) {
                                        Ok(formatted) => formatted,
                                        Err(_) => body,
                                    };
                                    app.status_info = format!(" 200 OK  |  {:.0?}", elapsed);
                                }
                                Err(e) => {
                                    let elapsed = start.elapsed();
                                    let err_str = e.to_string();
                                    // Try to extract status code from error message
                                    if err_str.starts_with("Status code ") {
                                        let status_part = err_str
                                            .split(':')
                                            .next()
                                            .unwrap_or("")
                                            .replace("Status code ", "");
                                        app.status_info =
                                            format!(" {status_part}  |  {:.0?}", elapsed);
                                        // Show error body formatted if possible
                                        let body =
                                            err_str.splitn(2, ": ").nth(1).unwrap_or(&err_str);
                                        app.response_text = match formatjson::format_json(body) {
                                            Ok(formatted) => formatted,
                                            Err(_) => body.to_string(),
                                        };
                                    } else {
                                        app.status_info = format!(" ERROR  |  {:.0?}", elapsed);
                                        app.response_text = err_str;
                                    }
                                }
                            }
                            app.is_loading = false;
                            app.response_scroll = 0;
                            app.active_panel = Panel::Response;
                        }
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn draw(frame: &mut Frame, app: &App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // main content
            Constraint::Length(1), // status bar
            Constraint::Length(1), // keybindings
        ])
        .split(frame.area());

    let main_area = outer[0];
    let status_area = outer[1];
    let help_area = outer[2];

    // Split main area into request (top) and response (bottom)
    let panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_area);

    draw_request_panel(frame, app, panels[0]);
    draw_response_panel(frame, app, panels[1]);
    draw_status_bar(frame, app, status_area);
    draw_help_bar(frame, help_area);
}

fn draw_request_panel(frame: &mut Frame, app: &App, area: Rect) {
    let (method, _) = app.method_and_url();
    let method_color = match method.as_str() {
        "GET" => Color::Green,
        "POST" => Color::Yellow,
        "PUT" => Color::Blue,
        "PATCH" => Color::Magenta,
        "DELETE" => Color::Red,
        _ => Color::White,
    };

    let border_style = if app.active_panel == Panel::Request {
        Style::default().fg(method_color)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Request ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let lines = app.request_lines();
    let paragraph = Paragraph::new(Text::from(lines.clone()))
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.request_scroll, 0));

    frame.render_widget(paragraph, area);

    // Scrollbar
    if app.active_panel == Panel::Request {
        let content_len = lines.len();
        let visible = area.height.saturating_sub(2) as usize;
        if content_len > visible {
            let mut scrollbar_state = ScrollbarState::new(content_len.saturating_sub(visible))
                .position(app.request_scroll as usize);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                area,
                &mut scrollbar_state,
            );
        }
    }
}

fn draw_response_panel(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.active_panel == Panel::Response {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if app.is_loading {
        " Response (loading...) "
    } else {
        " Response "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let lines = app.response_lines();
    let paragraph = Paragraph::new(Text::from(lines.clone()))
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.response_scroll, 0));

    frame.render_widget(paragraph, area);

    // Scrollbar
    if app.active_panel == Panel::Response {
        let content_len = lines.len();
        let visible = area.height.saturating_sub(2) as usize;
        if content_len > visible {
            let mut scrollbar_state = ScrollbarState::new(content_len.saturating_sub(visible))
                .position(app.response_scroll as usize);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                area,
                &mut scrollbar_state,
            );
        }
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (method, url) = app.method_and_url();

    let method_color = match method.as_str() {
        "GET" => Color::Green,
        "POST" => Color::Yellow,
        "PUT" => Color::Blue,
        "PATCH" => Color::Magenta,
        "DELETE" => Color::Red,
        _ => Color::White,
    };

    let mut spans = vec![
        Span::styled(
            format!(" {method} "),
            Style::default().fg(Color::Black).bg(method_color).bold(),
        ),
        Span::styled(format!(" {url} "), Style::default().fg(Color::White)),
    ];

    if !app.status_info.is_empty() {
        // Color the status based on success/error
        let status_color = if app.status_info.contains("200")
            || app.status_info.contains("201")
            || app.status_info.contains("204")
        {
            Color::Green
        } else if app.status_info.contains("ERROR") {
            Color::Red
        } else {
            Color::Yellow
        };
        spans.push(Span::styled(
            format!("│{}", app.status_info),
            Style::default().fg(status_color),
        ));
    }

    let status_bar =
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Rgb(30, 30, 30)));

    frame.render_widget(status_bar, area);
}

fn draw_help_bar(frame: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled(
            " Enter ",
            Style::default().fg(Color::Black).bg(Color::DarkGray).bold(),
        ),
        Span::styled(" Send ", Style::default().fg(Color::Gray)),
        Span::raw(" "),
        Span::styled(
            " Tab ",
            Style::default().fg(Color::Black).bg(Color::DarkGray).bold(),
        ),
        Span::styled(" Switch panel ", Style::default().fg(Color::Gray)),
        Span::raw(" "),
        Span::styled(
            " j/k ",
            Style::default().fg(Color::Black).bg(Color::DarkGray).bold(),
        ),
        Span::styled(" Scroll ", Style::default().fg(Color::Gray)),
        Span::raw(" "),
        Span::styled(
            " q ",
            Style::default().fg(Color::Black).bg(Color::DarkGray).bold(),
        ),
        Span::styled(" Quit ", Style::default().fg(Color::Gray)),
    ]);

    let help_bar = Paragraph::new(help).style(Style::default().bg(Color::Rgb(20, 20, 20)));
    frame.render_widget(help_bar, area);
}
