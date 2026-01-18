use iced::{
    Application, Element, Program, Subscription, Task as Command, Theme,
    highlighter::{Highlighter, Settings},
    keyboard,
    widget::text_editor,
};

struct App {
    content: text_editor::Content,
}

#[derive(Debug, Clone)]
enum Message {
    EditInput(text_editor::Action),
}

impl App {
    fn new() -> (Self, Command<Message>) {
        (
            Self {
                content: text_editor::Content::with_text(""),
            },
            Command::none(),
        )
    }

    fn view(&self) -> Element<'_, Message> {
        text_editor(&self.content)
            .placeholder("Type something here...")
            .highlight_with::<Highlighter>(
                Settings {
                    theme: iced::highlighter::Theme::SolarizedDark,
                    token: "json".to_string(),
                },
                |highlight, _theme| highlight.to_format(),
            )
            .on_action(Message::EditInput)
            .into()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EditInput(action) => {
                self.content.perform(action);
                Command::none()
            }
        }
    }

    fn title(&self) -> String {
        String::from("JTool")
    }

    fn subscription(&self) -> Subscription<Message> {
        use keyboard::key;

        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                modifiers,
                ..
            } => None,
            _ => None,
        })
    }
}

pub fn main() -> iced::Result {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    application().run()
}

fn application() -> Application<impl Program<Message = Message, Theme = Theme>> {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title(App::title)
        .window_size((500.0, 800.0))
}
