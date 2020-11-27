use druid::{AppLauncher, WindowDesc};

mod data;
use data::{AppState, TodoItem};

mod view;
use view::build_ui;

pub fn main() {
    let main_window = WindowDesc::new(build_ui)
        .title("Todo Tutorial")
        .window_size((400.0, 400.0));

    let todos = vec![TodoItem::new("thing one"), TodoItem::new("thing two")];
    let initial_state = AppState::new(todos);

    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}
