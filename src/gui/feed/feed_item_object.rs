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

use std::cell::RefCell;

use chrono::Duration;
use gdk::glib;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use gdk::{
    glib::{clone, Object},
    prelude::ObjectExt,
};
use tf_core::{ExtraVideoInfo, Video};
use tf_join::AnyVideo;

use crate::downloader::download;
use crate::gspawn;
use crate::player::play;

macro_rules! str_prop {
    ( $x:expr ) => {
        ParamSpecString::builder($x).build()
    };
}

macro_rules! prop_set {
    ( $x:expr, $value:expr ) => {
        let input = $value
            .get::<'_, Option<String>>()
            .expect("The value needs to be of type `Option<String>`.");
        $x.replace(input);
    };
}

macro_rules! prop_set_all {
    ( $value:expr, $pspec:expr, $( $key:expr, $element:expr ),* ) => {
        match $pspec.name() {
            $(
                $key => { prop_set!($element, $value); },
            )*
                _ => unimplemented!()
        }
    }
}

macro_rules! prop_get_all {
    ( $pspec:expr, $( $key:expr, $element:expr ),* ) => {
        match $pspec.name() {
            $(
                $key => { $element.borrow().to_value() },
            )*
                _ => unimplemented!()
        }
    }
}

gtk::glib::wrapper! {
    pub struct VideoObject(ObjectSubclass<imp::VideoObject>);
}

fn format_duration(d: Duration) -> String {
    if d.num_hours() > 0 {
        format!(
            "{}:{:0>2}:{:0>2}",
            d.num_hours(),
            (d - Duration::try_hours(d.num_hours()).unwrap_or_default()).num_minutes(),
            (d - Duration::try_minutes(d.num_minutes()).unwrap_or_default()).num_seconds()
        )
    } else {
        format!(
            "{:0>2}:{:0>2}",
            d.num_minutes(),
            (d - Duration::try_minutes(d.num_minutes()).unwrap_or_default()).num_seconds()
        )
    }
}

impl VideoObject {
    pub fn new(video: AnyVideo) -> Self {
        let s: Self = Object::builder::<Self>()
            .property("title", &video.title())
            .property("url", &video.url())
            .property("thumbnail-url", &video.thumbnail_url())
            .property("author", &video.subscription().to_string())
            .property("platform", &video.platform().to_string())
            .property(
                "date",
                &video
                    .uploaded()
                    // Translators: How to display the uploaded date. This is a date format corresponding to <https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers>. E.g. ``%F %T` can expand to "2024-07-08 12:34:30".
                    .format(&gettextrs::gettext("%F %T"))
                    .to_string(),
            )
            .property("duration", video.duration().map(format_duration))
            .property("playing", &false)
            .build();
        s.imp().duration.swap(&RefCell::new(video.duration()));
        s.imp().video.swap(&RefCell::new(Some(video)));
        s
    }

    pub fn video(&self) -> Option<AnyVideo> {
        self.imp().video.borrow().clone()
    }

    pub fn uploaded(&self) -> Option<chrono::NaiveDateTime> {
        self.video().map(|v| v.uploaded())
    }

    pub fn play(&self) -> futures::channel::oneshot::Receiver<Result<(), std::io::Error>> {
        self.set_property("playing", true);
        let (sender, receiver) = futures::channel::oneshot::channel();
        let (sender_return, receiver_return) = futures::channel::oneshot::channel();
        play(
            self.property::<Option<String>>("local-path")
                .unwrap_or_else(|| self.property::<Option<String>>("url").unwrap_or_default()),
            move |r| {
                let _ = sender.send(r);
            },
        );
        gspawn!(clone!(@weak self as s => async move {
            let r = receiver.await.expect("Video play receiver to not be cancelled");
            s.set_property("playing", false);
            let _ = sender_return.send(r);
        }));

        receiver_return
    }

