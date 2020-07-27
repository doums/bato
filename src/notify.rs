// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{Notification, Urgency};
use std::os::raw::c_char;
use std::ptr;

#[derive(Debug)]
#[repr(C)]
pub enum NotifyUrgency {
    NOTIFY_URGENCY_LOW,
    NOTIFY_URGENCY_NORMAL,
    NOTIFY_URGENCY_CRITICAL,
}

impl From<&Option<Urgency>> for NotifyUrgency {
    fn from(urgency: &Option<Urgency>) -> Self {
        return if let Some(v) = urgency {
            match v {
                Urgency::Low => NotifyUrgency::NOTIFY_URGENCY_LOW,
                Urgency::Normal => NotifyUrgency::NOTIFY_URGENCY_NORMAL,
                Urgency::Critical => NotifyUrgency::NOTIFY_URGENCY_CRITICAL,
            }
        } else {
            NotifyUrgency::NOTIFY_URGENCY_LOW
        };
    }
}

#[link(name = "notilus", kind = "static")]
extern "C" {
    pub fn notify(
        summary: *const c_char,
        body: *const c_char,
        icon: *const c_char,
        urgency: NotifyUrgency,
    );
}

pub fn send(notification: &Notification) {
    let mut body = ptr::null();
    if let Some(v) = &notification.body {
        body = v.as_ptr()
    }
    let mut icon = ptr::null();
    if let Some(v) = &notification.icon {
        icon = v.as_ptr()
    }
    let urgency = NotifyUrgency::from(&notification.urgency);
    unsafe {
        notify(notification.summary.as_ptr(), body, icon, urgency);
    }
}
