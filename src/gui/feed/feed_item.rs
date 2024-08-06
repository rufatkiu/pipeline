use gdk::subclass::prelude::ObjectSubclassIsExt;
use gdk_pixbuf::prelude::Cast;
use gtk::{glib::Object, prelude::WidgetExt};
use tf_join::AnyVideo;
use tf_playlist::PlaylistManager;

gtk::glib::wrapper! {
    pub struct FeedItem(ObjectSubclass<imp::FeedItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl FeedItem {
    pub fn new(playlist_manager: PlaylistManager<String, AnyVideo>) -> Self {
        let s: Self = Object::builder::<Self>().build();
        s.imp().playlist_manager.replace(Some(playlist_manager));
        s
    }

    pub fn click(&self) {
        self.imp().handle_clicked();
    }

    fn window(&self) -> crate::gui::window::Window {
        self.root()
            .expect("FeedItem to have root")
            .downcast::<crate::gui::window::Window>()
            .expect("Root to be window")
    }
}

pub mod imp {
    use std::cell::RefCell;

    use adw::prelude::AdwDialogExt;
    use gdk::gio::Cancellable;
    use gdk::gio::SimpleAction;
    use gdk::gio::SimpleActionGroup;
    use gdk::glib::clone;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use glib::ParamSpec;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::UriLauncher;
    use once_cell::sync::Lazy;
    use tf_core::Video;
    use tf_join::AnyVideo;
    use tf_playlist::PlaylistManager;

    use crate::gspawn;
    use crate::gui::feed::feed_item_object::VideoObject;
    use crate::gui::feed::thumbnail::Thumbnail;
    use crate::gui::utility::Utility;
    use crate::gui::video_information_window::video_information_window;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/feed_item.ui")]
    pub struct FeedItem {
        #[template_child]
        label_title: TemplateChild<gtk::Label>,
        #[template_child]
        label_author: TemplateChild<gtk::Label>,
        #[template_child]
        label_platform: TemplateChild<gtk::Label>,
        #[template_child]
        label_date: TemplateChild<gtk::Label>,

        #[template_child]
        thumbnail: TemplateChild<Thumbnail>,

        #[template_child]
        popover_menu: TemplateChild<gtk::PopoverMenu>,

        #[template_child]
        dialog_error: TemplateChild<adw::AlertDialog>,

        video: RefCell<Option<VideoObject>>,
        pub(super) playlist_manager: RefCell<Option<PlaylistManager<String, AnyVideo>>>,
    }

    impl FeedItem {
        fn setup_actions(&self, obj: &super::FeedItem) {
            let action_watch_later = SimpleAction::new("watch-later", None);
            action_watch_later.connect_activate(clone!(
                #[strong(rename_to = video)]
                self.video,
                #[strong(rename_to = playlist_manager)]
                self.playlist_manager,
                move |_, _| {
                    let video = video.borrow().as_ref().and_then(|v| v.video());
                    if let Some(video) = video {
                        let mut playlist_manager = playlist_manager.borrow_mut();
                        playlist_manager
                            .as_mut()
                            .unwrap()
                            .toggle(&"WATCHLATER".to_owned(), &video);
                    }
                }
            ));
            let action_download = SimpleAction::new("download", None);
            action_download.connect_activate(clone!(
                #[weak(rename_to = s)]
                self,
                #[strong(rename_to = video)]
                self.video,
                move |_, _| {
                    let receiver = video
                        .borrow()
                        .as_ref()
                        .expect("Video should be set up")
                        .download();
                    gspawn!(clone!(
                        #[weak]
                        s,
                        async move {
                            if let Err(e) = receiver.await.expect("Video receiver to not fail") {
                                log::error!("Failed to download video: {}", e);
                                let window = s.obj().window();
                                s.dialog_error.present(Some(&window));
                            }
                        }
                    ));
                }
            ));
            let action_open_in_browser = SimpleAction::new("open-in-browser", None);
            action_open_in_browser.connect_activate(clone!(
                #[strong(rename_to = video)]
                self.video,
                #[strong]
                obj,
                move |_, _| {
                    let video_url = &video
                        .borrow()
                        .as_ref()
                        .expect("Video should be set up")
                        .video()
                        .expect("Video should be set up")
                        .url();
                    UriLauncher::new(video_url).launch(
                        Some(&obj.window()),
                        Cancellable::NONE,
                        |_| (),
                    );
                }
            ));
            let action_clipboard = SimpleAction::new("clipboard", None);
            action_clipboard.connect_activate(clone!(
                #[strong(rename_to = video)]
                self.video,
                #[strong]
                obj,
                move |_, _| {
                    let clipboard = obj.display().clipboard();
                    clipboard.set_text(
                        &video
                            .borrow()
                            .as_ref()
                            .expect("Video should be set up")
                            .video()
                            .expect("Video should be set up")
                            .url(),
                    );
                }
            ));
            let action_information = SimpleAction::new("information", None);
            // Currently not sure how to fix that lint.
            #[allow(clippy::await_holding_refcell_ref)]
            action_information.connect_activate(clone!(
                #[strong(rename_to = video)]
                self.video,
                #[strong]
                obj,
                move |_, _| {
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(
                        #[strong]
                        video,
                        #[strong]
                        obj,
                        async move {
                            let video = video.borrow();
                            let video = video.as_ref().expect("Video should be set up");
                            let info = &video.extra_info().await;
                            if let Ok(Some(info)) = info {
                                video_information_window(
                                    video.video().expect("Video to be set up"),
                                    info,
                                    &obj.window(),
                                )
                                .present();
                            }
                        }
                    ));
                }
            ));

            let actions = SimpleActionGroup::new();
            obj.insert_action_group("item", Some(&actions));
            actions.add_action(&action_watch_later);
            actions.add_action(&action_download);
            actions.add_action(&action_open_in_browser);
            actions.add_action(&action_clipboard);
            actions.add_action(&action_information);
        }
    }

    #[gtk::template_callbacks]
    impl FeedItem {
        #[template_callback]
        pub fn handle_clicked(&self) {
            self.popover_menu.popup();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FeedItem {
        const NAME: &'static str = "TFFeedItem";
        type Type = super::FeedItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FeedItem {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecObject::builder::<VideoObject>("video").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "video" => {
                    let value: Option<VideoObject> =
                        value.get().expect("Property video of incorrect type");
                    self.video.replace(value);
                    self.setup_actions(&self.obj());
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "video" => self.video.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for FeedItem {}
    impl BoxImpl for FeedItem {}
}