    pub fn download(&self) -> futures::channel::oneshot::Receiver<Result<(), std::io::Error>> {
        self.set_property("downloading", true);
        let (sender, receiver) = futures::channel::oneshot::channel();
        let (sender_return, receiver_return) = futures::channel::oneshot::channel();
        download(
            self.property::<Option<String>>("url").unwrap_or_default(),
            move |r| {
                let _ = sender.send(r);
            },
        );
        gspawn!(clone!(@weak self as s => async move {
            let r = receiver.await.expect("Video play receiver to not be cancelled");
            s.set_property("downloading", false);
            if let Ok(local_path) = &r {
                s.set_property("local-path", local_path);
                s.notify("is-local");
            }
            let _ = sender_return.send(r.map(|_| ()));
        }));

        receiver_return
    }

    pub async fn extra_info(&self) -> Result<Option<ExtraVideoInfo>, tf_join::AnyFetchError> {
        self.imp()
            .video
            .borrow()
            .as_ref()
            .expect("Video to be set")
            .extra_information_with_client(&reqwest::Client::new())
            .await
    }

    pub fn duration(&self) -> Option<Duration> {
        self.imp().duration.borrow().as_ref().cloned()
    }
}

mod imp {
    use chrono::Duration;
    use gtk::glib::{self, Object};
    use std::cell::{Cell, RefCell};
    use tf_join::AnyVideo;

    use gdk::{
        glib::{ParamSpec, ParamSpecBoolean, ParamSpecString, Value},
        prelude::ToValue,
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default)]
    pub struct VideoObject {
        title: RefCell<Option<String>>,
        author: RefCell<Option<String>>,
        platform: RefCell<Option<String>>,
        date: RefCell<Option<String>>,
        url: RefCell<Option<String>>,
        thumbnail_url: RefCell<Option<String>>,
        local_path: RefCell<Option<String>>,
        duration_fmt: RefCell<Option<String>>,

        playing: Cell<bool>,
        downloading: Cell<bool>,

        pub(super) duration: RefCell<Option<Duration>>,
        pub(super) video: RefCell<Option<AnyVideo>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VideoObject {
        const NAME: &'static str = "TFVideoObject";
        type Type = super::VideoObject;
        type ParentType = Object;
    }

    impl ObjectImpl for VideoObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    str_prop!("title"),
                    str_prop!("url"),
                    str_prop!("thumbnail-url"),
                    str_prop!("author"),
                    str_prop!("platform"),
                    str_prop!("date"),
                    str_prop!("local-path"),
                    str_prop!("duration"),
                    ParamSpecBoolean::builder("playing").build(),
                    ParamSpecBoolean::builder("downloading").build(),
                    ParamSpecBoolean::builder("is-local").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            if pspec.name() == "playing" {
                self.playing
                    .set(value.get().expect("Expect 'playing' to be a boolean."));
                return;
            }
            if pspec.name() == "downloading" {
                self.downloading
                    .set(value.get().expect("Expect 'downloading' to be a boolean."));
                return;
            }
            prop_set_all!(
                value,
                pspec,
                "title",
                self.title,
                "url",
                self.url,
                "thumbnail-url",
                self.thumbnail_url,
                "author",
                self.author,
                "platform",
                self.platform,
                "date",
                self.date,
                "local-path",
                self.local_path,
                "duration",
                self.duration_fmt
            );
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            if pspec.name() == "playing" {
                return self.playing.get().to_value();
            }
            if pspec.name() == "downloading" {
                return self.downloading.get().to_value();
            }
            if pspec.name() == "is-local" {
                return self.local_path.borrow().is_some().to_value();
            }
            prop_get_all!(
                pspec,
                "title",
                self.title,
                "url",
                self.url,
                "thumbnail-url",
                self.thumbnail_url,
                "author",
                self.author,
                "platform",
                self.platform,
                "date",
                self.date,
                "local-path",
                self.local_path,
                "duration",
                self.duration_fmt
            )
        }
    }
}
