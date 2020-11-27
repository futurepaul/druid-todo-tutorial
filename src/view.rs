use druid::{widget::Label, Widget};

use crate::data::*;

pub fn build_ui() -> impl Widget<AppState> {
    Label::new("Hello")
}
