// In-memory EventKit objects only. No permissions, account reads or saved events.
#import "../src/calendar/eventkit_macos.m"
#include <assert.h>

int main(void) {
    @autoreleasepool {
        [NSTimeZone setDefaultTimeZone:[NSTimeZone timeZoneWithName:@"Europe/Berlin"]];
        EKEventStore *memoryStore = [[EKEventStore alloc] init];
        EKEvent *event = [EKEvent eventWithEventStore:memoryStore];
        event.title = @"Fixture";
        // Single-day all-day events cross 23h and 25h days correctly.
        for (NSString *day in @[@"2026-03-29", @"2026-10-25"]) {
            applySchedule(event, @{@"date": day});
            NSDictionary *data = eventData(event);
            assert([data[@"date"] isEqual:day]);
            assert(data[@"endsOn"] == NSNull.null && data[@"startsAt"] == NSNull.null);
            double hours = ([event.endDate timeIntervalSinceDate:event.startDate] + 1) / 3600;
            assert(hours == ([day isEqual:@"2026-03-29"] ? 23 : 25));
        }
        applySchedule(event, @{@"date": @"2026-03-28", @"endsOn": @"2026-03-30"});
        assert([eventData(event)[@"endsOn"] isEqual:@"2026-03-30"]);
        assert([dayFormatter() stringFromDate:event.endDate].length > 0);
        assert([[dayFormatter() stringFromDate:event.endDate] isEqual:@"2026-03-30"]);
        // Timed input accepts chrono RFC3339 with/without fractional seconds.
        NSDictionary *timed = @{@"date": @"2026-09-28", @"endsOn": @"2026-09-29",
            @"startsAt": @"2026-09-28T21:30:00Z", @"endsAt": @"2026-09-28T23:00:00.000Z"};
        applySchedule(event, timed);
        NSDictionary *data = eventData(event);
        assert(!event.allDay);
        assert([data[@"date"] isEqual:timed[@"date"]] && [data[@"endsOn"] isEqual:timed[@"endsOn"]]);
        assert([instantValue(data[@"startsAt"]) isEqual:event.startDate]);
        assert([instantValue(data[@"endsAt"]) isEqual:event.endDate]);
        assert([event.endDate timeIntervalSinceDate:event.startDate] == 90 * 60);
        // A repeated local hour retains the two distinct absolute instants.
        applySchedule(event, @{@"date": @"2026-10-25", @"startsAt": @"2026-10-25T00:30:00Z", @"endsAt": @"2026-10-25T01:30:00Z"});
        assert([event.endDate timeIntervalSinceDate:event.startDate] == 3600);
        assert(eventData(event)[@"endsOn"] == NSNull.null);
        // Precise recurrence message; ordinary timed events are allowed.
        [event addRecurrenceRule:[[EKRecurrenceRule alloc] initRecurrenceWithFrequency:EKRecurrenceFrequencyDaily interval:1 end:nil]];
        BOOL rejected = NO;
        @try { eventData(event); } @catch (NSException *error) { rejected = [error.reason containsString:@"Wiederholungen"]; }
        assert(rejected);
        puts("Native calendar tests passed: DST, multi-day, timed, recurrence.");
    }
    return 0;
}
