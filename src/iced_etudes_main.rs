use iced::event::{self, Event};
use iced::widget::{button, center, checkbox, text, text_input, Column};
use iced::window;
use iced::{Center, Element, Fill, Subscription, Task};

pub fn main() -> iced::Result {
    iced::application("Events - Iced", Events::update, Events::view)
        .subscription(Events::subscription)
        .exit_on_close_request(false)
        .run()
}

#[derive(Debug, Default)]
struct Events {
    last: Vec<Event>,
    enabled: bool,
    input_value: String,
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(Event),
    Toggled(bool),
    Exit,
    InputChanged(String),
}

impl Events {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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

        let mut text_input = text_input("Type something to continue...", &self.input_value)
            .on_input(Message::InputChanged)
            .padding(10)
            .width(600);
        let content = Column::new()
            .align_x(Center)
            .spacing(20)
            .push(events)
            .push(toggle)
            .push(text_input)
            .push(exit);

        center(content).into()
    }
}
