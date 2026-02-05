// Register with NSAppleEventManager for kAEOpenDocuments so we handle drop-on-icon
// and "Open With" in the Cocoa chain and return success (avoiding the system error dialog).
// Calls back into Rust to push file paths to PENDING_FILES.

#import <Cocoa/Cocoa.h>
#import <ApplicationServices/ApplicationServices.h>
#import <objc/runtime.h>

extern void crtool_macos_push_pending_file(const char *path);

static void handle_open_document(id self, SEL _cmd, NSAppleEventDescriptor *event, NSAppleEventDescriptor *replyEvent) {
    (void)self;
    (void)_cmd;
    (void)replyEvent;

    // keyDirectObject = '----' = 0x2D2D2D2D
    static const AEKeyword keyDirectObject = 0x2D2D2D2D;

    NSAppleEventDescriptor *directObj = [event paramDescriptorForKeyword:keyDirectObject];
    if (!directObj) return;

    NSInteger count = [directObj numberOfItems];
    for (NSInteger i = 1; i <= count; i++) {
        NSAppleEventDescriptor *item = [directObj descriptorAtIndex:(int)i];
        if (!item) continue;

        // Try to get file URL (typeFileURL) - descriptorType is 'furl' or we get stringValue
        NSString *urlString = [item stringValue];
        if (!urlString || [urlString length] == 0) continue;

        NSURL *url = [NSURL URLWithString:urlString];
        if (!url) continue;

        if (![url isFileURL]) continue;

        const char *path = [[url path] UTF8String];
        if (path) crtool_macos_push_pending_file(path);
    }
}

static Class s_handlerClass = nil;
static id s_handlerInstance = nil;

void crtool_macos_install_open_document_handler(void) {
    if (s_handlerInstance != nil) return;

    // Create a subclass of NSObject that implements handleOpenDocument:withReplyEvent:
    s_handlerClass = objc_allocateClassPair([NSObject class], "CrtoolOpenDocumentHandler", 0);
    if (!s_handlerClass) return;

    SEL sel = @selector(handleOpenDocument:withReplyEvent:);
    IMP imp = (IMP)handle_open_document;
    class_addMethod(s_handlerClass, sel, imp, "v@:@@");

    objc_registerClassPair(s_handlerClass);

    s_handlerInstance = [[s_handlerClass alloc] init];
    if (!s_handlerInstance) return;

    NSAppleEventManager *aem = [NSAppleEventManager sharedAppleEventManager];
    [aem setEventHandler:s_handlerInstance
            andSelector:sel
         forEventClass:kCoreEventClass
            andEventID:kAEOpenDocuments];
}
