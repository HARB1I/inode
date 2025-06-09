use std::io::{self};
use std::path::PathBuf;

use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::layout::Size;
use ratatui::text::Span;
use ratatui::widgets::{List, ListItem};
use ratatui::{
    DefaultTerminal, Frame, buffer,
    layout::{self, Constraint, Layout, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.hide_cursor()?;
    let size = terminal.size()?;

    let mut app = App::new(size);

    let app_result = app.run(&mut terminal);

    ratatui::restore();

    if app.get_path {
        // 🔽 Печатаем путь после выхода
        println!("{}", app.file_manager.current_path.display());
    }

    app_result
}

pub struct App {
    exit: bool,
    file_manager: FileManager,
    selected_index: usize,
    max_list: u16,
    offset_y: u16,
    get_path: bool,
}

impl App {
    fn new(size: Size) -> Self {
        Self {
            exit: false,
            file_manager: FileManager::new(),
            selected_index: 0,
            max_list: size.height - 8,
            offset_y: 0,
            get_path: false,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.draw(|f| self.draw(f))?;
        while !self.exit {
            self.handle_events()?;
            terminal.draw(|f| self.draw(f))?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => self.exit = true,
                    KeyCode::Down => {
                        if self.selected_index < self.file_manager.entries.len().saturating_sub(1) {
                            if self.selected_index - self.offset_y as usize + 2
                                > self.max_list as usize
                            {
                                self.offset_y += 1;
                            }

                            self.selected_index += 1;
                        }
                    }
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            if self.selected_index - 1 < self.offset_y as usize {
                                self.offset_y -= 1;
                            }

                            self.selected_index -= 1;
                        }
                    }
                    KeyCode::Right => {
                        let ok = self.file_manager.navigate_to(self.selected_index);
                        if ok {
                            self.offset_y = 0;
                            self.selected_index = 0;
                        }
                    }
                    KeyCode::Left => {
                        self.offset_y = 0;
                        self.file_manager.go_back();
                        self.selected_index = 0;
                    }
                    KeyCode::Enter => {
                        self.get_path = true;
                        self.exit = true;
                    }
                    _ => {}
                }
            }
        } else if let event::Event::Resize(_, y) = event::read()? {
            self.max_list = y - 8;
            if self.selected_index - self.offset_y as usize > self.max_list as usize {
                self.selected_index = self.max_list as usize;
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer)
    where
        Self: Sized,
    {
        let vertical_layout =
            Layout::vertical([Constraint::Percentage(0), Constraint::Percentage(100)]);

        let [_, gauge_area] = vertical_layout.areas(area);

        let block = Block::bordered()
            .title(Line::from(" Explorer "))
            .border_set(border::THICK);

        let content = Text::from(vec![
            Line::from(format!(
                "current path: {}",
                self.file_manager.current_path.display()
            )),
            Line::from("Quick help:"),
            Line::from(
                "           <ENTER>:open  <RIGHT>:forward   <LEFT>:back    <UP>:up   <DOWN>:down",
            ),
            // Line::from(format!(
            //     "               <Q>:exit             DEBUG: sel_idx: {}, offset_y: {}, len: {}, max_list: {}",
            //     self.selected_index,
            //     self.offset_y,
            //     self.file_manager.entries.len(),
            //     self.max_list
            // )),
            Line::from("               <Q>:exit"),
        ]);

        let paragraph = Paragraph::new(content)
            .block(block)
            .style(Style::default().fg(Color::Rgb(82, 165, 163)));

        let items: Vec<ListItem> = self
            .file_manager
            .entries
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= self.offset_y as usize)
            .map(|(i, entry)| {
                let style = if i == self.selected_index {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let icon = if entry.is_dir {
                    "📁"
                } else {
                    get_icon_from_extension(entry.extension.as_deref())
                };
                let text = format!("{} {}", icon, entry.name);

                ListItem::new(Span::styled(text, style))
            })
            .collect();

        let list = List::new(items)
            .block(Block::bordered())
            .highlight_symbol("▶ ");

        paragraph.render(
            Rect {
                x: gauge_area.left(),
                y: gauge_area.top(),
                width: gauge_area.width,
                height: 6,
            },
            buf,
        );

        list.render(
            Rect {
                x: gauge_area.left(),
                y: gauge_area.top() + 6,
                width: gauge_area.width,
                height: gauge_area.height - 6,
            },
            buf,
        );
    }
}

struct FileManager {
    current_path: PathBuf,
    entries: Vec<FileEntry>,
}

#[derive(Clone)]
struct FileEntry {
    name: String,
    is_dir: bool,
    extension: Option<String>,
}

