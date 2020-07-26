#ifndef NOTILUS_H
#define NOTILUS_H

#include <stdio.h>
#include <libnotify/notify.h>
#include <libnotify/notification.h>

void notify(const char *summary, const char *body, const char *icon, NotifyUrgency urgency);

#endif //NOTILUS_H
