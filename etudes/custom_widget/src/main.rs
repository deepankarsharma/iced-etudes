use iced::{self, border, Color, Length, Rectangle, Size, Element};
use iced::widget::{scrollable, container};
use iced::advanced::layout::{self, Layout};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::widget::{Tree, Widget};
use iced::advanced::renderer::{self, Style};
use iced::mouse::Cursor;

struct CustomWidget {

}

impl CustomWidget {
    pub fn new() -> Self {
        Self {}
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for CustomWidget
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, _limits: &Limits) -> Node {
        layout::Node::new(Size::new(200., 20000.))
    }

    fn draw(&self, _tree: &Tree, renderer: &mut Renderer, _theme: &Theme, _style: &Style, layout: Layout<'_>, _cursor: Cursor, _viewport: &Rectangle) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: border::rounded(5),
                ..renderer::Quad::default()
            },
            Color::BLACK,
        )
    }
}

fn custom_widget() -> CustomWidget {
    CustomWidget::new()
}


impl<'a, Message, Theme, Renderer> From<CustomWidget> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(buffer_w: CustomWidget) -> Self {
        Self::new(buffer_w)
    }
}

#[derive(Debug)]
struct App {

}

impl Default for App {
    fn default() -> Self {
        App {}
    }
}

impl App {

    fn update(&mut self, _message: Message) {}

    fn view(&self) -> iced::Element<Message> {
        scrollable(container(custom_widget())).into()
    }
}

#[derive(Debug)]
struct Message {

}

fn main() -> iced::Result {

    iced::run("custom_widget", App::update, App::view)
}