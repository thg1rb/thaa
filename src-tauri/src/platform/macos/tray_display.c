#include <CoreGraphics/CoreGraphics.h>
#include <math.h>
#include <stddef.h>
#include <stdint.h>

/*
 * tray-icon reports a physical cursor position, while macOS monitor geometry
 * is rooted in CoreGraphics' global screen coordinate space. Querying the
 * display under the current cursor avoids guessing which per-display backing
 * scale was used for the event coordinates.
 */
int32_t thaa_macos_cursor_display_bounds(
    double *x,
    double *y,
    double *width,
    double *height) {
    if (x == NULL || y == NULL || width == NULL || height == NULL) {
        return 0;
    }

    CGEventRef event = CGEventCreate(NULL);
    if (event == NULL) {
        return 0;
    }
    const CGPoint cursor = CGEventGetLocation(event);
    CFRelease(event);

    CGDirectDisplayID displays[1] = {0};
    uint32_t display_count = 0;
    if (CGGetDisplaysWithPoint(cursor, 1, displays, &display_count) != kCGErrorSuccess ||
        display_count == 0) {
        return 0;
    }

    const CGRect bounds = CGDisplayBounds(displays[0]);
    if (!isfinite(bounds.origin.x) || !isfinite(bounds.origin.y) ||
        !isfinite(bounds.size.width) || !isfinite(bounds.size.height) ||
        bounds.size.width <= 0 || bounds.size.height <= 0) {
        return 0;
    }

    *x = bounds.origin.x;
    *y = bounds.origin.y;
    *width = bounds.size.width;
    *height = bounds.size.height;
    return 1;
}
