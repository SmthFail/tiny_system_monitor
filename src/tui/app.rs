use super::app_error::AppError;
use super::buffer::{Buffer, Cell, Color, Patch};
use super::widget::Rect;
use crate::tui::widget::Widget; // ВАЖНО: трейт в скоуп, чтобы вызывать render()/update()

use crossterm::{
    cursor,
    execute,
    queue,
    style::{Color as CrossColor, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
    terminal::{Clear, ClearType},
};
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use std::io::{stdout, Stdout, Write};
use std::mem;
use std::time::Duration;

use crate::tui::cell_types::CellString;
use crate::tui::engine::{
    layout_equal_split, paint, reconcile, update_tree, Direction, Element, Key, VNode,
};
use crate::{get_version, TextWidget};
use crate::tui::engine::IntoNode; // для app.add_child(...)
use crate::tui::widgets::dialog_widget::DialogWidget;

struct ErrorRow {
    visible: bool,
    message: CellString,
    ui: Box<TextWidget>,
}

impl ErrorRow {
    fn new() -> Self {
        let message = CellString::new();
        let row =
            TextWidget::new_editable(message.clone()).set_color(Some(Color::White), Some(Color::Yellow));
        let ui = Box::new(row);

        ErrorRow { visible: false, message, ui }
    }

    fn set_message(&mut self, message: String) {
        self.visible = true;
        self.message.update(message);
    }

    fn render(&mut self, buffer: &mut Buffer, area: Rect) -> Result<(), AppError> {
        if self.visible {
            self.ui.render(buffer, area)?;
        }
        Ok(())
    }

    fn clear(&mut self) {
        self.message.clear();
        self.visible = false;
    }
}

pub struct App {
    pub buffer: Buffer,
    prev_buffer: Buffer,
    stdout: Stdout,

    // строки статуса/ошибок
    error_row: Box<ErrorRow>,
    status_row: Box<TextWidget>,

    // декларативное описание (VNode)
    vnodes: Vec<VNode>,
    // живое дерево между кадрами (для reconcile)
    root_elem: Option<Element>,

    // диалоговое окно
    dialog: Option<DialogWidget>,
}

impl App {
    pub fn new() -> Result<Self, AppError> {
        let (width, height) =
            terminal::size().map_err(|e| AppError::error(format!("Can't get terminal size {}", e)))?;

        let version = get_version();
        let status_string = format!("q: exit, ver:{:?}", version);
        let status_row =
            TextWidget::new_static(&status_string).set_color(Some(Color::White), Some(Color::Blue));

        Ok(App {
            buffer: Buffer::new(width, height),
            prev_buffer: Buffer::new(width, height),
            stdout: stdout(),
            status_row: Box::new(status_row),
            error_row: Box::new(ErrorRow::new()),
            vnodes: vec![],
            root_elem: None,
            dialog: None,
        })
    }

    pub fn add_child<N: IntoNode>(&mut self, child: N) {
        self.vnodes.push(child.into_node());
    }

    pub fn show_dialog(&mut self, title: &str, message: &str) {
        let mut dialog = DialogWidget::new(title, message);
        dialog.show();
        self.dialog = Some(dialog);
    }

    pub fn hide_dialog(&mut self) {
        if let Some(ref mut dialog) = self.dialog {
            dialog.hide();
        }
    }

    pub fn toggle_help_dialog(&mut self) {
        if self.dialog.is_none() {
            self.dialog = Some(DialogWidget::new("Help", "This is help"));
        }

        if let Some(ref mut dialog) = self.dialog {
            dialog.toggle();
        }
    }

    /// Отладочный вывод виртуального дерева (для `--tree`)
    pub fn debug_print_tree(&self) {
        fn walk(nodes: &[VNode], depth: usize) {
            for n in nodes {
                let indent = "  ".repeat(depth);
                match n {
                    VNode::Container { key, dir, children } => {
                        eprintln!(
                            "{}Container key={:?} dir={:?}",
                            indent,
                            key.as_ref().map(|k| &k.0),
                            dir
                        );
                        walk(children, depth + 1);
                    }
                    VNode::LeafInstance { key, .. } => {
                        eprintln!("{}Leaf key={}", indent, key.0);
                    }
                }
            }
        }
        eprintln!("--- VNode tree ---");
        walk(&self.vnodes, 0);
        eprintln!("------------------");
    }

    pub fn update(&mut self) -> Result<(), AppError> {
        // обновляем только status_row здесь; дерево — в update_tree() после layout
        self.status_row.update()?;
        Ok(())
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.buffer.resize(width, height);
        self.prev_buffer.resize(width, height);
        let _ = execute!(self.stdout, ResetColor, Clear(ClearType::All), cursor::MoveTo(0, 0));
    }

    pub fn render(&mut self) -> Result<(), AppError> {
        let buffer_width = self.buffer.width;
        let buffer_height = self.buffer.height;
        let body_height = if self.error_row.visible {
            buffer_height - 2
        } else {
            buffer_height - 1
        };

        // корневой контейнер виртуального дерева
        let root_vnode = VNode::Container {
            key: Some(Key("root".into())),
            dir: Direction::Vertical,
            children: self.vnodes.clone(),
        };

        // первый кадр — mount; далее — reconcile
        if let Some(elem) = &mut self.root_elem {
            reconcile(elem, &root_vnode);
        } else {
            self.root_elem = Some(crate::tui::engine::mount(&root_vnode));
        }
        let elem = self.root_elem.as_mut().unwrap();

        // layout -> update (живых виджетов) -> paint
        layout_equal_split(
            elem,
            Rect { x: 0, y: 0, width: buffer_width, height: body_height },
        );

        if let Err(e) = update_tree(elem) {
            self.error_row.set_message(e.message);
        }

        match paint(&mut self.buffer, elem) {
            Ok(_) => self.error_row.clear(),
            Err(e) => self.error_row.set_message(e.message),
        }

        // error row
        let _ = self.error_row.render(
            &mut self.buffer,
            Rect { x: 0, y: body_height, width: buffer_width, height: 1 },
        );

        // status row
        let status_y = buffer_height - 1;
        self.status_row.render(
            &mut self.buffer,
            Rect { x: 0, y: status_y, width: buffer_width, height: 1 },
        )?;

        // render dialog on top if visible
        if let Some(ref mut dialog) = self.dialog {
            if dialog.is_visible() {
                let dialog_area = Rect {
                    x: 0,
                    y: 0,
                    width: buffer_width,
                    height: buffer_height
                };
                dialog.render(&mut self.buffer, dialog_area)?;
            }
        }

        Ok(())
    }

    fn match_cross_color(color: &Color) -> CrossColor {
        match color {
            Color::Red => CrossColor::Red,
            Color::Green => CrossColor::Green,
            Color::Blue => CrossColor::Blue,
            Color::White => CrossColor::White,
            Color::Black => CrossColor::Black,
            Color::Yellow => CrossColor::DarkYellow,
        }
    }

    fn flush_to_terminal(&mut self) -> crossterm::Result<()> {
        queue!(self.stdout, ResetColor)?;

        let mut cur_fg: Option<Color> = None;
        let mut cur_bg: Option<Color> = None;

        let patches = self.prev_buffer.get_diff(&self.buffer);
        for Patch { cell: Cell { ch, fg, bg }, x, y } in patches {
            queue!(self.stdout, cursor::MoveTo(x as u16, y as u16))?;

            if fg != cur_fg {
                match &fg {
                    Some(color) => queue!(
                        self.stdout,
                        SetForegroundColor(Self::match_cross_color(&color))
                    )?,
                    None => queue!(self.stdout, SetForegroundColor(CrossColor::Reset))?,
                }
                cur_fg = fg.clone();
            }

            if bg != cur_bg {
                match &bg {
                    Some(color) => queue!(
                        self.stdout,
                        SetBackgroundColor(Self::match_cross_color(&color))
                    )?,
                    None => queue!(self.stdout, SetBackgroundColor(CrossColor::Reset))?,
                }
                cur_bg = bg.clone();
            }

            queue!(self.stdout, Print(ch))?;
        }

        self.stdout.flush()?;
        mem::swap(&mut self.buffer, &mut self.prev_buffer);
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        execute!(self.stdout, terminal::EnterAlternateScreen, cursor::Hide)
            .map_err(|e| AppError::error(format!("Can't enter alternate screen: {}", e)))?;
        terminal::enable_raw_mode()
            .map_err(|e| AppError::error(format!("Can't enable_raw_mode: {}", e)))?;

        loop {
            self.update()?;
            self.render()?;
            let _ = self.flush_to_terminal();

            if poll(Duration::from_millis(500))
                .map_err(|e| AppError::error(format!("Poll error: {}", e)))?
            {
                let event = read().map_err(|e| AppError::error(format!("Event read error: {}", e)))?;
                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    }) => break,
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('h'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    }) => self.toggle_help_dialog(),
                    Event::Resize(width, height) => self.resize(width, height),
                    _ => (),
                }
            }
        }

        terminal::disable_raw_mode()
            .map_err(|e| AppError::error(format!("Can't disable raw mode: {}", e)))?;
        execute!(self.stdout, terminal::LeaveAlternateScreen, cursor::Show)
            .map_err(|e| AppError::error(format!("Can't leave alternate screen: {}", e)))?;
        Ok(())
    }
}

