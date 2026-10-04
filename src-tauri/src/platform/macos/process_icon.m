#import <AppKit/AppKit.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define THAA_ICON_SIDE 64
#define THAA_ICON_MAX_BYTES (64 * 1024)

/* Returns PNG bytes for a public NSRunningApplication icon, or zero if none. */
int32_t thaa_macos_process_icon_png(int32_t pid, uint8_t **output, size_t *output_length) {
    if (pid <= 0 || output == NULL || output_length == NULL) {
        return 0;
    }

    *output = NULL;
    *output_length = 0;

    @autoreleasepool {
        NSRunningApplication *application =
            [NSRunningApplication runningApplicationWithProcessIdentifier:(pid_t)pid];
        NSImage *icon = application.icon;
        if (icon == nil) {
            return 0;
        }

        NSBitmapImageRep *bitmap = [[NSBitmapImageRep alloc]
            initWithBitmapDataPlanes:NULL
            pixelsWide:THAA_ICON_SIDE
            pixelsHigh:THAA_ICON_SIDE
            bitsPerSample:8
            samplesPerPixel:4
            hasAlpha:YES
            isPlanar:NO
            colorSpaceName:NSDeviceRGBColorSpace
            bitmapFormat:NSBitmapFormatAlphaNonpremultiplied
            bytesPerRow:0
            bitsPerPixel:0];
        if (bitmap == nil) {
            return 0;
        }

        NSGraphicsContext *context = [NSGraphicsContext graphicsContextWithBitmapImageRep:bitmap];
        if (context == nil) {
            return 0;
        }

        [NSGraphicsContext saveGraphicsState];
        [NSGraphicsContext setCurrentContext:context];
        [icon drawInRect:NSMakeRect(0, 0, THAA_ICON_SIDE, THAA_ICON_SIDE)
                fromRect:NSZeroRect
               operation:NSCompositingOperationCopy
                fraction:1.0];
        [context flushGraphics];
        [NSGraphicsContext restoreGraphicsState];

        NSData *png = [bitmap representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
        if (png == nil || png.length == 0 || png.length > THAA_ICON_MAX_BYTES) {
            return 0;
        }

        uint8_t *copy = malloc(png.length);
        if (copy == NULL) {
            return 0;
        }
        memcpy(copy, png.bytes, png.length);
        *output = copy;
        *output_length = png.length;
        return 1;
    }
}

void thaa_macos_process_icon_free(uint8_t *bytes) {
    free(bytes);
}
