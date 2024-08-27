use iced::{
    alignment, event, mouse, overlay, touch,
    Color, Element, Length, Padding, Rectangle, Size, Theme,
};
use iced::overlay::menu;
use iced::advanced::{Clipboard, Shell};
use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{tree, Tree, Widget};
use iced::widget::{self, text};
use std::cell::RefCell;
use std::fmt::Display;
use std::time::Instant;
use iced::advanced::renderer::{self, Style};

pub trait LazyOptions {
    type Item: Display + Clone;

    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<Self::Item>;
}

pub struct ComboBox<'a, T, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    state: &'a State<T>,
    placeholder: String,
    on_selected: Box<dyn Fn(T::Item) -> Message>,
    on_option_hovered: Option<Box<dyn Fn(T::Item) -> Message>>,
    on_close: Option<Message>,
    on_input: Option<Box<dyn Fn(String) -> Message>>,
    width: Length,
    padding: Padding,
    text_size: Option<f32>,
}

impl<'a, T, Message, Theme, Renderer> ComboBox<'a, T, Message, Theme, Renderer>
where
    T: LazyOptions,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub fn new(
        state: &'a State<T>,
        placeholder: &str,
        on_selected: impl Fn(T::Item) -> Message + 'static,
    ) -> Self {
        Self {
            state,
            placeholder: placeholder.to_string(),
            on_selected: Box::new(on_selected),
            on_option_hovered: None,
            on_close: None,
            on_input: None,
            width: Length::Fill,
            padding: Padding::new(5),
            text_size: None,
        }
    }

    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'static) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    pub fn on_option_hovered(mut self, on_option_hovered: impl Fn(T::Item) -> Message + 'static) -> Self {
        self.on_option_hovered = Some(Box::new(on_option_hovered));
        self
    }

    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = Some(size);
        self
    }
}

#[derive(Debug, Clone)]
pub struct State<T: LazyOptions> {
    options: T,
    inner: RefCell<Inner>,
}

#[derive(Debug, Clone)]
struct Inner {
    value: String,
    filtered_indices: Filtered,
}

#[derive(Debug, Clone)]
struct Filtered {
    indices: Vec<usize>,
    updated: Instant,
}

impl<T: LazyOptions> State<T> {
    pub fn new(options: T) -> Self {
        Self {
            options,
            inner: RefCell::new(Inner {
                value: String::new(),
                filtered_indices: Filtered::new((0..options.len()).collect()),
            }),
        }
    }

    fn value(&self) -> String {
        self.inner.borrow().value.clone()
    }

    fn with_inner<O>(&self, f: impl FnOnce(&Inner) -> O) -> O {
        let inner = self.inner.borrow();
        f(&inner)
    }

    fn with_inner_mut(&self, f: impl FnOnce(&mut Inner)) {
        let mut inner = self.inner.borrow_mut();
        f(&mut inner);
    }
}

impl Filtered {
    fn new(indices: Vec<usize>) -> Self {
        Self {
            indices,
            updated: Instant::now(),
        }
    }

    fn update(&mut self, indices: Vec<usize>) {
        self.indices = indices;
        self.updated = Instant::now();
    }
}

struct Menu {
    menu: menu::State,
    hovered_option: Option<usize>,
    new_selection: Option<usize>,
    filtered_indices: Filtered,
}

impl<'a, T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
for ComboBox<'a, T, Message, Theme, Renderer>
where
    T: LazyOptions,
    Message: Clone,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let padding = self.padding;
        let text_size = self.text_size.unwrap_or(renderer.default_size());

        let limits = limits
            .width(self.width)
            .height(Length::Shrink);

        let mut content = layout::Node::new(limits.resolve(Size::ZERO));
        content.move_to(Point::new(padding.left as f32, padding.top as f32));

        layout::Node::with_children(
            limits.resolve(content.size()),
            vec![content],
        )
    }

    fn on_event(
        &mut self,
        _state: &mut tree::State,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | iced::Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(layout.bounds()) {
                    shell.publish(event::Status::Captured);
                    return event::Status::Captured;
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        _state: &widget::tree::State,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let text_size = self.text_size.unwrap_or(renderer.default_size());

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: 0.0.into(),
                border_width: 1.0,
                border_color: theme.border_color(style.border_color),
            },
            theme.background_color(style.background_color),
        );

        let value = self.state.value();
        let content = if value.is_empty() {
            &self.placeholder
        } else {
            &value
        };

        renderer.fill_text(
            text::Text {
                content,
                size: text_size,
                color: if value.is_empty() {
                    theme.placeholder_color()
                } else {
                    theme.text_color()
                },
                font: renderer.default_font(),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
            },
            bounds,
        );
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Menu>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Menu {
            menu: menu::State::default(),
            filtered_indices: Filtered::new(Vec::new()),
            hovered_option: None,
            new_selection: None,
        })
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut tree::State,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let menu = state.downcast_mut::<Menu>();

        self.state.with_inner(|inner| {
            if inner.filtered_indices.indices.is_empty() {
                None
            } else {
                let bounds = layout.bounds();

                Some(
                    menu::Menu::new(
                        &mut menu.menu,
                        &inner.filtered_indices.indices,
                        &menu.hovered_option,
                        |index| {
                            if let Some(item) = self.state.options.get(index) {
                                (self.on_selected)(item)
                            } else {
                                // Handle the case where the item is not found
                                Message::default() // You might want to define a default behavior
                            }
                        },
                    )
                        .width(bounds.width)
                        .padding(self.padding)
                        .overlay(layout.position(), bounds.height)
                )
            }
        })
    }
}

impl<'a, T, Message, Theme, Renderer> From<ComboBox<'a, T, Message, Theme, Renderer>>
for Element<'a, Message, Theme, Renderer>
where
    T: LazyOptions,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(combo_box: ComboBox<'a, T, Message, Theme, Renderer>) -> Self {
        Self::new(combo_box)
    }
}

pub trait Catalog: menu::Catalog {
    fn border_color(&self, _style: &renderer::Style) -> Color {
        Color::BLACK
    }

    fn background_color(&self, _style: &renderer::Style) -> Color {
        Color::WHITE
    }

    fn text_color(&self) -> Color {
        Color::BLACK
    }

    fn placeholder_color(&self) -> Color {
        Color::from_rgb(0.7, 0.7, 0.7)
    }
}

impl Catalog for Theme {}

fn search<'a, T: LazyOptions>(
    options: &'a T,
    query: &'a str,
) -> impl Iterator<Item = usize> + 'a {
    let query = query.to_lowercase();
    (0..options.len()).filter(move |&index| {
        options.get(index).map_or(false, |item| {
            item.to_string().to_lowercase().contains(&query)
        })
    })
}

fn main() {
    println!("Hello, world!");
}
