// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{Notification, Urgency};
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

#[repr(C)]
// for details see https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
pub struct NotifyNotification {
    _private: [u8; 0],
}

#[link(name = "notilus", kind = "static")]
extern "C" {
    pub fn init(app_name: *const c_char) -> *mut NotifyNotification;
    pub fn notify(
        notification: *mut NotifyNotification,
        summary: *const c_char,
        body: *const c_char,
        icon: *const c_char,
        urgency: Urgency,
    ) -> i32;
    pub fn uninit(notification: *mut NotifyNotification);
}

pub fn init_libnotilus(app_name: &str) -> *mut NotifyNotification {
    let notification;
    unsafe {
        notification = init(
            CString::new(app_name)
                .expect("CString::new failed")
                .as_ptr(),
        );
    }
    return notification;
}

pub fn send(notification: *mut NotifyNotification, data: &Notification) {
    let mut body = ptr::null();
    if let Some(v) = &data.body {
        body = v.as_ptr()
    }
    let mut icon = ptr::null();
    if let Some(v) = &data.icon {
        icon = v.as_ptr()
    }
    let mut urgency = Urgency::Normal;
    if let Some(v) = &data.urgency {
        urgency = *v
    }
    let i;
    unsafe {
        i = notify(notification, data.summary.as_ptr(), body, icon, urgency);
    }
    match i {
        1 => eprintln!("bato error: in libnotilus, fail to update the notification"),
        2 => eprintln!("bato error: in libnotilus, fail to show the notification"),
        _ => {}
    };
}

pub fn close_libnotilus(notification: *mut NotifyNotification) {
    unsafe { uninit(notification) }
}
