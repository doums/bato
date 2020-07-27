/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "../include/notilus.h"

void notify(const char *summary, const char *body, const char *icon, NotifyUrgency urgency) {
    if (!notify_init("bato")) {
        fprintf(stderr, "bato error: in lib_notify, notify_init fails");
        return;
    }
    NotifyNotification *notification = notify_notification_new (summary, body, icon);
    notify_notification_set_urgency(notification, urgency);
    if (!notify_notification_show(notification, NULL)) {
        fprintf(stderr, "bato error: in lib_notify, notify_notification_show fails");
    }
    g_object_unref(G_OBJECT(notification));
    notify_uninit();
}
