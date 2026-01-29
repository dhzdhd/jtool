use gpui::{
    App, Application, Bounds, Context, Entity, SharedString, Window, WindowBounds, WindowOptions,
    div, prelude::*, px, size,
};
use gpui_component::button::Button;
use gpui_component::input::{Input, InputState};
use gpui_component::resizable::{h_resizable, resizable_panel};
use gpui_component::switch::Switch;
use gpui_component::tab::{Tab, TabBar};
use gpui_component::{ActiveTheme as _, Root};
use gpui_component::{Theme, ThemeRegistry};
use std::path::PathBuf;

#[derive(Clone)]
struct ParseData {
    prettify: bool,
    input_state: Entity<InputState>,
    output_state: Entity<InputState>,
}

#[derive(Clone)]
struct StringifyData {
    prettify: bool,
    input_state: Entity<InputState>,
    output_state: Entity<InputState>,
}

#[derive(Clone)]
struct RemoveSpacesData {
    prettify: bool,
    input_state: Entity<InputState>,
    output_state: Entity<InputState>,
}

#[derive(Clone)]
struct CompareData {
    prettify: bool,
    input_state: Entity<InputState>,
    output_state: Entity<InputState>,
}

enum Action {
    Parse(ParseData),
    Stringify(StringifyData),
    RemoveSpaces(RemoveSpacesData),
    Compare(CompareData),
}

struct JToolApp {
    tab_selected_index: usize,
    action: Action,
}

impl JToolApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("json")
                .multi_line(true)
                .soft_wrap(false)
                .line_number(false)
                .searchable(false)
                .placeholder(r#"{"hello": "world"}"#)
        });
        let output_state = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("json")
                .multi_line(true)
                .line_number(true)
                .searchable(true)
                .placeholder(r#"{"hello": "world"}"#)
        });

        Self {
            tab_selected_index: 0,
            action: Action::Parse(ParseData {
                prettify: true,
                input_state: input_state,
                output_state: output_state,
            }),
        }
    }
}

impl Render for JToolApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .flex_grow()
            .h_full()
            .w_full()
            .shadow_lg()
            .border_1()
            .text_xl()
            .bg(_cx.theme().background)
            .child(
                TabBar::new("actions")
                    .selected_index(self.tab_selected_index)
                    .w_full()
                    .h_10()
                    .on_click(_cx.listener(|view, selected_index, _, _| {
                        view.tab_selected_index = *selected_index;
                        // match selected_index {
                        // }
                    }))
                    .child(Tab::new().label("Parse").w_1_4())
                    .child(Tab::new().label("Stringify").w_1_4())
                    .child(Tab::new().label("Remove Spaces").w_1_4())
                    .child(Tab::new().label("Compare").w_1_4()),
            )
            .child(match &self.action {
                Action::Parse(data) => div()
                    .flex()
                    .flex_col()
                    .flex_grow()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .items_center()
                            .px_2()
                            .pb_2()
                            .child(
                                Switch::new("prettify-toggle")
                                    .label("Prettify")
                                    .text_color(_cx.theme().foreground)
                                    .checked(data.prettify)
                                    .on_click(_cx.listener(|view, checked, _, _| {
                                        if let Action::Parse(data) = &view.action {
                                            view.action = Action::Parse(ParseData {
                                                prettify: *checked,
                                                ..data.clone()
                                            });
                                        }
                                    })),
                            )
                            .child(Button::new("submit-btn").label("Submit")),
                    )
                    .child(
                        div().flex().flex_row().h_full().child(
                            h_resizable("vertical-layout")
                                .child(resizable_panel().child(Input::new(&data.input_state)))
                                .child(
                                    div()
                                        .pl_1()
                                        .h_full()
                                        .w_full()
                                        .child(Input::new(&data.output_state).h_full())
                                        .into_any_element(),
                                ),
                        ),
                    ),
                _ => div(),
            })
    }
}

pub fn init_theme(cx: &mut App) {
    let theme_name = SharedString::from("Tokyo Storm");
    if let Err(err) =
        ThemeRegistry::watch_dir(PathBuf::from("crates/gpui/assets/themes"), cx, move |cx| {
            if let Some(theme) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
                Theme::global_mut(cx).apply_config(&theme);
            } else {
                tracing::error!("Failed to load theme");
            }
        })
    {
        tracing::error!("Failed to watch themes directory: {}", err);
    }
}

fn main() {
    let app = Application::new().with_assets(gpui_component_assets::Assets);

    app.run(|cx: &mut App| {
        gpui_component::init(cx);
        init_theme(cx);

        let bounds = Bounds::centered(None, size(px(1500.), px(800.0)), cx);
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|cx| JToolApp::new(window, cx));

                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
