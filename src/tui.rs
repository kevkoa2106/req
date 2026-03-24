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

struct RequestTab {
    tokens: Vec<Token>,
    response_text: String,
    status_info: String,
    is_loading: bool,
    request_scroll: u16,
    response_scroll: u16,
}

impl RequestTab {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            response_text: String::from("Press Enter to send request"),
            status_info: String::new(),
            is_loading: false,
            request_scroll: 0,
            response_scroll: 0,
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

    fn label(&self) -> String {
        let (method, url) = self.method_and_url();
        let short_url = url
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        let short_url = if short_url.len() > 30 {
            &short_url[..30]
        } else {
            short_url
        };
        format!("{method} {short_url}")
    }

    fn request_lines(&self) -> Vec<Line<'_>> {
        let (method, url) = self.method_and_url();

        let method_color = method_to_color(&method);

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
}

struct App {
    tabs: Vec<RequestTab>,
    active_tab: usize,
    active_panel: Panel,
    should_quit: bool,
    client: reqwest::Client,
}

impl App {
    fn new(requests: Vec<Vec<Token>>) -> Self {
        let tabs = requests.into_iter().map(RequestTab::new).collect();
        Self {
            tabs,
            active_tab: 0,
            active_panel: Panel::Request,
            should_quit: false,
            client: reqwest::Client::new(),
        }
    }

    fn current_tab(&self) -> &RequestTab {
        &self.tabs[self.active_tab]
    }

    fn current_tab_mut(&mut self) -> &mut RequestTab {
        &mut self.tabs[self.active_tab]
    }

    fn scroll_up(&mut self) {
        let tab = &mut self.tabs[self.active_tab];
        match self.active_panel {
            Panel::Request => tab.request_scroll = tab.request_scroll.saturating_sub(1),
            Panel::Response => tab.response_scroll = tab.response_scroll.saturating_sub(1),
        }
    }

    fn scroll_down(&mut self) {
        let tab = &mut self.tabs[self.active_tab];
        match self.active_panel {
            Panel::Request => tab.request_scroll = tab.request_scroll.saturating_add(1),
            Panel::Response => tab.response_scroll = tab.response_scroll.saturating_add(1),
        }
    }
}

fn method_to_color(method: &str) -> Color {
    match method {
        "GET" => Color::Green,
        "POST" => Color::Yellow,
        "PUT" => Color::Blue,
        "PATCH" => Color::Magenta,
        "DELETE" => Color::Red,
        _ => Color::White,
    }
}

pub async fn run(requests: Vec<Vec<Token>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal, requests).await;
    ratatui::restore();
    result
}

async fn run_app(
    terminal: &mut DefaultTerminal,
    requests: Vec<Vec<Token>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(requests);

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
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let idx = c.to_digit(10).unwrap() as usize;
                        // 1-indexed: press '1' for tab 0, '2' for tab 1, etc.
                        if idx >= 1 && idx <= app.tabs.len() {
                            app.active_tab = idx - 1;
                        }
                    }
                    KeyCode::Enter => {
                        let tab = app.current_tab_mut();
                        if !tab.is_loading {
                            tab.is_loading = true;
                            tab.response_text = String::from("Sending request...");
                            tab.status_info = String::new();

                            terminal.draw(|frame| draw(frame, &app))?;

                            let start = Instant::now();
                            let client = app.client.clone();
                            let tokens = &app.tabs[app.active_tab].tokens;
                            match process(client, tokens).await {
                                Ok(body) => {
                                    let elapsed = start.elapsed();
                                    let tab = app.current_tab_mut();
                                    tab.response_text = match formatjson::format_json(&body) {
                                        Ok(formatted) => formatted,
                                        Err(_) => body,
                                    };
                                    tab.status_info = format!(" 200 OK  |  {:.0?}", elapsed);
                                }
                                Err(e) => {
                                    let elapsed = start.elapsed();
                                    let err_str = e.to_string();
                                    let tab = app.current_tab_mut();
                                    if err_str.starts_with("Status code ") {
                                        let status_part = err_str
                                            .split(':')
                                            .next()
                                            .unwrap_or("")
                                            .replace("Status code ", "");
                                        tab.status_info =
                                            format!(" {status_part}  |  {:.0?}", elapsed);
                                        let body =
                                            err_str.splitn(2, ": ").nth(1).unwrap_or(&err_str);
                                        tab.response_text = match formatjson::format_json(body) {
                                            Ok(formatted) => formatted,
                                            Err(_) => body.to_string(),
                                        };
                                    } else {
                                        tab.status_info = format!(" ERROR  |  {:.0?}", elapsed);
                                        tab.response_text = err_str;
                                    }
                                }
                            }
                            let tab = app.current_tab_mut();
                            tab.is_loading = false;
                            tab.response_scroll = 0;
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
            Constraint::Length(1), // tab bar
            Constraint::Min(1),    // main content
            Constraint::Length(1), // status bar
            Constraint::Length(1), // keybindings
        ])
        .split(frame.area());

    let tab_area = outer[0];
    let main_area = outer[1];
    let status_area = outer[2];
    let help_area = outer[3];

    draw_tab_bar(frame, app, tab_area);

    let panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_area);

    draw_request_panel(frame, app, panels[0]);
    draw_response_panel(frame, app, panels[1]);
    draw_status_bar(frame, app, status_area);
    draw_help_bar(frame, app, help_area);
}

