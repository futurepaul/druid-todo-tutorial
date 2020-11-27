use druid::{
    widget::TextBox,
    widget::{Button, Checkbox, Flex, Label, List},
    Widget, WidgetExt,
};

use crate::{controllers::TodoItemController, data::*};

fn new_todo_textbox() -> impl Widget<AppState> {
    let new_todo_textbox = TextBox::new()
        .with_placeholder("Add a new todo")
        .expand_width()
        .lens(AppState::new_todo);

    let add_todo_button = Button::new("Add").on_click(AppState::click_add);

    Flex::row()
        .with_flex_child(new_todo_textbox, 1.)
        .with_child(add_todo_button)
}

fn todo_item() -> impl Widget<TodoItem> {
    let checkbox = Checkbox::new("").lens(TodoItem::done);
    let label = Label::raw().lens(TodoItem::text);

    let delete_button = Button::new("Delete").on_click(TodoItem::click_delete);

    Flex::row()
        .with_child(checkbox)
        .with_child(label)
        .with_flex_spacer(1.)
        .with_child(delete_button)
        .controller(TodoItemController)
}

pub fn build_ui() -> impl Widget<AppState> {
    let clear_completed_button = Button::new("Clear completed").on_click(AppState::clear_completed);

    Flex::column()
        .with_child(new_todo_textbox())
        .with_child(List::new(todo_item).lens(AppState::todos))
        .with_flex_spacer(1.)
        .with_child(clear_completed_button)
}
