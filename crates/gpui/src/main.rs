use std::cell::OnceCell;

use gpui::{
    App, Application, Bounds, Context, Rgba, SharedString, Window, WindowBounds, WindowOptions,
    div, prelude::*, px, rgb, rgba, size,
};
use gpui_component::tab::{Tab, TabBar};

enum Action {
    Parse(usize),
    Stringify(),
    RemoveSpaces(),
    Compare(),
}

#[derive(Copy, Clone)]
struct ColorScheme {
    background: Rgba,
    foreground: Rgba,
    primary: Rgba,
    primary_foreground: Rgba,
    secondary: Rgba,
    secondary_foreground: Rgba,
    accent: Rgba,
    accent_foreground: Rgba,
    card: Rgba,
    card_foreground: Rgba,
    popover: Rgba,
    popover_foreground: Rgba,
    muted: Rgba,
    muted_foreground: Rgba,
    input: Rgba,
    text: Rgba,
    constructive: Rgba,
    destructive: Rgba,
    warning: Rgba,
}

const DARK_COLOR_SCHEME: OnceCell<ColorScheme> = OnceCell::new();
const LIGHT_COLOR_SCHEME: OnceCell<ColorScheme> = OnceCell::new();

struct HelloWorld {
    color_scheme: ColorScheme,
    text: SharedString,
    tab_selected_index: usize,
}

impl Default for HelloWorld {
    fn default() -> Self {
        Self {
            color_scheme: DARK_COLOR_SCHEME
                .get_or_init(|| ColorScheme {
                    background: rgb(0x0f172a),
                    foreground: rgb(0xf1f5f9),
                    primary: rgb(0x93c5fd),
                    primary_foreground: rgb(0x0f172a),
                    secondary: rgb(0x64748b),
                    secondary_foreground: rgb(0xf1f5f9),
                    accent: rgb(0xbde0fe),
                    accent_foreground: rgb(0x0f172a),
                    card: rgb(0x1e293b),
                    card_foreground: rgb(0xf1f5f9),
                    popover: rgb(0x0f172a),
                    popover_foreground: rgb(0xf1f5f9),
                    muted: rgb(0x1e293b),
                    muted_foreground: rgb(0x94a3b8),
                    input: rgb(0x1e293b),
                    text: rgb(0xf1f5f9),
                    constructive: rgb(0x93fdcb),
                    destructive: rgb(0xfca5a5),
                    warning: rgb(0xfab560),
                })
                .clone(),
            text: Default::default(),
            tab_selected_index: 0,
        }
    }
}

fn render_tab(index: usize, title: String) -> impl IntoElement {
    Tab::new().label("Stringify")
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let render_tab = |index: usize, title: &'static str| {
            Tab::new()
                .label(title)
                .bg(if index == self.tab_selected_index {
                    self.color_scheme.primary
                } else {
                    self.color_scheme.muted
                })
                .text_color(self.color_scheme.text)
                .active(|s| s.bg(self.color_scheme.primary))
                .focus(|s| s.bg(self.color_scheme.primary))
        };

        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(self.color_scheme.background)
            .flex_grow()
            .h_full()
            .w_full()
            .shadow_lg()
            .border_1()
            .border_color(self.color_scheme.primary)
            .text_xl()
            .text_color(self.color_scheme.text)
            .child(
                TabBar::new("Actions")
                    .selected_index(self.tab_selected_index)
                    .w_full()
                    .h_10()
                    .bg(self.color_scheme.card)
                    .text_color(self.color_scheme.card_foreground)
                    .on_click(_cx.listener(|view, selected_index, _, _| {
                        view.tab_selected_index = *selected_index;
                    }))
                    .child(render_tab(0, "Parse"))
                    .child(render_tab(1, "Stringify"))
                    .child(render_tab(2, "Remove Spaces"))
                    .child(render_tab(3, "Compare")),
            )
    }
}

fn main() {
    let app = Application::new().with_assets(gpui_component_assets::Assets);

    app.run(|cx: &mut App| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(1500.), px(800.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| HelloWorld::default()),
        )
        .unwrap();
        cx.activate(true);
    });
}
