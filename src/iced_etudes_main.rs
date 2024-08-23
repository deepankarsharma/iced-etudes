use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::event::{self, Event};
use iced::widget::text_editor::{Action, Edit, Motion};
use iced::widget::{button, center, checkbox, text, text_editor, text_input, Column};
use iced::window;
use iced::{border, Center, Color, Element, Fill, Length, Rectangle, Size, Subscription, Task};
use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::io::{self, Error, ErrorKind, Read, Write};

// ---------------- DATA MODELS ------------------------------------
// Since we are dealing with a Text Editor the most central datatype
// we have to deal with is a Buffer is the data structure that will
// back all the visible text windows.

// Buffers can be of following types:
// - inmemory buffer that points to some memory span in memory
// - a mmaped file buffer
// - a virtualized buffer that can show infinite datasets

trait Buffer {
    fn insert(&mut self, position: usize, text: &str) -> io::Result<()>;
    fn delete(&mut self, start: usize, end: usize) -> io::Result<()>;
    fn get_slice(&self, start: usize, end: usize) -> io::Result<String>;
    fn len(&self) -> usize;
}

// But to start with we will only be dealing with mmap-ed buffers
// as our only buffer type.
pub struct MmapBuffer {
    mmap: MmapMut,
}

pub fn main() -> iced::Result {
    iced::application("Events - Iced", Events::update, Events::view)
        .subscription(Events::subscription)
        .exit_on_close_request(false)
        .run()
}

impl MmapBuffer {
    pub fn new(path: &str, size: usize) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        file.set_len(size as u64)?;
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        Ok(MmapBuffer { mmap })
    }
}

impl Buffer for MmapBuffer {
    fn insert(&mut self, position: usize, text: &str) -> io::Result<()> {
        if position > self.mmap.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Position out of bounds",
            ));
        }
        let end = position + text.len();
        if end > self.mmap.len() {
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "Not enough space",
            ));
        }
        self.mmap[position..end].copy_from_slice(text.as_bytes());
        Ok(())
    }

    fn delete(&mut self, start: usize, end: usize) -> io::Result<()> {
        if start > end || end > self.mmap.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid range"));
        }
        let mmap_len = self.mmap.len();
        let remaining = self.mmap.len() - end;
        self.mmap.copy_within(end..mmap_len, start);
        self.mmap[start + remaining..].fill(0);
        Ok(())
    }

    fn get_slice(&self, start: usize, end: usize) -> io::Result<String> {
        if start > end || end > self.mmap.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid range"));
        }
        String::from_utf8(self.mmap[start..end].to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn len(&self) -> usize {
        self.mmap.len()
    }
}

pub struct EmptyBuffer {}

impl EmptyBuffer {
    pub fn new() -> Self {
        EmptyBuffer {}
    }
}

impl Buffer for EmptyBuffer {
    fn insert(&mut self, _position: usize, _text: &str) -> io::Result<()> {
        todo!()
    }

    fn delete(&mut self, _start: usize, _end: usize) -> io::Result<()> {
        todo!()
    }

    fn get_slice(&self, _start: usize, _end: usize) -> io::Result<String> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }
}

// ------------- Custom Widgets -----------------------------

#[allow(missing_debug_implementations)]
pub struct BufferWidget {
    buffer: Box<dyn Buffer>,
}

impl BufferWidget {
    pub fn new() -> Self {
        Self {
            buffer: Box::new(EmptyBuffer::new()),
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for BufferWidget
where
    Renderer: renderer::Renderer,
{
    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: border::rounded(5),
                ..renderer::Quad::default()
            },
            Color::BLACK,
        )
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(200., 200.))
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }
}

pub fn buffer_widget() -> BufferWidget {
    BufferWidget::new()
}

impl<'a, Message, Theme, Renderer> From<BufferWidget> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(buffer_w: BufferWidget) -> Self {
        Self::new(buffer_w)
    }
}

//------------- Application ---------------------------------

#[derive(Debug, Default)]
struct Events {
    last: Vec<Event>,
    enabled: bool,
    input_value: String,
    text_editor_content: text_editor::Content,
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(Event),
    Toggled(bool),
    Exit,
    InputChanged(String),
    TextEditorAction(text_editor::Action),
}

impl Events {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TextEditorAction(action) => {
                self.text_editor_content.perform(action);
                Task::none()
            }
            Message::EventOccurred(event) if self.enabled => {
                self.last.push(event);

                if self.last.len() > 5 {
                    let _ = self.last.remove(0);
                }

                Task::none()
            }
            Message::EventOccurred(event) => {
                if let Event::Window(window::Event::CloseRequested) = event {
                    window::get_latest().and_then(window::close)
                } else {
                    Task::none()
                }
            }
            Message::InputChanged(input_value) => {
                self.input_value = input_value;
                Task::none()
            }
            Message::Toggled(enabled) => {
                self.enabled = enabled;

                Task::none()
            }
            Message::Exit => window::get_latest().and_then(window::close),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }

    fn view(&self) -> Element<Message> {
        let events = Column::with_children(
            self.last
                .iter()
                .map(|event| text!("{event:?}").size(40))
                .map(Element::from),
        );

        let toggle = checkbox("Listen to runtime events", self.enabled).on_toggle(Message::Toggled);

        let exit = button(text("Exit").width(Fill).align_x(Center))
            .width(100)
            .padding(10)
            .on_press(Message::Exit);

        let text_input = text_input("Type something to continue...", &self.input_value)
            .on_input(Message::InputChanged)
            .padding(10)
            .width(600);

        let text_editor = text_editor(&self.text_editor_content)
            .height(Fill)
            .on_action(Message::TextEditorAction);

        let buffer_w = buffer_widget();

        let content = Column::new()
            .align_x(Center)
            .spacing(20)
            .push(events)
            .push(toggle)
            .push(buffer_w)
            .push(text_editor)
            .push(text_input)
            .push(exit);

        center(content).into()
    }
}
