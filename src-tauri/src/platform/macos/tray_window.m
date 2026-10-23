#import <AppKit/AppKit.h>
#import <dispatch/dispatch.h>

typedef void (*ThaaApplicationDeactivationCallback)(void *context);

/*
 * Tao applies NSWindow size and position changes asynchronously on the main
 * dispatch queue. Enqueueing the reveal on that same serial queue guarantees
 * those geometry changes run first. The block retains the window until reveal.
 * Activation is cooperative rather than forcing focus away from another app.
 * AppKit still owns application-level stacking and activation behavior; this
 * helper does not directly change Main Window visibility.
 */
void thaa_macos_order_window_front_after_pending_updates(void *window_pointer) {
    NSWindow *window = (__bridge NSWindow *)window_pointer;
    dispatch_async(dispatch_get_main_queue(), ^{
        [window makeKeyAndOrderFront:nil];
        [[NSApplication sharedApplication] activate];
    });
}

/*
 * A Tauri window focus-loss event handles clicks that transfer key status to
 * another window in this app. App deactivation is an additional native signal
 * for clicks into another application or the desktop. This observes only the
 * application's lifecycle notification; it does not monitor global input.
 */
void *thaa_macos_observe_application_deactivation(
    ThaaApplicationDeactivationCallback callback,
    void *context) {
    if (callback == NULL) {
        return NULL;
    }

    id observer = [[NSNotificationCenter defaultCenter]
        addObserverForName:NSApplicationDidResignActiveNotification
                    object:NSApp
                     queue:nil
                usingBlock:^(NSNotification *notification) {
                    (void)notification;
                    callback(context);
                }];
    return (__bridge_retained void *)observer;
}

void thaa_macos_remove_application_deactivation_observer(void *observer_pointer) {
    if (observer_pointer == NULL) {
        return;
    }

    id observer = (__bridge_transfer id)observer_pointer;
    [[NSNotificationCenter defaultCenter] removeObserver:observer];
}
