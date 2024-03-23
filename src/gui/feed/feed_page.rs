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
use tf_join::{AnyVideo, Joiner};
use tf_playlist::PlaylistManager;

use crate::gui::stack_page::StackPage;

gtk::glib::wrapper! {
    pub struct FeedPage(ObjectSubclass<imp::FeedPage>)
        @extends StackPage, adw::Bin, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl FeedPage {
    pub fn setup(&self, playlist_manager: PlaylistManager<String, AnyVideo>, joiner: Joiner) {
        self.imp().playlist_manager.replace(Some(playlist_manager));
        self.imp().joiner.replace(Some(joiner));
        self.imp().setup(&self);
    }

    pub fn reload(&self) {
        self.imp().reload();
    }

    pub fn emit_watch_later(&self) {
        self.imp().feed_list.emit_watch_later();
    }

    pub fn emit_download(&self) {
        self.imp().feed_list.emit_download();
    }

    pub fn emit_copy_to_clipboard(&self) {
        self.imp().feed_list.emit_copy_to_clipboard();
    }

    pub fn emit_open_in_browser(&self) {
        self.imp().feed_list.emit_open_in_browser();
    }

    pub fn emit_information(&self) {
        self.imp().feed_list.emit_information();
    }
}

pub mod imp {
    use std::cell::Cell;
    use std::cell::RefCell;

    use chrono::Duration;
    use futures::SinkExt;
    use futures::StreamExt;
    use gdk::glib::clone;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecBoolean;
    use gdk::prelude::{ObjectExt, SettingsExt, ToValue};
    use glib::subclass::InitializingObject;
    use gtk::gio::Settings;
    use gtk::glib;
    use gtk::glib::subclass::Signal;
    use gtk::prelude::ButtonExt;

    use gtk::subclass::prelude::*;

    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use tf_core::ErrorStore;
    use tf_core::Generator;
    use tf_join::AnyVideo;
    use tf_join::Joiner;
    use tf_playlist::PlaylistManager;

    use crate::config::APP_ID;
    use crate::gspawn;
    use crate::gui::feed::error_label::ErrorLabel;
    use crate::gui::feed::feed_item_object::VideoObject;
    use crate::gui::feed::feed_list::FeedList;
    use crate::gui::stack_page::StackPage;
    use crate::gui::stack_page::StackPageImpl;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/feed_page.ui")]
    pub struct FeedPage {
        #[template_child]
        pub(super) feed_list: TemplateChild<FeedList>,

        #[template_child]
        pub(super) btn_reload: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) btn_manage_subscriptions: TemplateChild<gtk::Button>,

        #[template_child]
        pub(super) error_label: TemplateChild<ErrorLabel>,

        reloading: Cell<bool>,

        pub(super) playlist_manager: RefCell<Option<PlaylistManager<String, AnyVideo>>>,
        pub(super) joiner: RefCell<Option<Joiner>>,
        error_store: RefCell<ErrorStore>,

        pub settings: gtk::gio::Settings,
    }

    impl Default for FeedPage {
        fn default() -> Self {
            Self {
                feed_list: Default::default(),
                btn_reload: Default::default(),
                btn_manage_subscriptions: Default::default(),
                error_label: Default::default(),
                reloading: Default::default(),
                playlist_manager: Default::default(),
                joiner: Default::default(),
                error_store: Default::default(),
                settings: Settings::new(APP_ID),
            }
        }
    }

    impl FeedPage {
        pub(super) fn reload(&self) {
            self.btn_reload.emit_clicked();
        }

        fn setup_reload(&self, obj: &super::FeedPage) {
            let joiner = self
                .joiner
                .borrow()
                .clone()
                .expect("Joiner should be set up");

            let (sender, mut receiver) = futures::channel::mpsc::channel(1);
            let sender = sender.clone();
            let joiner = joiner.clone();
            let error_store = self.error_store.borrow().clone();
            let settings = self.settings.clone();

            self.btn_reload.connect_clicked(
                clone!(@strong obj as s, @strong joiner, @strong error_store => move |_| {
                    log::debug!("Reloading");
                    s.set_property("reloading", &true);

                    let mut sender = sender.clone();
                    let joiner = joiner.clone();
                    let error_store = error_store.clone();
                    error_store.clear();
                    tokio::spawn(async move {
                        let videos = joiner.generate(&error_store).await;
                        let _ = sender.send(videos).await;
                    });
                }),
            );
            gspawn!(
                clone!(@strong obj as s, @strong settings => @default-return Propagate::Stop, async move {
                    while let Some(videos) = receiver.next().await {
                        let yesterday = chrono::Local::now().date_naive() - Duration::try_days(1).expect("one day to be expressable as a duration");

                        let only_yesterday = settings.boolean("only-videos-yesterday");
                        let remove_short = settings.boolean("remove-short-videos");

                        let video_objects = videos
                            .into_iter()
                            .map(VideoObject::new)
                            .filter(|v| !only_yesterday || v.uploaded().map(|d| d.date()) == Some(yesterday))
                            .filter(|v| !(remove_short && v.duration().map(|d| d < Duration::try_seconds(61).expect("61 seconds to be expressable as a duration")).unwrap_or_default())) // One more second padding to be sure.
                            .collect::<Vec<_>>();

                        s.imp().feed_list.get().set_items(video_objects);
                        s.set_property("reloading", &false);
                    }

                })
            );

            // Setup Error Label
            self.error_label
                .set_error_store(self.error_store.borrow().clone());

            // Simulate reload on startup.
            self.btn_reload.emit_clicked();

            self.joiner.replace(Some(joiner));
        }

        fn setup_manage_subscriptions(&self, obj: &super::FeedPage) {
            self.btn_manage_subscriptions
                .connect_clicked(clone!(@weak obj => move |_| {
                    obj.emit_by_name::<()>("go-to-subscriptions", &[]);
                }));
        }

        pub(super) fn setup(&self, obj: &super::FeedPage) {
            self.feed_list.set_playlist_manager(
                self.playlist_manager
                    .borrow()
                    .clone()
                    .expect("PlaylistManager has to be set up"),
            );
            self.setup_reload(obj);
            self.setup_manage_subscriptions(obj);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FeedPage {
        const NAME: &'static str = "TFFeedPage";
        type Type = super::FeedPage;
        type ParentType = StackPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FeedPage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecBoolean::builder("reloading").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "reloading" => {
                    let _ = self.reloading.replace(
                        value
                            .get()
                            .expect("The property 'reloading' of TFWindow has to be boolean"),
                    );
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "reloading" => self.reloading.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("go-to-subscriptions").build()]);
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for FeedPage {}
    impl BinImpl for FeedPage {}
    impl StackPageImpl for FeedPage {}
}
