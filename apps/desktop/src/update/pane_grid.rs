use iced::widget::pane_grid;
use iced::Task;

use crate::app::App;
use crate::message::Message;

pub fn handle_pane_resized(app: &mut App, event: pane_grid::ResizeEvent) -> Task<Message> {
    app.pane_grid_state.resize(event.split, event.ratio);

    if app.left_split == Some(event.split) {
        app.left_ratio = event.ratio;
    }

    if app.right_split == Some(event.split) {
        app.right_ratio = event.ratio;
    }

    Task::none()
}
