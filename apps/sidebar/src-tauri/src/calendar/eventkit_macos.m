// EventKit stays inside the application: no helper process, credentials or server.
#import <Foundation/Foundation.h>
#import <EventKit/EventKit.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

static EKEventStore *store;

static NSString *stringValue(id value) {
    return [value isKindOfClass:NSString.class] ? value : nil;
}

static NSString *permissionStatus(void) {
    EKAuthorizationStatus status = [EKEventStore authorizationStatusForEntityType:EKEntityTypeEvent];
    if (@available(macOS 14.0, *)) {
        if (status == EKAuthorizationStatusFullAccess) return @"authorized";
        if (status == EKAuthorizationStatusWriteOnly) return @"writeOnly";
    } else {
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
        if (status == EKAuthorizationStatusAuthorized) return @"authorized";
#pragma clang diagnostic pop
    }
    if (status == EKAuthorizationStatusNotDetermined) return @"notDetermined";
    if (status == EKAuthorizationStatusRestricted) return @"restricted";
    return @"denied";
}

static void requireAccess(void) {
    if (![permissionStatus() isEqualToString:@"authorized"])
        @throw [NSException exceptionWithName:@"Calendar" reason:@"Kalenderzugriff fehlt. Unter Systemeinstellungen → Datenschutz & Sicherheit → Kalender erlauben." userInfo:nil];
    if (!store) store = [[EKEventStore alloc] init];
}

static EKCalendar *calendarFor(NSString *identifier) {
    EKCalendar *calendar = [store calendarWithIdentifier:identifier];
    if (!calendar || !calendar.allowsContentModifications)
        @throw [NSException exceptionWithName:@"Calendar" reason:@"Ein ausgewählter Kalender fehlt oder ist schreibgeschützt. Kalenderauswahl prüfen." userInfo:nil];
    return calendar;
}

static NSDateFormatter *dayFormatter(void) {
    NSDateFormatter *formatter = [[NSDateFormatter alloc] init];
    formatter.locale = [[NSLocale alloc] initWithLocaleIdentifier:@"en_US_POSIX"];
    formatter.calendar = [[NSCalendar alloc] initWithCalendarIdentifier:NSCalendarIdentifierGregorian];
    formatter.timeZone = NSTimeZone.localTimeZone;
    formatter.dateFormat = @"yyyy-MM-dd";
    formatter.lenient = NO;
    return formatter;
}

static NSString *marker(NSDictionary *ref) {
    return [@"todosidebar://task/" stringByAppendingString:ref[@"taskId"]];
}

static NSISO8601DateFormatter *instantFormatter(BOOL fractional) {
    NSISO8601DateFormatter *formatter = [[NSISO8601DateFormatter alloc] init];
    formatter.formatOptions = NSISO8601DateFormatWithInternetDateTime |
        (fractional ? NSISO8601DateFormatWithFractionalSeconds : 0);
    return formatter;
}

static NSDate *instantValue(id value) {
    NSString *text = stringValue(value);
    return text ? ([instantFormatter(YES) dateFromString:text] ?: [instantFormatter(NO) dateFromString:text]) : nil;
}

static BOOL belongsTo(EKEvent *event, EKCalendar *calendar) {
    return event && [event.calendar.calendarIdentifier isEqualToString:calendar.calendarIdentifier];
}

static EKEvent *findEvent(NSDictionary *ref, EKCalendar *calendar) {
    NSString *eventId = stringValue(ref[@"eventId"]);
    EKEvent *event = eventId ? [store eventWithIdentifier:eventId] : nil;
    if (belongsTo(event, calendar)) return event;
    NSString *externalId = stringValue(ref[@"externalId"]);
    if (externalId.length) {
        for (EKCalendarItem *item in [store calendarItemsWithExternalIdentifier:externalId]) {
            if ([item isKindOfClass:EKEvent.class] && belongsTo((EKEvent *)item, calendar)) return (EKEvent *)item;
        }
    }
    // Recover a save interrupted before its link reached SQLite. Searching the
    // previous and desired day also survives identifiers changing during sync.
    for (id rawDay in ref[@"searchDates"] ?: @[]) {
        NSString *day = stringValue(rawDay);
        NSDate *date = day ? [dayFormatter() dateFromString:day] : nil;
        if (!date) continue;
        NSCalendar *gregorian = dayFormatter().calendar;
        NSDate *start = [gregorian dateByAddingUnit:NSCalendarUnitDay value:-1 toDate:date options:0];
        NSDate *end = [gregorian dateByAddingUnit:NSCalendarUnitDay value:2 toDate:date options:0];
        NSPredicate *predicate = [store predicateForEventsWithStartDate:start endDate:end calendars:@[calendar]];
        for (EKEvent *candidate in [store eventsMatchingPredicate:predicate]) {
            if ([candidate.URL.absoluteString isEqualToString:marker(ref)]) return candidate;
        }
    }
    return nil;
}

