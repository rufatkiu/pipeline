use gtk::{
    traits::{GtkWindowExt, WidgetExt},
    Builder,
};
use tf_core::{ExtraVideoInfo, Video};

pub fn video_information_window(
    video: tf_join::AnyVideo,
    video_information: &ExtraVideoInfo,
    parent: &crate::gui::window::Window,
) -> libadwaita::Window {
    log::trace!("Displaying video information: {:#?}", video_information);
    let builder = Builder::from_resource("/ui/video_information_window.ui");
    let window: libadwaita::Window = builder
        .object("window")
        .expect("video_information_window.ui to have the object window");
    window.set_transient_for(Some(parent));
    window.set_modal(true);

    let label_title: gtk::Label = builder
        .object("label_title")
        .expect("video_information_window.ui to have the object label_title");
    label_title.set_text(&video.title());

    if let Some(desc) = &video_information.description {
        let desc = desc.replace("<br>", "\n");
        let desc = desc.replace("&nbsp;", "");
        let desc = desc.replace("&", "&amp;");
        let label_description: gtk::Label = builder
            .object("label_description")
            .expect("video_information_window.ui to have the object label_description");
        label_description.set_markup(&desc);
    }
    if let Some(likes) = &video_information.likes {
        let label_likes: gtk::Label = builder
            .object("label_likes")
            .expect("video_information_window.ui to have the object label_likes");
        label_likes.set_text(&likes.to_string());
    } else {
        let box_likes: gtk::Box = builder
            .object("box_likes")
            .expect("video_information_window.ui to have the object box_likes");
        box_likes.set_visible(false);
    }
    if let Some(dislikes) = &video_information.dislikes {
        let label_dislikes: gtk::Label = builder
            .object("label_dislikes")
            .expect("video_information_window.ui to have the object label_dislikes");
        label_dislikes.set_text(&dislikes.to_string());
    } else {
        let box_dislikes: gtk::Box = builder
            .object("box_dislikes")
            .expect("video_information_window.ui to have the object box_dislikes");
        box_dislikes.set_visible(false);
    }
    if let Some(views) = &video_information.views {
        let label_views: gtk::Label = builder
            .object("label_views")
            .expect("video_information_window.ui to have the object label_views");
        label_views.set_text(&views.to_string());
    } else {
        let box_views: gtk::Box = builder
            .object("box_views")
            .expect("video_information_window.ui to have the object box_views");
        box_views.set_visible(false);
    }

    window
}
