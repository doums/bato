/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "../include/notilus.h"

NotifyNotification *init(const char *app_name) {
    if (!notify_init(app_name)) {
        return NULL;
    }
    return notify_notification_new(app_name, NULL, NULL);
}

int notify(NotifyNotification *notification, const char *summary, const char *body, const char *icon, NotifyUrgency urgency) {
    if (!notify_notification_update(notification, summary, body, icon)) {
        return 1;
    }
    notify_notification_set_urgency(notification, urgency);
    if (!notify_notification_show(notification, NULL)) {
        return 2;
    }
    return 0;
}

void uninit(NotifyNotification *notification) {
    g_object_unref(G_OBJECT(notification));
    notify_uninit();
}