static NSDictionary *eventData(EKEvent *event) {
    NSDateFormatter *formatter = dayFormatter();
    if (event.hasRecurrenceRules)
        @throw [NSException exceptionWithName:@"Calendar" reason:[NSString stringWithFormat:@"„%@“: Wiederholungen werden noch nicht unterstützt. Wiederholung im Kalender entfernen. Diese Aufgabe wird bis dahin übersprungen.", event.title ?: @"Aufgabe"] userInfo:nil];
    if (!event.startDate || !event.endDate || [event.endDate compare:event.startDate] != NSOrderedDescending)
        @throw [NSException exceptionWithName:@"Calendar" reason:[NSString stringWithFormat:@"„%@“: Ende muss nach Beginn liegen. Diese Aufgabe wird übersprungen.", event.title ?: @"Aufgabe"] userInfo:nil];
    NSString *date = [formatter stringFromDate:event.startDate];
    // macOS returns the last day's 23:59:59. Subtracting a second also accepts
    // providers exposing an exclusive midnight boundary without adding a day.
    NSDate *lastDate = event.allDay ? [event.endDate dateByAddingTimeInterval:-1] : event.endDate;
    NSString *endsOn = [formatter stringFromDate:lastDate];
    NSString *notes = event.notes.length ? event.notes : nil;
    return @{@"title": event.title ?: @"", @"notes": notes ?: NSNull.null,
                   @"date": date, @"endsOn": [endsOn isEqualToString:date] ? NSNull.null : endsOn,
                   @"startsAt": event.allDay ? NSNull.null : [instantFormatter(YES) stringFromDate:event.startDate],
                   @"endsAt": event.allDay ? NSNull.null : [instantFormatter(YES) stringFromDate:event.endDate]};
}

static NSDictionary *eventValue(EKEvent *event) {
    return @{
        @"eventId": event.eventIdentifier,
        @"externalId": event.calendarItemExternalIdentifier ?: NSNull.null,
        @"data": eventData(event),
        @"modifiedAt": @([event.lastModifiedDate timeIntervalSince1970])
    };
}

static void applySchedule(EKEvent *event, NSDictionary *data) {
    NSDate *date = [dayFormatter() dateFromString:data[@"date"]];
    if (!date) @throw [NSException exceptionWithName:@"Calendar" reason:@"Ungültiges Kalenderdatum." userInfo:nil];
    NSCalendar *gregorian = dayFormatter().calendar;
    gregorian.timeZone = NSTimeZone.localTimeZone;
    NSDate *startsAt = instantValue(data[@"startsAt"]);
    NSDate *endsAt = instantValue(data[@"endsAt"]);
    BOOL timed = stringValue(data[@"startsAt"]) || stringValue(data[@"endsAt"]);
    if (timed) {
        if (!startsAt || !endsAt || [endsAt compare:startsAt] != NSOrderedDescending)
            @throw [NSException exceptionWithName:@"Calendar" reason:@"Beginn und Ende des Termins sind ungültig." userInfo:nil];
        event.allDay = NO;
        event.timeZone = NSTimeZone.localTimeZone;
        event.startDate = startsAt;
        event.endDate = endsAt;
    } else {
        NSString *endDay = stringValue(data[@"endsOn"]) ?: data[@"date"];
        NSDate *lastDay = [dayFormatter() dateFromString:endDay];
        if (!lastDay || [lastDay compare:date] == NSOrderedAscending)
            @throw [NSException exceptionWithName:@"Calendar" reason:@"Enddatum liegt vor Beginn." userInfo:nil];
        event.allDay = YES;
        event.timeZone = nil;
        event.startDate = date;
        NSDate *boundary = [gregorian dateByAddingUnit:NSCalendarUnitDay value:1 toDate:lastDay options:0];
        // EKEvent normalizes the supplied civil end day to 23:59:59. Passing
        // next midnight would include that next day and make a two-day event.
        event.endDate = [boundary dateByAddingTimeInterval:-1];
    }
}

static void checkRevision(EKEvent *event, NSDictionary *request) {
    id expected = request[@"expectedModifiedAt"];
    if ([expected isKindOfClass:NSNumber.class]) {
        if (!event || fabs([event.lastModifiedDate timeIntervalSince1970] - [expected doubleValue]) > 0.001)
            @throw [NSException exceptionWithName:@"Calendar" reason:@"Kalendertermin wurde während der Synchronisierung geändert. Beim nächsten Durchlauf erneut versuchen." userInfo:nil];
    } else if (event) {
        @throw [NSException exceptionWithName:@"Calendar" reason:@"Kalendertermin ist inzwischen erschienen. Beim nächsten Durchlauf erneut prüfen." userInfo:nil];
    }
}

