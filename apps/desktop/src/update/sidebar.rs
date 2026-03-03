use iced::Task;

use crate::app::App;
use crate::components::sidebar_left::LeftSidebarMode;
use crate::components::sidebar_right::RightSidebarMode;
use crate::message::Message;

pub fn handle_left_sidebar_clicked(app: &mut App, mode: LeftSidebarMode) -> Task<Message> {
    if app.left_sidebar_mode != LeftSidebarMode::Hidden && app.left_sidebar_mode == mode {
        app.left_sidebar_mode = LeftSidebarMode::Hidden;
    } else {
        app.left_sidebar_mode = mode;
    }
    app.rebuild_pane_grid();
    Task::none()
}

pub fn handle_right_sidebar_clicked(app: &mut App, mode: RightSidebarMode) -> Task<Message> {
    if app.right_sidebar_mode != RightSidebarMode::Hidden && app.right_sidebar_mode == mode {
        app.right_sidebar_mode = RightSidebarMode::Hidden;
    } else {
        app.right_sidebar_mode = mode;
    }
    app.rebuild_pane_grid();
    Task::none()
}
