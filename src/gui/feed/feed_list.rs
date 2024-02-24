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

use std::{
    cmp::{min, Reverse},
    time::Duration,
};

use gdk::{
    gio::{SimpleAction, SimpleActionGroup},
    glib,
    glib::clone,
    prelude::{ActionMapExt, ListModelExt, ObjectExt, ToValue},
    subclass::prelude::ObjectSubclassIsExt,
};
use gdk_pixbuf::prelude::CastNone;
use glib::object::Cast;
use gtk::{
    prelude::{AdjustmentExt, WidgetExt},
    Adjustment,
};
use tf_join::AnyVideo;
use tf_playlist::PlaylistManager;

use super::{feed_item::FeedItem, feed_item_object::VideoObject};

const LOAD_COUNT: usize = 10;

gtk::glib::wrapper! {
    pub struct FeedList(ObjectSubclass<imp::FeedList>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl FeedList {
    fn add_actions(&self) {
        let action_more = SimpleAction::new("more", None);
        action_more.connect_activate(clone!(@strong self as s => move |_, _| {
            let imp = s.imp();
            let items = &imp.items.borrow();
            let model = &imp.model.borrow();
            let loaded_count = &imp.loaded_count.get();

            let to_load = min(LOAD_COUNT, items.len() - loaded_count);

            model.splice(model.n_items(), 0, &items[*loaded_count..(loaded_count + to_load)]);
            imp.loaded_count.set(loaded_count + to_load);

            s.set_more_available();
        }));

        let actions = SimpleActionGroup::new();
        self.insert_action_group("feed", Some(&actions));
        actions.add_action(&action_more);
    }

    pub fn emit_watch_later(&self) {
        if let Ok(item) = self
            .imp()
            .feed_list
            .focus_child()
            .and_then(|w| w.first_child())
            .and_dynamic_cast::<FeedItem>()
        {
            let _ = item.activate_action("item.watch-later", None);
        }
    }

    pub fn emit_download(&self) {
        if let Ok(item) = self
            .imp()
            .feed_list
            .focus_child()
            .and_then(|w| w.first_child())
            .and_dynamic_cast::<FeedItem>()
        {
            let _ = item.activate_action("item.download", None);
        }
    }

    pub fn emit_copy_to_clipboard(&self) {
        if let Ok(item) = self
            .imp()
            .feed_list
            .focus_child()
            .and_then(|w| w.first_child())
            .and_dynamic_cast::<FeedItem>()
        {
            let _ = item.activate_action("item.clipboard", None);
        }
    }

    pub fn emit_open_in_browser(&self) {
        if let Ok(item) = self
            .imp()
            .feed_list
            .focus_child()
            .and_then(|w| w.first_child())
            .and_dynamic_cast::<FeedItem>()
        {
            let _ = item.activate_action("item.open-in-browser", None);
        }
    }

    pub fn emit_information(&self) {
        if let Ok(item) = self
            .imp()
            .feed_list
            .focus_child()
            .and_then(|w| w.first_child())
            .and_dynamic_cast::<FeedItem>()
        {
            let _ = item.activate_action("item.information", None);
        }
    }

    fn setup_autoload(&self) {
        let adj = self.imp().scrolled_window.vadjustment();
        adj.connect_changed(clone!(@weak self as s => move |adj| {
            s.load_if_screen_not_filled(adj);
        }));
    }

    fn load_if_screen_not_filled(&self, adj: &Adjustment) {
        let ctx = glib::MainContext::default();
        ctx.spawn_local(clone!(@weak self as s, @weak adj => async move {
            // Add timeout, otherwise there are some critical errors (not sure why).
            glib::timeout_future(Duration::from_millis(100)).await;
            if s.property("more-available") && adj.upper() <= adj.page_size() {
                // The screen is not yet filled.
                let _ = s.activate_action("feed.more", None);
            }
        }));
    }

    pub fn set_items(&self, new_items: Vec<VideoObject>) {
        let imp = self.imp();
        let items = &imp.items;
        let model = &imp.model;
        let loaded_count = &imp.loaded_count;

        let _ = items.replace(new_items);
        model.borrow().remove_all();
        loaded_count.set(0);

        self.set_more_available();
        let _ = self.activate_action("feed.more", None);

        self.notify("is-empty");
    }

    pub fn set_items_ordered(&self, new_items: Vec<VideoObject>) {
        let mut new_items = new_items;
        new_items.sort_unstable_by_key(|v| Reverse(v.uploaded().unwrap_or_default()));
        self.set_items(new_items);
    }

    pub fn prepend(&self, new_item: VideoObject) {
        let imp = self.imp();
        let items = &imp.items;
        let model = &imp.model;
        let loaded_count = &imp.loaded_count;

        let _ = items.borrow_mut().insert(0, new_item.clone());
        model.borrow_mut().insert(0, &new_item);
        loaded_count.set(loaded_count.get() + 1);

        self.set_more_available();
        self.notify("is-empty");
    }

    pub fn insert_ordered_time(&self, new_item: VideoObject) {
        // Extra block needed to end the mutable borrow of `items`.
        {
            let imp = self.imp();
            let mut items = imp.items.borrow_mut();
            let model = imp.model.borrow();
            let loaded_count = &imp.loaded_count;

            let idx = items
                .binary_search_by_key(&Reverse(new_item.uploaded().unwrap_or_default()), |v| {
                    Reverse(v.uploaded().unwrap_or_default())
                })
                .unwrap_or_else(|i| i);

            let _ = items.insert(idx, new_item.clone());
            if idx <= model.n_items() as usize {
                model.insert(idx as u32, &new_item);
                loaded_count.set(loaded_count.get() + 1);
            }
        }

        self.set_more_available();
        self.notify("is-empty");
    }

    pub fn remove(&self, new_item: VideoObject) {
        // Extra block needed to end the mutable borrow of `items`.
        {
            let imp = self.imp();
            let mut items = imp.items.borrow_mut();
            let model = &imp.model;
            let loaded_count = &imp.loaded_count;

            if let Some(idx) = items.iter().position(|i| i.video() == new_item.video()) {
                if idx < loaded_count.get() {
                    model.borrow().remove(idx as u32);
                    loaded_count.set(loaded_count.get() - 1);
                }

                items.remove(idx);
            }
        }

        self.set_more_available();
        self.notify("is-empty");
    }

    pub fn set_playlist_manager(&self, playlist_manager: PlaylistManager<String, AnyVideo>) {
        self.imp().playlist_manager.replace(Some(playlist_manager));
        self.imp().setup();
    }

    fn set_more_available(&self) {
        let imp = self.imp();
        let items_count = imp.items.borrow().len();
        let loaded_count = imp.loaded_count.get();

        self.set_property("more-available", (items_count != loaded_count).to_value());
    }

    fn window(&self) -> crate::gui::window::Window {
        self.root()
            .expect("FeedList to have root")
            .downcast::<crate::gui::window::Window>()
            .expect("Root to be window")
    }
}

pub mod imp {
    use std::cell::{Cell, RefCell};

    use gdk::gio::ListStore;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecBoolean;
    use gdk::glib::Value;
    use gdk_pixbuf::glib::clone;
    use gdk_pixbuf::glib::Propagation;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::PositionType;
    use gtk::SignalListItemFactory;
    use gtk::Widget;

    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use tf_join::AnyVideo;
    use tf_playlist::PlaylistManager;

    use crate::gspawn;
    use crate::gui::feed::feed_item::FeedItem;
    use crate::gui::feed::feed_item_object::VideoObject;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/feed_list.ui")]
    pub struct FeedList {
        #[template_child]
        pub(super) feed_list: TemplateChild<gtk::GridView>,
        #[template_child]
        pub(super) scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub(super) dialog_error: TemplateChild<adw::MessageDialog>,

        pub(super) items: RefCell<Vec<VideoObject>>,
        pub(super) model: RefCell<ListStore>,
        pub(super) loaded_count: Cell<usize>,

        pub(super) playlist_manager: RefCell<Option<PlaylistManager<String, AnyVideo>>>,

        pub(super) more_available: Cell<bool>,
    }

    impl Default for FeedList {
        fn default() -> Self {
            Self {
                feed_list: Default::default(),
                scrolled_window: Default::default(),
                dialog_error: Default::default(),
                items: Default::default(),
                model: RefCell::new(ListStore::new::<FeedItem>()),
                loaded_count: Default::default(),
                playlist_manager: Default::default(),
                more_available: Default::default(),
            }
        }
    }

    impl FeedList {
        pub(super) fn setup(&self) {
            let model = gtk::gio::ListStore::new::<VideoObject>();
            let selection_model = gtk::NoSelection::new(Some(model.clone()));
            self.feed_list.get().set_model(Some(&selection_model));

            self.model.replace(model);

            let factory = SignalListItemFactory::new();
            let playlist_manager = self
                .playlist_manager
                .borrow()
                .clone()
                .expect("PlaylistManager should be set up");
            factory.connect_setup(move |_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                let feed_item = FeedItem::new(playlist_manager.clone());
                list_item.set_child(Some(&feed_item));

                list_item
                    .property_expression("item")
                    .bind(&feed_item, "video", Widget::NONE);
            });
            self.feed_list.set_factory(Some(&factory));
            self.feed_list.set_single_click_activate(true);

            self.feed_list
                .connect_activate(clone!(@weak self as s => move |list_view, position| {
                    let model = list_view.model().expect("The model has to exist.");
                    let video_object = model
                        .item(position)
                        .expect("The item has to exist.")
                        .downcast::<VideoObject>()
                        .expect("The item has to be an `VideoObject`.");

                    let receiver = video_object.play();

                    gspawn!(async move {
                        if let Err(e) = receiver.await.expect("Video receiver to not fail") {
                            log::error!("Failed to play video: {}", e);
                            let window = s.obj().window();
                            let dialog_error = &s.dialog_error;
                            dialog_error.set_transient_for(Some(&window));
                            dialog_error.present();
                        }
                    });
                }));

            let key_events = gtk::EventControllerKey::new();
            key_events.connect_key_pressed(
                clone!(@strong self.feed_list as feed_list => @default-return Propagation::Proceed, move |_, key, _, _| {
                    if key == gdk::Key::Menu {
                        feed_list.focus_child().and_then(|w| w.first_child()).and_dynamic_cast::<FeedItem>().expect("FeedList to have highlighted FeedItem").click();
                        Propagation::Stop
                    } else {
                        Propagation::Proceed
                    }
                }),
            );
            self.feed_list.add_controller(key_events);

            self.obj().setup_autoload();
        }
    }

    #[gtk::template_callbacks]
    impl FeedList {
        #[template_callback]
        fn edge_reached(&self, pos: PositionType) {
            if pos == PositionType::Bottom {
                let _ = gtk::prelude::WidgetExt::activate_action(
                    self.obj().as_ref(),
                    "feed.more",
                    None,
                );
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FeedList {
        const NAME: &'static str = "TFFeedList";
        type Type = super::FeedList;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FeedList {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().add_actions();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecBoolean::builder("more-available").build(),
                    ParamSpecBoolean::builder("is-empty").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "more-available" => {
                    let value: bool = value
                        .get()
                        .expect("Property more-available of incorrect type");
                    self.more_available.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "more-available" => self.more_available.get().to_value(),
                "is-empty" => (self.model.borrow().n_items() == 0).to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for FeedList {}
    impl BoxImpl for FeedList {}
}