static id perform(NSDictionary *request) {
    NSString *operation = request[@"operation"];
    if ([operation isEqualToString:@"status"]) return permissionStatus();
    if ([operation isEqualToString:@"authorize"]) {
        NSString *status = permissionStatus();
        if (![status isEqualToString:@"notDetermined"] && ![status isEqualToString:@"writeOnly"]) return status;
        if (!store) store = [[EKEventStore alloc] init];
        dispatch_semaphore_t done = dispatch_semaphore_create(0);
        __block NSError *accessError;
        __block BOOL accessGranted = NO;
        void (^completion)(BOOL, NSError *) = ^(BOOL granted, NSError *error) {
            accessGranted = granted;
            accessError = error;
            dispatch_semaphore_signal(done);
        };
        if (@available(macOS 14.0, *)) [store requestFullAccessToEventsWithCompletion:completion];
        else {
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
            [store requestAccessToEntityType:EKEntityTypeEvent completion:completion];
#pragma clang diagnostic pop
        }
        // Executed on a Rust worker, leaving the application's main loop free.
        if (dispatch_semaphore_wait(done, dispatch_time(DISPATCH_TIME_NOW, 120 * NSEC_PER_SEC)) != 0)
            @throw [NSException exceptionWithName:@"Calendar" reason:@"Kalenderfreigabe dauert zu lange. Freigabe abschließen und erneut versuchen." userInfo:nil];
        if (accessError) @throw [NSException exceptionWithName:@"Calendar" reason:accessError.localizedDescription userInfo:nil];
        NSString *result = permissionStatus();
        if (!accessGranted && [result isEqualToString:@"notDetermined"])
            @throw [NSException exceptionWithName:@"Calendar" reason:@"macOS konnte keinen Kalenderdialog anzeigen. Die aktualisierte todoSideBar-App installieren und vollständig neu starten, dann erneut freigeben." userInfo:nil];
        return result;
    }
    requireAccess();
    if ([operation isEqualToString:@"calendars"]) {
        [store reset];
        NSMutableArray *values = [NSMutableArray array];
        for (EKCalendar *calendar in [store calendarsForEntityType:EKEntityTypeEvent]) {
            [values addObject:@{@"id": calendar.calendarIdentifier, @"title": calendar.title,
                @"account": calendar.source.title ?: @"Kalender", @"writable": @(calendar.allowsContentModifications)}];
        }
        return values;
    }
    if ([operation isEqualToString:@"read"]) {
        [store reset];
        // Resolve all selected calendars before interpreting any missing events.
        for (NSString *identifier in request[@"calendarIds"]) calendarFor(identifier);
        NSMutableArray *values = [NSMutableArray array];
        for (NSDictionary *ref in request[@"refs"]) {
            @try {
                EKEvent *event = findEvent(ref, calendarFor(ref[@"calendarId"]));
                [values addObject:@{@"taskId": ref[@"taskId"], @"calendarId": ref[@"calendarId"],
                    @"event": event ? eventValue(event) : NSNull.null}];
            } @catch (NSException *exception) {
                [values addObject:@{@"taskId": ref[@"taskId"], @"calendarId": ref[@"calendarId"],
                    @"event": NSNull.null, @"error": exception.reason ?: @"Termin konnte nicht gelesen werden."}];
            }
        }
        return values;
    }
    EKCalendar *calendar = calendarFor(request[@"calendarId"]);
    // Reload objects before writes to detect edits arriving after the read pass.
    [store reset];
    calendar = calendarFor(request[@"calendarId"]);
    EKEvent *event = findEvent(request, calendar);
    if ([operation isEqualToString:@"delete"]) {
        if (!event) return NSNull.null;
        checkRevision(event, request);
        NSError *error;
        if (![store removeEvent:event span:EKSpanThisEvent commit:YES error:&error])
            @throw [NSException exceptionWithName:@"Calendar" reason:error.localizedDescription userInfo:nil];
        return NSNull.null;
    }
    if ([operation isEqualToString:@"save"]) {
        checkRevision(event, request);
        NSDictionary *data = request[@"data"];
        if (!event) event = [EKEvent eventWithEventStore:store];
        event.calendar = calendar;
        event.title = data[@"title"];
        event.notes = stringValue(data[@"notes"]);
        applySchedule(event, data);
        event.URL = [NSURL URLWithString:marker(request)];
        NSError *error;
        if (![store saveEvent:event span:EKSpanThisEvent commit:YES error:&error])
            @throw [NSException exceptionWithName:@"Calendar" reason:error.localizedDescription userInfo:nil];
        return eventValue(event);
    }
    @throw [NSException exceptionWithName:@"Calendar" reason:@"Unbekannter Kalenderaufruf." userInfo:nil];
}

// Rust serializes all calls, including status, using CalendarState::gate.
char *todosidebar_calendar_call(const char *input) {
    @autoreleasepool {
        NSDictionary *response;
        @try {
            NSData *data = [[NSString stringWithUTF8String:input] dataUsingEncoding:NSUTF8StringEncoding];
            NSDictionary *request = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];
            response = @{@"ok": perform(request) ?: NSNull.null};
        } @catch (NSException *exception) {
            response = @{@"error": exception.reason ?: @"Kalenderzugriff fehlgeschlagen."};
        }
        NSData *json = [NSJSONSerialization dataWithJSONObject:response options:NSJSONWritingFragmentsAllowed error:nil];
        return strdup([[NSString alloc] initWithData:json encoding:NSUTF8StringEncoding].UTF8String);
    }
}

void todosidebar_calendar_free(char *value) { free(value); }