fn draw_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    for (i, tab) in app.tabs.iter().enumerate() {
        let (method, _) = tab.method_and_url();
        let method_color = method_to_color(&method);
        let label = format!(" {} {} ", i + 1, tab.label());

        if i == app.active_tab {
            spans.push(Span::styled(
                label,
                Style::default()
                    .fg(Color::Black)
                    .bg(method_color)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                label,
                Style::default().fg(method_color).bg(Color::Rgb(40, 40, 40)),
            ));
        }
        spans.push(Span::raw(" "));
    }

    let tab_bar =
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Rgb(20, 20, 20)));
    frame.render_widget(tab_bar, area);
}

fn draw_request_panel(frame: &mut Frame, app: &App, area: Rect) {
    let tab = app.current_tab();
    let (method, _) = tab.method_and_url();
    let method_color = method_to_color(&method);

    let border_style = if app.active_panel == Panel::Request {
        Style::default().fg(method_color)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Request ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let lines = tab.request_lines();
    let paragraph = Paragraph::new(Text::from(lines.clone()))
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((tab.request_scroll, 0));

    frame.render_widget(paragraph, area);

    if app.active_panel == Panel::Request {
        let content_len = lines.len();
        let visible = area.height.saturating_sub(2) as usize;
        if content_len > visible {
            let mut scrollbar_state = ScrollbarState::new(content_len.saturating_sub(visible))
                .position(tab.request_scroll as usize);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                area,
                &mut scrollbar_state,
            );
        }
    }
}

fn draw_response_panel(frame: &mut Frame, app: &App, area: Rect) {
    let tab = app.current_tab();

    let border_style = if app.active_panel == Panel::Response {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if tab.is_loading {
        " Response (loading...) "
    } else {
        " Response "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let lines = tab.response_lines();
    let paragraph = Paragraph::new(Text::from(lines.clone()))
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((tab.response_scroll, 0));

    frame.render_widget(paragraph, area);

    if app.active_panel == Panel::Response {
        let content_len = lines.len();
        let visible = area.height.saturating_sub(2) as usize;
        if content_len > visible {
            let mut scrollbar_state = ScrollbarState::new(content_len.saturating_sub(visible))
                .position(tab.response_scroll as usize);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                area,
                &mut scrollbar_state,
            );
        }
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let tab = app.current_tab();
    let (method, url) = tab.method_and_url();
    let method_color = method_to_color(&method);

    let mut spans = vec![
        Span::styled(
            format!(" {method} "),
            Style::default().fg(Color::Black).bg(method_color).bold(),
        ),
        Span::styled(format!(" {url} "), Style::default().fg(Color::White)),
    ];

    if !tab.status_info.is_empty() {
        let status_color = if tab.status_info.contains("200")
            || tab.status_info.contains("201")
            || tab.status_info.contains("204")
        {
            Color::Green
        } else if tab.status_info.contains("ERROR") {
            Color::Red
        } else {
            Color::Yellow
        };
        spans.push(Span::styled(
            format!("│{}", tab.status_info),
            Style::default().fg(status_color),
        ));
    }

    let status_bar =
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Rgb(30, 30, 30)));

    frame.render_widget(status_bar, area);
}

fn draw_help_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut help_spans = vec![
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
    ];

    if app.tabs.len() > 1 {
        help_spans.push(Span::styled(
            format!(" 1-{} ", app.tabs.len()),
            Style::default().fg(Color::Black).bg(Color::DarkGray).bold(),
        ));
        help_spans.push(Span::styled(
            " Switch request ",
            Style::default().fg(Color::Gray),
        ));
        help_spans.push(Span::raw(" "));
    }

    help_spans.push(Span::styled(
        " q ",
        Style::default().fg(Color::Black).bg(Color::DarkGray).bold(),
    ));
    help_spans.push(Span::styled(" Quit ", Style::default().fg(Color::Gray)));

    let help_bar =
        Paragraph::new(Line::from(help_spans)).style(Style::default().bg(Color::Rgb(20, 20, 20)));
    frame.render_widget(help_bar, area);
}