impl FileManager {
    pub fn new() -> Self {
        let current_path = std::env::current_dir().unwrap_or_default();
        let entries = Self::read_dir(&current_path);

        Self {
            current_path,
            entries,
        }
    }

    fn read_dir(path: &PathBuf) -> Vec<FileEntry> {
        std::fs::read_dir(path)
            .map(|entries| {
                entries
                    .filter_map(|res| {
                        let entry = res.ok()?;
                        let extension = if entry.path().is_dir() {
                            None
                        } else {
                            entry
                                .path()
                                .extension()
                                .and_then(|s| s.to_str())
                                .map(|s| s.to_lowercase())
                        };

                        Some(FileEntry {
                            name: entry.file_name().into_string().unwrap_or_default(),
                            is_dir: entry.path().is_dir(),
                            extension,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn navigate_to(&mut self, index: usize) -> bool {
        if let Some(entry) = self.entries.get(index).cloned() {
            if entry.is_dir {
                let new_path = self.current_path.join(entry.name);
                self.current_path = new_path;
                self.entries = Self::read_dir(&self.current_path);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn go_back(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.entries = Self::read_dir(&self.current_path);
        }
    }
}

fn get_icon_from_extension(ext: Option<&str>) -> &'static str {
    match ext {
        // 🧠 Языки программирования
        Some("rs") => "🦀",                  // Rust
        Some("py") => "🐍",                  // Python
        Some("js") => "📜",                  // JavaScript
        Some("ts") => "📘",                  // TypeScript
        Some("go") => "🐹",                  // Go
        Some("java") => "☕",                // Java
        Some("c") => "🇨",                    // C
        Some("cpp" | "cc" | "cxx") => "🇨++", // C++
        Some("cs") => "🩸",                  // C#
        Some("php") => "🐘",                 // PHP
        Some("rb") => "💎",                  // Ruby
        Some("swift") => "🍏",               // Swift
        Some("kt" | "kts") => "🤖",          // Kotlin
        Some("dart") => "🎯",                // Dart
        Some("scala") => "🧪",               // Scala
        Some("pl") => "🐪",                  // Perl
        Some("r") => "📊",                   // R
        Some("hs") => "🧮",                  // Haskell
        Some("lua") => "🌘",                 // Lua
        Some("sh" | "bash") => "⚡",         // Shell Script
        Some("ps1") => "🐚",                 // PowerShell
        Some("vbs") => "🪟",                 // VBScript
        Some("m") => "🪟",                   // MATLAB — обновлено
        Some("jl") => "🟦",                  // Julia — обновлено

        // 📄 Текстовые/форматированные документы
        Some("txt") => "📄",
        Some("md") => "📝",
        Some("log") => "📋",
        Some("csv") => "🧮",
        Some("xml") => "📄",
        Some("toml") => "🔧",

        // 📦 JSON
        Some("json") => "📦",

        Some("yaml" | "yml") => "📄",

        // 🎨 Веб-технологии
        Some("html" | "htm") => "🌐",
        Some("css") => "🎨",
        Some("scss" | "sass") => "🎨",
        Some("jsx") => "⚛️",
        Some("tsx") => "📘",

        // 📁 Изображения
        Some("png" | "jpg" | "jpeg" | "gif") => "🖼️",
        Some("svg") => "🖼️",
        Some("webp") => "🖼️",

        // 📂 Архивы и пакеты
        Some("zip" | "tar" | "gz" | "7z" | "xz" | "bz2" | "zst") => "📦",
        Some("deb") => "🐧",
        Some("apk") => "📱",
        Some("rpm") => "📦",
        Some("jar") => "☕",
        Some("iso") => "💿",
        Some("dmg") => "🍎",
        Some("msi") => "🪟",

        // 💾 Базы данных
        Some("sql") => "🗄️",
        Some("db") => "💾",

        // 🎵 Аудио
        Some("mp3" | "wav" | "ogg" | "flac") => "🔊",

        // 🎬 Видео
        Some("mp4" | "avi" | "mkv" | "mov") => "🎬",

        // 📄 Документы (офис)
        Some("doc" | "docx") => "📘",
        Some("xls" | "xlsx") => "📊",
        Some("ppt" | "pptx") => "🖉",
        Some("pdf") => "📄",

        // ⚙️ Системные / исполняемые
        Some("exe") => "⚙️",
        Some("dll") => "⚙️",
        Some("so") => "⚙️",
        Some("appimage") => "🚀",
        Some("lock") => "🔒",
        Some("ttf" | "otf") => "🅰️",
        Some("bat") => "🪟",
        Some("cmd") => "🪟",

        // 📄 Все остальное
        _ => "📄",
    }
}
