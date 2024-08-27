use iced::{self, Theme, Color, Background, Border};
use iced::widget::{text, container};
use iced::widget::container::{Style};
use iced::border::{Radius};

#[derive(Debug)]
struct Con {

}

impl Default for Con {
    fn default() -> Self {
        Con {}
    }
}

#[derive(Debug)]
struct Message {

}

impl Con {
    fn view(&self) -> iced::Element<Message> {
        container(text("Hello")).style(container::rounded_box).padding(5).style(|_theme: &Theme| Style {
            text_color: Some(Color::from_rgb8(100, 100, 100)),
            background: Some(Background::from(Color::from_rgb8(222, 110, 100))),
            border: Border { radius: Radius::from(2), width: 2., color: Color::from_rgb8(0,0,0) },
            shadow: Default::default(),
        }).into()
    }

    fn update(&mut self, _message: Message ) {}
}

fn main() ->  iced::Result {
    iced::application("Hello", Con::update, Con::view).theme(|_| {Theme::GruvboxLight}).run()
}
