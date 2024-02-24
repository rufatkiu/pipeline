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
use tf_join::AnyVideo;
use tf_playlist::PlaylistManager;

use super::stack_page::StackPage;

gtk::glib::wrapper! {
    pub struct WatchLaterPage(ObjectSubclass<imp::WatchLaterPage>)
        @extends StackPage, adw::Bin, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl WatchLaterPage {
    pub fn set_playlist_manager(&self, playlist_manager: PlaylistManager<String, AnyVideo>) {
        self.imp().playlist_manager.replace(Some(playlist_manager));
        self.imp().setup();
    }

    pub fn emit_watch_later(&self) {
        self.imp().feed_page.emit_watch_later();
    }

    pub fn emit_download(&self) {
        self.imp().feed_page.emit_download();
    }

    pub fn emit_copy_to_clipboard(&self) {
        self.imp().feed_page.emit_copy_to_clipboard();
    }

    pub fn emit_open_in_browser(&self) {
        self.imp().feed_page.emit_open_in_browser();
    }

    pub fn emit_information(&self) {
        self.imp().feed_page.emit_information();
    }
}

pub mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use futures::SinkExt;
    use futures::StreamExt;
    use glib::clone;
    use glib::subclass::InitializingObject;
    use gtk::glib;

    use gtk::subclass::prelude::*;

    use gtk::CompositeTemplate;
    use adw::subclass::prelude::BinImpl;
    use tf_join::AnyVideo;
    use tf_observer::Observer;
    use tf_playlist::PlaylistEvent;
    use tf_playlist::PlaylistManager;

    use crate::gspawn;
    use crate::gui::feed::feed_item_object::VideoObject;
    use crate::gui::feed::feed_list::FeedList;
    use crate::gui::stack_page::StackPage;
    use crate::gui::stack_page::StackPageImpl;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/watch_later.ui")]
    pub struct WatchLaterPage {
        #[template_child]
        pub(super) feed_page: TemplateChild<FeedList>,

        pub(super) playlist_manager: RefCell<Option<PlaylistManager<String, AnyVideo>>>,

        _playlist_observer:
            RefCell<Option<Arc<Mutex<Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>>>>>,
    }

    impl WatchLaterPage {
        pub(super) fn setup(&self) {
            let mut playlist_manager = self
                .playlist_manager
                .borrow()
                .clone()
                .expect("Playlist Manager has to exist");

            let (sender, mut receiver) = futures::channel::mpsc::channel(1);

            let observer = Arc::new(Mutex::new(Box::new(PlaylistPageObserver {
                sender: sender.clone(),
            })
                as Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>));

            let mut existing: Vec<VideoObject> = playlist_manager
                .items(&"WATCHLATER".to_string())
                .iter()
                .map(|v| VideoObject::new(v.clone()))
                .collect();
            existing.reverse();

            playlist_manager.attach_at(Arc::downgrade(&observer), &"WATCHLATER".to_string());
            self._playlist_observer.replace(Some(observer));

            let feed_page = &self.feed_page.clone();
            feed_page.set_playlist_manager(playlist_manager);
            feed_page.set_items_ordered(existing);

            gspawn!(clone!(@strong feed_page => async move {
                while let Some(playlist_event) = receiver.next().await {
                    match playlist_event {
                        PlaylistEvent::Add(v) => {
                            let video = VideoObject::new(v);
                            feed_page.insert_ordered_time(video);
                        }
                        PlaylistEvent::Remove(v) => {
                            let video = VideoObject::new(v);
                            feed_page.remove(video);
                        }
                    }
                }
            }));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WatchLaterPage {
        const NAME: &'static str = "TFWatchLaterPage";
        type Type = super::WatchLaterPage;
        type ParentType = StackPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WatchLaterPage {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for WatchLaterPage {}
    impl BinImpl for WatchLaterPage {}
    impl StackPageImpl for WatchLaterPage {}

    pub struct PlaylistPageObserver {
        sender: futures::channel::mpsc::Sender<PlaylistEvent<AnyVideo>>,
    }

    impl Observer<PlaylistEvent<AnyVideo>> for PlaylistPageObserver {
        fn notify(&mut self, message: PlaylistEvent<AnyVideo>) {
            let mut sender = self.sender.clone();
            gspawn!(async move {
                let _ = sender.send(message).await;
            });
        }
    }
}
