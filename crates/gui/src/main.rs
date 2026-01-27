use iced::Padding;
use iced::keyboard::key::{Code, Physical};
use iced::widget::pane_grid::Configuration;
use iced::widget::text_editor::Binding;
use iced::widget::{button, column, pick_list, row, space, text};
use iced::{
    Application, Element,
    Length::Fill,
    Program, Subscription, Task as Command, Theme,
    highlighter::{Highlighter, Settings},
    keyboard,
    widget::{container, pane_grid, text_editor},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ParseOptions {
    pub is_prettify_checked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StringifyOptions {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompareOptions {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RemoveSpacesOptions {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Parse(ParseOptions),
    Stringify(StringifyOptions),
    Compare(CompareOptions),
    RemoveSpaces(RemoveSpacesOptions),
}

impl Action {
    fn to_json_action(&self) -> JsonAction {
        match self {
            Action::Parse(_) => JsonAction::Parse,
            Action::Stringify(_) => JsonAction::Stringify,
            Action::Compare(_) => JsonAction::Compare,
            Action::RemoveSpaces(_) => JsonAction::RemoveSpaces,
        }
    }
}

pub enum Panes {
    LeftEditorPane,
    RightEditorPane,
}

struct App {
    left_content: text_editor::Content,
    right_content: text_editor::Content,
    panes: pane_grid::State<Panes>,
    action: JsonAction,
}

#[derive(Debug, Clone)]
enum Message {
    EditInput(text_editor::Action),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    JsonActionSelected(JsonAction),
    SubmitPressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsonAction {
    Parse,
    Stringify,
    Compare,
    RemoveSpaces,
}

impl std::fmt::Display for JsonAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Parse => "Parse",
            Self::Stringify => "Stringify",
            Self::Compare => "Compare",
            Self::RemoveSpaces => "RemoveSpaces",
        })
    }
}

impl App {
    fn new() -> (Self, Command<Message>) {
        (
            Self {
                left_content: text_editor::Content::with_text(""),
                right_content: text_editor::Content::with_text(""),
                panes: pane_grid::State::with_configuration(Configuration::Split {
                    axis: pane_grid::Axis::Vertical,
                    ratio: 0.5,
                    a: Box::new(Configuration::Pane(Panes::LeftEditorPane)),
                    b: Box::new(Configuration::Pane(Panes::RightEditorPane)),
                }),
                action: JsonAction::Parse,
            },
            Command::none(),
        )
    }

    fn view(&self) -> Element<'_, Message> {
        let dropdown = pick_list(
            [
                JsonAction::Parse,
                JsonAction::Stringify,
                JsonAction::RemoveSpaces,
                JsonAction::Compare,
            ],
            Some(self.action),
            Message::JsonActionSelected,
        );

        let submit_button = button(text("Submit")).on_press(Message::SubmitPressed);

        let content = pane_grid(
            &self.panes,
            |_pane, pane_state, _is_maximized| match pane_state {
                Panes::LeftEditorPane => self.left_editor_view().into(),
                Panes::RightEditorPane => self.right_editor_view().into(),
            },
        )
        .on_drag(Message::PaneDragged)
        .on_resize(10, Message::PaneResized)
        .spacing(10);

        container(
            column([
                row(match self.action {
                    JsonAction::Parse => vec![
                        dropdown.into(),
                        space::horizontal().into(),
                        submit_button.into(),
                    ],
                    _ => vec![submit_button.into()],
                })
                .into(),
                content.into(),
            ])
            .spacing(10.0),
        )
        .padding(Padding::new(10.0))
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn left_editor_view(&self) -> Element<'_, Message> {
        text_editor(&self.left_content)
            .placeholder("Input")
            .highlight_with::<Highlighter>(
                Settings {
                    theme: iced::highlighter::Theme::SolarizedDark,
                    token: "json".to_string(),
                },
                |highlight, _theme| highlight.to_format(),
            )
            .key_binding(|key| match key.physical_key {
                Physical::Code(code) if code == Code::Enter && key.modifiers.alt() => {
                    Some(Binding::Custom(Message::SubmitPressed))
                }
                _ => Binding::from_key_press(key),
            })
            .on_action(Message::EditInput)
            .into()
    }

    fn right_editor_view(&self) -> Element<'_, Message> {
        text_editor(&self.right_content)
            .placeholder("Output")
            .highlight_with::<Highlighter>(
                Settings {
                    theme: iced::highlighter::Theme::SolarizedDark,
                    token: "json".to_string(),
                },
                |highlight, _theme| highlight.to_format(),
            )
            .into()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EditInput(action) => {
                self.left_content.perform(action);
                Command::none()
            }
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                if ratio >= 0.2 {
                    self.panes.resize(split, ratio);
                }
                Command::none()
            }
            Message::PaneDragged(pane_grid::DragEvent::Dropped { pane, target }) => {
                self.panes.drop(pane, target);
                Command::none()
            }
            Message::JsonActionSelected(action) => {
                self.action = action;
                Command::none()
            }
            Message::SubmitPressed => {
                self.submit();
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn title(&self) -> String {
        String::from("JTool")
    }

    fn submit(&mut self) {
        let result = match self.action {
            JsonAction::Parse => core::parse::parse(self.left_content.text()),
            _ => todo!(),
        };

        match result {
            Ok(val) => {
                self.right_content.perform(text_editor::Action::SelectAll);
                self.right_content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                self.right_content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                        val.to_string().into(),
                    )))
            }
            Err(err) => println!("{err:?}"),
        };
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                modifiers,
                physical_key: Physical::Code(code),
                ..
            } => {
                if code == Code::Enter && modifiers.alt() {
                    Some(Message::SubmitPressed)
                } else {
                    None
                }
            }
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
        .window_size((1500.0, 800.0))
}
