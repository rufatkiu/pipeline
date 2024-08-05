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
use gdk_pixbuf::prelude::Cast;
use gtk::prelude::WidgetExt;
use tf_join::{AnySubscriptionList, AnyVideo};
use tf_playlist::PlaylistManager;

use crate::gui::stack_page::StackPage;

gtk::glib::wrapper! {
    pub struct SubscriptionPage(ObjectSubclass<imp::SubscriptionPage>)
        @extends StackPage, adw::Bin, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl SubscriptionPage {
    pub fn present_subscribe(&self) {
        log::trace!("Subscription page got request to present the subscription window");
        self.imp().present_subscribe();
    }

    pub fn set_subscription_list(
        &self,
        subscription_list: AnySubscriptionList,
        playlist_manager: PlaylistManager<String, AnyVideo>,
    ) {
        self.imp()
            .any_subscription_list
            .replace(Some(subscription_list.clone()));
        self.imp()
            .subscription_list
            .get()
            .set_subscription_list(subscription_list);
        self.imp()
            .subscription_video_list
            .get()
            .set_playlist_manager(playlist_manager);
    }

    fn window(&self) -> crate::gui::window::Window {
        self.root()
            .expect("SubscriptionPage to have root")
            .downcast::<crate::gui::window::Window>()
            .expect("Root to be window")
    }

    pub fn emit_watch_later(&self) {
        self.imp().subscription_video_list.emit_watch_later();
    }

    pub fn emit_download(&self) {
        self.imp().subscription_video_list.emit_download();
    }

    pub fn emit_copy_to_clipboard(&self) {
        self.imp().subscription_video_list.emit_copy_to_clipboard();
    }

    pub fn emit_open_in_browser(&self) {
        self.imp().subscription_video_list.emit_open_in_browser();
    }

    pub fn emit_information(&self) {
        self.imp().subscription_video_list.emit_information();
    }
}

pub mod imp {
    use std::cell::RefCell;
    use std::str::FromStr;

    use futures::SinkExt;
    use futures::StreamExt;
    use gdk::gio::ListStore;
    use gdk::glib::clone;
    use gdk::glib::Object;
    use gdk::glib::ParamSpec;
    use gdk_pixbuf::glib::subclass::Signal;
    use glib::subclass::InitializingObject;
    use gtk::glib;

    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::ConstantExpression;
    use gtk::PropertyExpression;

    use adw::prelude::AdwDialogExt;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use tf_core::Generator;
    use tf_join::AnySubscriptionList;
    use tf_join::Platform;
    // use tf_lbry::LbrySubscription;
    use tf_pt::PTSubscription;
    use tf_yt::YTSubscription;

    use crate::config::APP_ID;
    use crate::gspawn;
    use crate::gui::feed::feed_item_object::VideoObject;
    use crate::gui::feed::feed_list::FeedList;
    use crate::gui::stack_page::StackPage;
    use crate::gui::stack_page::StackPageImpl;
    use crate::gui::subscription::platform::PlatformObject;
    use crate::gui::subscription::subscription_item_object::SubscriptionObject;
    use crate::gui::subscription::subscription_list::SubscriptionList;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/subscription_page.ui")]
    pub struct SubscriptionPage {
        #[template_child]
        pub(super) subscription_list: TemplateChild<SubscriptionList>,

        #[template_child]
        pub(super) btn_toggle_add_subscription: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) btn_add_subscription: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) btn_go_back: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) dropdown_platform: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub(super) entry_url: TemplateChild<gtk::Entry>,
        #[template_child]
        pub(super) entry_name_id: TemplateChild<gtk::Entry>,
        #[template_child]
        pub(super) dialog_add: TemplateChild<adw::AlertDialog>,
        #[template_child]
        pub(super) dialog_error: TemplateChild<adw::AlertDialog>,

        #[template_child]
        pub(super) subscription_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) subscription_video_list: TemplateChild<FeedList>,

        pub(super) any_subscription_list: RefCell<Option<AnySubscriptionList>>,

        settings: gtk::gio::Settings,
    }

    impl Default for SubscriptionPage {
        fn default() -> Self {
            Self {
                subscription_list: Default::default(),
                btn_toggle_add_subscription: Default::default(),
                btn_add_subscription: Default::default(),
                btn_go_back: Default::default(),
                dropdown_platform: Default::default(),
                entry_url: Default::default(),
                entry_name_id: Default::default(),
                dialog_add: Default::default(),
                dialog_error: Default::default(),
                subscription_stack: Default::default(),
                subscription_video_list: Default::default(),
                any_subscription_list: Default::default(),
                settings: gtk::gio::Settings::new(APP_ID),
            }
        }
    }

    impl SubscriptionPage {
        pub(super) fn present_subscribe(&self) {
            let platform = Platform::from_str(&self.settings.string("last-added-platform"))
                .unwrap_or(Platform::Youtube);

            self.dropdown_platform.set_selected(
                self.platforms()
                    .into_iter()
                    .position(|p| p == &platform)
                    .unwrap_or_default()
                    .try_into()
                    .unwrap_or_default(),
            );
            self.entry_url.set_text("");
            self.entry_name_id.set_text("");

            let window = self.obj().window();
            self.dialog_add.present(Some(&window));

            // XXX: Duplicated logic with `url_visible`.
            if platform == Platform::Peertube {
                self.entry_url.grab_focus();
            } else {
                self.entry_name_id.grab_focus();
            }
        }

        fn setup_toggle_add_subscription(&self, obj: &super::SubscriptionPage) {
            self.btn_toggle_add_subscription.connect_clicked(clone!(
                #[weak]
                obj,
                move |_| {
                    obj.present_subscribe();
                }
            ));
            self.btn_add_subscription.connect_clicked(clone!(
                #[weak]
                obj,
                move |_| {
                    obj.present_subscribe();
                }
            ));
        }

        fn platforms(&self) -> &'static [Platform] {
            // XXX: Maybe lazy?
            &[
                Platform::Youtube,
                // PlatformObject::new(Platform::Lbry),
                Platform::Peertube,
            ]
        }

        fn setup_platform_dropdown(&self) {
            self.dropdown_platform
                .set_expression(Some(&PropertyExpression::new(
                    PlatformObject::static_type(),
                    None::<ConstantExpression>,
                    "name",
                )));

            let model = ListStore::new::<PlatformObject>();
            model.splice(
                0,
                0,
                &self
                    .platforms()
                    .into_iter()
                    .map(|p| PlatformObject::new(p.clone()))
                    .collect::<Vec<_>>(),
            );
            self.dropdown_platform.set_model(Some(&model));
        }
    }

    #[gtk::template_callbacks]
    impl SubscriptionPage {
        #[template_callback]
        fn handle_entry_name_id_activate(&self) {
            self.handle_add_subscription(Some("add"));
            self.dialog_add.close();
        }

        #[template_callback]
        fn handle_add_subscription(&self, response: Option<&str>) {
            if response != Some("add") {
                return;
            }

            let in_platform = &self.dropdown_platform;
            let in_url = &self.entry_url;
            let in_name_id = &self.entry_name_id;

            let platform = in_platform
                .selected_item()
                .expect("Something has to be selected.")
                .downcast::<PlatformObject>()
                .expect("Dropdown items should be of type PlatformObject.")
                .platform()
                .expect("The platform has to be set up.");
            let url = in_url.text();
            let name_id = in_name_id.text();

            in_url.set_text("");
            in_name_id.set_text("");
            let _ = self
                .settings
                .set_string("last-added-platform", &platform.to_string());

            let (sender, mut receiver) = futures::channel::mpsc::channel(1);
            let mut sender = sender.clone();
            tokio::spawn(async move {
                let subscription = match platform {
                    Platform::Youtube => YTSubscription::try_from_search(&name_id)
                        .await
                        .map(|s| s.into()),
                    Platform::Peertube => Some(PTSubscription::new(&url, &name_id).into()),
                    // Platform::Lbry => Some(LbrySubscription::new(&name_id).into()),
                    // -- Add case here
                };
                sender.send(subscription).await
            });

            let obj = self.obj();
            gspawn!(clone!(
                #[strong(rename_to = list)]
                self.any_subscription_list,
                #[strong]
                obj,
                async move {
                    while let Some(sub) = receiver.next().await {
                        if let Some(sub) = sub {
                            list.borrow()
                                .as_ref()
                                .expect("SubscriptionList should be set up")
                                .add(sub);
                            obj.emit_by_name::<()>("subscription-added", &[]);
                        } else {
                            log::error!("Failed to get subscription with supplied data");
                            let window = obj.window();
                            let dialog_error = &obj.imp().dialog_error;
                            dialog_error.present(Some(&window));
                        }
                    }
                }
            ));
        }

        #[template_callback]
        fn handle_go_to_videos_page(&self, subscription: SubscriptionObject) {
            log::debug!(
                "Going to videos of subscription {}",
                subscription
                    .subscription()
                    .expect("SubscriptionObject to have value")
            );
            self.subscription_stack.set_visible_child_name("page-vid");
            let joiner = tf_join::Joiner::new();
            joiner.subscription_list().add(
                subscription
                    .subscription()
                    .expect("SubscriptionObject to have value"),
            );

            let error_store = tf_core::ErrorStore::new();

            let (mut sender, mut receiver) = futures::channel::mpsc::channel(1);
            tokio::spawn(async move {
                let videos = joiner.generate(&error_store).await;
                let _ = sender.send(videos).await;
            });
            let obj = self.obj();
            gspawn!(clone!(
                #[weak]
                obj,
                #[upgrade_or_default]
                async move {
                    while let Some(videos) = receiver.next().await {
                        let video_objects =
                            videos.into_iter().map(VideoObject::new).collect::<Vec<_>>();
                        obj.imp()
                            .subscription_video_list
                            .get()
                            .set_items(video_objects);
                    }
                }
            ));

            self.obj()
                .set_property("header-widget", &self.btn_go_back.get());
        }

        #[template_callback]
        fn handle_go_to_subscriptions_page(&self) {
            log::debug!("Going back to the subscriptions page",);
            self.subscription_stack.set_visible_child_name("page-sub");

            self.obj()
                .set_property("header-widget", &self.btn_toggle_add_subscription.get());
        }

        #[template_callback(function)]
        fn url_visible(#[rest] values: &[gtk::glib::Value]) -> bool {
            let platform: Option<PlatformObject> = values[0]
                .get::<Option<Object>>()
                .expect("Parameter must be a Object")
                .map(|o| o.downcast().expect("Parameter must be PlatformObject"));
            platform.as_ref().map(PlatformObject::platform).flatten() == Some(Platform::Peertube)
        }

        #[template_callback(function)]
        fn name_visible(#[rest] _values: &[gtk::glib::Value]) -> bool {
            true
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubscriptionPage {
        const NAME: &'static str = "TFSubscriptionPage";
        type Type = super::SubscriptionPage;
        type ParentType = StackPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SubscriptionPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_toggle_add_subscription(&self.obj());
            self.setup_platform_dropdown();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(Vec::new);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &glib::Value, _pspec: &glib::ParamSpec) {
            unimplemented!()
        }

        fn property(&self, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
            unimplemented!()
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("subscription-added").build()]);
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for SubscriptionPage {}
    impl BinImpl for SubscriptionPage {}
    impl StackPageImpl for SubscriptionPage {}
}
