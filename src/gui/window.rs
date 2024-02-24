/*
 * Copyright 2021 - 2022 Julian Schmidhuber <github@schmiddi.anonaddy.com>
 *
 * This file is part of Pipeline.
 *
 * Pipeline is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Pipeline is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Pipeline.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use gdk::subclass::prelude::ObjectSubclassIsExt;
use gdk_pixbuf::prelude::SettingsExt;
use gtk::prelude::GtkWindowExt;
use gtk::{glib::Object, prelude::WidgetExt};
use adw::prelude::GtkApplicationExt;

fn setup_joiner() -> tf_join::Joiner {
    let joiner = tf_join::Joiner::new();
    joiner
}

gtk::glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, adw::Window, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &gtk::Application) -> Self {
        app.set_accels_for_action("win.settings", &["<Control>comma"]);
        app.set_accels_for_action("win.about", &["F1"]);
        app.set_accels_for_action("win.show-help-overlay", &["<Control>question"]);
        app.set_accels_for_action("window.close", &["<Control>q"]);

        app.set_accels_for_action("feed.watch-later", &["<Control>w"]);
        app.set_accels_for_action("feed.download", &["<Control>s"]);
        app.set_accels_for_action("feed.open_in_browser", &["<Control>b"]);
        app.set_accels_for_action("feed.clipboard", &["<Control>c"]);
        app.set_accels_for_action("feed.information", &["<Control>i"]);

        app.set_accels_for_action("win.feed", &["<Control>1"]);
        app.set_accels_for_action("win.watch-later", &["<Control>2"]);
        app.set_accels_for_action("win.filters", &["<Control>3"]);
        app.set_accels_for_action("win.subscriptions", &["<Control>4"]);

        Object::builder::<Self>()
            .property("title", &gettextrs::gettext("Pipeline"))
            .property("application", app)
            .build()
    }

    pub fn reload(&self) {
        let _ = self.activate_action("win.reload", None);
    }

    fn save_window_size(&self) -> Result<(), gtk::glib::BoolError> {
        let imp = self.imp();

        let (width, height) = self.default_size();

        imp.settings.set_int("window-width", width)?;
        imp.settings.set_int("window-height", height)?;

        imp.settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let imp = self.imp();

        let width = imp.settings.int("window-width");
        let height = imp.settings.int("window-height");
        let is_maximized = imp.settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }
}

pub mod imp {
    use crate::config::{APP_ID, PROFILE};
    use crate::gui::import_window;
    use crate::gui::predefined_player::PredefinedPlayer;
    use crate::gui::preferences_window::PreferencesWindow;

    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use gdk_pixbuf::gio::{SimpleAction, SimpleActionGroup};
    use gdk_pixbuf::glib::clone;
    use glib::subclass::InitializingObject;
    use gtk::glib::Propagation;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Builder};
    use gtk::{prelude::*, ShortcutsWindow};

    use gtk::CompositeTemplate;
    use adw::subclass::prelude::AdwApplicationWindowImpl;
    use adw::subclass::prelude::AdwWindowImpl;
    use adw::AboutWindow;

    use tf_filter::FilterEvent;
    use tf_join::AnySubscriptionList;
    use tf_join::AnyVideo;
    use tf_join::AnyVideoFilter;
    use tf_join::Joiner;
    use tf_join::SubscriptionEvent;
    use tf_observer::Observable;
    use tf_observer::Observer;
    use tf_playlist::PlaylistEvent;
    use tf_playlist::PlaylistManager;

    use crate::csv_file_manager::CsvFileManager;
    use crate::gui::feed::feed_page::FeedPage;
    use crate::gui::filter::filter_page::FilterPage;
    use crate::gui::subscription::subscription_page::SubscriptionPage;
    use crate::gui::watch_later::WatchLaterPage;

    use super::setup_joiner;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/window.ui")]
    pub struct Window {
        #[template_child]
        pub(in crate::gui) application_stack: TemplateChild<adw::ViewStack>,

        #[template_child]
        pub(in crate::gui) switcher_bar: TemplateChild<adw::ViewSwitcherBar>,

        pub settings: gtk::gio::Settings,

        #[template_child]
        pub(super) feed_page: TemplateChild<FeedPage>,
        #[template_child]
        pub(super) watchlater_page: TemplateChild<WatchLaterPage>,
        #[template_child]
        pub(super) filter_page: TemplateChild<FilterPage>,
        #[template_child]
        pub(super) subscription_page: TemplateChild<SubscriptionPage>,

        pub(in crate::gui) joiner: RefCell<Option<Joiner>>,
        playlist_manager: RefCell<Option<PlaylistManager<String, AnyVideo>>>,
        any_subscription_list: RefCell<Option<AnySubscriptionList>>,
        _watchlater_file_manager:
            RefCell<Option<Arc<Mutex<Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>>>>>,
        _subscription_file_manager:
            RefCell<Option<Arc<Mutex<Box<dyn Observer<SubscriptionEvent> + Send>>>>>,
        _filter_file_manager:
            RefCell<Option<Arc<Mutex<Box<dyn Observer<FilterEvent<AnyVideoFilter>> + Send>>>>>,
    }

    impl Default for Window {
        fn default() -> Self {
            Self {
                settings: gtk::gio::Settings::new(APP_ID),
                application_stack: Default::default(),
                switcher_bar: Default::default(),
                feed_page: Default::default(),
                watchlater_page: Default::default(),
                filter_page: Default::default(),
                subscription_page: Default::default(),
                joiner: Default::default(),
                playlist_manager: Default::default(),
                any_subscription_list: Default::default(),
                _watchlater_file_manager: Default::default(),
                _subscription_file_manager: Default::default(),
                _filter_file_manager: Default::default(),
            }
        }
    }

    impl Window {
        fn setup_actions(&self, obj: &super::Window) {
            let action_settings = SimpleAction::new("settings", None);
            action_settings.connect_activate(clone!(@weak obj => move |_, _| {
                let settings = PreferencesWindow::new();
                settings.set_transient_for(Some(&obj));
                settings.present();
            }));
            let action_import = SimpleAction::new("import", None);
            action_import.connect_activate(clone!(@weak obj => move |_, _| {
                let import = import_window::import_window(obj.imp().joiner.borrow().clone().expect("Joiner to be set up"), &obj);
                import.present();
            }));

            let action_about = SimpleAction::new("about", None);
            action_about.connect_activate(clone!(@weak obj => move |_, _| {
                let builder = Builder::from_resource("/ui/about.ui");
                let about: AboutWindow = builder
                    .object("about")
                    .expect("about.ui to have at least one object about");
                about.add_link(
                    &gettextrs::gettext("Donate"),
                    "https://gitlab.com/schmiddi-on-mobile/pipeline#donate",
                );
                about.set_transient_for(Some(&obj));
                about.present();
            }));
            let action_show_help_overlay = SimpleAction::new("show-help-overlay", None);
            action_show_help_overlay.connect_activate(|_, _| {
                let builder = Builder::from_resource("/ui/shortcuts.ui");
                let shortcuts_window: ShortcutsWindow = builder
                    .object("help_overlay")
                    .expect("shortcuts.ui to have at least one object help_overlay");
                shortcuts_window.present();
            });

            let action_feed = SimpleAction::new("feed", None);
            action_feed.connect_activate(clone!(@weak obj => move |_, _| {
                obj.imp().application_stack.set_visible_child(&obj.imp().feed_page.get());
            }));
            let action_watch_later = SimpleAction::new("watch-later", None);
            action_watch_later.connect_activate(clone!(@weak obj => move |_, _| {
                obj.imp().application_stack.set_visible_child(&obj.imp().watchlater_page.get());
            }));
            let action_filters = SimpleAction::new("filters", None);
            action_filters.connect_activate(clone!(@weak obj => move |_, _| {
                obj.imp().application_stack.set_visible_child(&obj.imp().filter_page.get());
            }));
            let action_subscriptions = SimpleAction::new("subscriptions", None);
            action_subscriptions.connect_activate(clone!(@weak obj => move |_, _| {
                obj.imp().application_stack.set_visible_child(&obj.imp().subscription_page.get());
            }));

            let actions = SimpleActionGroup::new();
            obj.insert_action_group("win", Some(&actions));
            actions.add_action(&action_import);
            actions.add_action(&action_settings);
            actions.add_action(&action_show_help_overlay);
            actions.add_action(&action_about);
            actions.add_action(&action_feed);
            actions.add_action(&action_watch_later);
            actions.add_action(&action_filters);
            actions.add_action(&action_subscriptions);

            let action_feed_watch_later = SimpleAction::new("watch-later", None);
            action_feed_watch_later.connect_activate(clone!(@weak obj => move |_, _| {
                let stack = &obj.imp().application_stack;
                let child = stack.visible_child();

                if let Some(page) = child.and_dynamic_cast_ref::<FeedPage>() {
                    page.emit_watch_later();
                } else if let Some(page) = child.and_dynamic_cast_ref::<WatchLaterPage>() {
                    page.emit_watch_later();
                } else if let Some(page) = child.and_dynamic_cast_ref::<SubscriptionPage>() {
                    page.emit_watch_later();
                }
            }));
            let action_feed_download = SimpleAction::new("download", None);
            action_feed_download.connect_activate(clone!(@weak obj => move |_, _| {
                let stack = &obj.imp().application_stack;
                let child = stack.visible_child();

                if let Some(page) = child.and_dynamic_cast_ref::<FeedPage>() {
                    page.emit_download();
                } else if let Some(page) = child.and_dynamic_cast_ref::<WatchLaterPage>() {
                    page.emit_download();
                } else if let Some(page) = child.and_dynamic_cast_ref::<SubscriptionPage>() {
                    page.emit_download();
                }
            }));
            let action_feed_copy_to_clipboard = SimpleAction::new("clipboard", None);
            action_feed_copy_to_clipboard.connect_activate(clone!(@weak obj => move |_, _| {
                let stack = &obj.imp().application_stack;
                let child = stack.visible_child();

                if let Some(page) = child.and_dynamic_cast_ref::<FeedPage>() {
                    page.emit_copy_to_clipboard();
                } else if let Some(page) = child.and_dynamic_cast_ref::<WatchLaterPage>() {
                    page.emit_copy_to_clipboard();
                } else if let Some(page) = child.and_dynamic_cast_ref::<SubscriptionPage>() {
                    page.emit_copy_to_clipboard();
                }
            }));
            let action_feed_open_in_browser = SimpleAction::new("open_in_browser", None);
            action_feed_open_in_browser.connect_activate(clone!(@weak obj => move |_, _| {
                let stack = &obj.imp().application_stack;
                let child = stack.visible_child();

                if let Some(page) = child.and_dynamic_cast_ref::<FeedPage>() {
                    page.emit_open_in_browser();
                } else if let Some(page) = child.and_dynamic_cast_ref::<WatchLaterPage>() {
                    page.emit_open_in_browser();
                } else if let Some(page) = child.and_dynamic_cast_ref::<SubscriptionPage>() {
                    page.emit_open_in_browser();
                }
            }));
            let action_feed_information = SimpleAction::new("information", None);
            action_feed_information.connect_activate(clone!(@weak obj => move |_, _| {
                let stack = &obj.imp().application_stack;
                let child = stack.visible_child();

                if let Some(page) = child.and_dynamic_cast_ref::<FeedPage>() {
                    page.emit_information();
                } else if let Some(page) = child.and_dynamic_cast_ref::<WatchLaterPage>() {
                    page.emit_information();
                } else if let Some(page) = child.and_dynamic_cast_ref::<SubscriptionPage>() {
                    page.emit_information();
                }
            }));

            let actions_feed = SimpleActionGroup::new();
            obj.insert_action_group("feed", Some(&actions_feed));
            actions_feed.add_action(&action_feed_watch_later);
            actions_feed.add_action(&action_feed_download);
            actions_feed.add_action(&action_feed_open_in_browser);
            actions_feed.add_action(&action_feed_copy_to_clipboard);
            actions_feed.add_action(&action_feed_information);
        }

        fn setup_feed(&self) {
            self.feed_page.connect_local(
                "go-to-subscriptions",
                true,
                clone!(@strong self.application_stack as stack, @strong self.subscription_page as s => move |_| {
                    stack.set_visible_child(&s);
                    None
                }),
            );
        }
        fn setup_watch_later(&self) {
            let joiner = setup_joiner();
            self.joiner.replace(Some(joiner.clone()));

            let mut watchlater_file_path = glib::user_data_dir();
            watchlater_file_path.push("tubefeeder");
            watchlater_file_path.push("playlist_watch_later.csv");

            let mut playlist_manager = PlaylistManager::new();
            let mut playlist_manager_clone = playlist_manager.clone();

            let _watchlater_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
                &watchlater_file_path,
                &mut move |v| {
                    let join_video = joiner.upgrade_video(&v);
                    playlist_manager_clone.toggle(&"WATCHLATER".to_string(), &join_video);
                },
            ))
                as Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>));

            playlist_manager.attach_at(
                Arc::downgrade(&_watchlater_file_manager),
                &"WATCHLATER".to_string(),
            );

            self.playlist_manager
                .replace(Some(playlist_manager.clone()));
            self._watchlater_file_manager
                .replace(Some(_watchlater_file_manager));
            self.watchlater_page
                .get()
                .set_playlist_manager(playlist_manager);
        }

        fn setup_subscriptions(&self) {
            let joiner = self
                .joiner
                .borrow()
                .clone()
                .expect("Joiner should be set up");

            let mut subscription_list = joiner.subscription_list();

            let mut user_data_dir = gtk::glib::user_data_dir();
            user_data_dir.push("tubefeeder");

            let mut subscriptions_file_path = user_data_dir.clone();
            subscriptions_file_path.push("subscriptions.csv");

            let _subscription_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
                &subscriptions_file_path,
                &mut |sub| subscription_list.add(sub),
            ))
                as Box<dyn Observer<SubscriptionEvent> + Send>));

            subscription_list.attach(Arc::downgrade(&_subscription_file_manager));

            self.any_subscription_list
                .replace(Some(subscription_list.clone()));
            self._subscription_file_manager
                .replace(Some(_subscription_file_manager));
            self.subscription_page.get().set_subscription_list(
                subscription_list.clone(),
                self.playlist_manager
                    .borrow()
                    .clone()
                    .expect("PlaylistManager should be set up"),
            );
            self.feed_page.get().setup(
                self.playlist_manager
                    .borrow()
                    .clone()
                    .expect("PlaylistManager should be set up"),
                joiner,
            );

            self.subscription_page.connect_local(
                "subscription-added",
                true,
                clone!(@strong self.feed_page as f => move |_| {
                    f.reload();
                    None
                }),
            );
        }

        fn setup_filter(&self) {
            let joiner = self
                .joiner
                .borrow()
                .clone()
                .expect("Joiner should be set up");
            let filters = joiner.filters();

            let mut user_data_dir = gtk::glib::user_data_dir();
            user_data_dir.push("tubefeeder");

            let mut filters_file_path = user_data_dir.clone();
            filters_file_path.push("filters.csv");

            let _filter_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
                &filters_file_path,
                &mut |filter| {
                    filters
                        .lock()
                        .expect("Filter Group to be lockable")
                        .add(filter)
                },
            ))
                as Box<dyn Observer<FilterEvent<AnyVideoFilter>> + Send>));

            filters
                .lock()
                .expect("Filter Group to be lockable")
                .attach(Arc::downgrade(&_filter_file_manager));

            self._filter_file_manager
                .replace(Some(_filter_file_manager));
            self.filter_page.get().set_filter_group(filters);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "TFWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            crate::gui::stack_page::StackPage::ensure_type();
            PredefinedPlayer::ensure_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_actions(&self.obj());
            self.setup_feed();
            self.setup_watch_later();
            self.setup_subscriptions();
            self.setup_filter();

            let obj = self.obj();
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
            obj.load_window_size();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        fn close_request(&self) -> Propagation {
            let mut user_cache_dir = glib::user_cache_dir();
            user_cache_dir.push("tubefeeder");

            if user_cache_dir.exists() {
                std::fs::remove_dir_all(user_cache_dir).unwrap_or(());
            }

            let obj = self.obj();
            if let Err(err) = obj.save_window_size() {
                log::warn!("Failed to save window state, {}", &err);
            }

            self.parent_close_request()
        }
    }
    impl ApplicationWindowImpl for Window {}
    impl AdwWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}
