use iced::widget::{center, column, combo_box, scrollable, text, vertical_space};
use iced::{Center, Fill};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Language {
    Danish,
    Db,
    Dc,
    Dd,
    #[default]
    English,
}

impl Language {
    const ALL: [Language; 5] = [
        Language::Danish,
        Language::Db,
        Language::Dc,
        Language::Dd,
        Language::English
    ];
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Language::Danish => "Danish",
                Language::English => "English",
                Language::Db => "Db",
                Language::Dc => "Dc",
                Language::Dd => "Dd"
            }
        )
    }
}
#[derive(Debug, Clone, Copy)]
enum Message {
    Selected(Language),
}

struct Combo {
    languages: combo_box::State<Language>,
    selected_language: Option<Language>,
    text: String,
}

impl Default for Combo {
    fn default() -> Self {
        Combo {
            languages: combo_box::State::new(Language::ALL.to_vec()),
            selected_language: None,
            text: String::new()
        }
    }
}

impl Combo {

    fn update(&mut self, message: Message) {
        match message {
            Message::Selected(language) => {
                self.selected_language = Some(language); self.text = language.to_string();
            }
        }

    }

    fn view(&self) -> iced::Element<Message> {
        let combo = combo_box(&self.languages,
                              "Type a language..",
                              self.selected_language.as_ref(),
                              Message::Selected);
        let content = column![
            text(&self.text),
            combo,
            vertical_space().height(100)
        ].width(Fill).align_x(Center);
        center(scrollable(content)).into()

    }
}


fn main() -> iced::Result {
    iced::run("Combo Box - Iced", Combo::update, Combo::view)
}
