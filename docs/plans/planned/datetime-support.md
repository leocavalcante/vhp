# DateTime Support

## Status: Planned

## Overview

Implement comprehensive date and time functionality including DateTime, DateTimeImmutable, DateInterval, DatePeriod, and procedural date/time functions.

## Current Status

No date/time functions or classes are currently implemented.

## Background

Date and time handling is essential for PHP applications. Used extensively in:
- Scheduling and calendar applications
- Data analysis and reporting
- Timezone conversions
- Logging and timestamps
- Database queries with date ranges

## Requirements

### Procedural Functions

1. **time**
   ```php
   time(): int
   ```
   - Return current Unix timestamp

2. **mktime**
   ```php
   mktime($hour, $minute, $second, $month, $day, $year): int|false
   ```
   - Get Unix timestamp for a date

3. **strtotime**
   ```php
   strtotime($datetime, $baseTimestamp = null): int|false
   ```
   - Parse English textual datetime description into Unix timestamp
   - Support relative formats: "+1 day", "-2 weeks", "next Monday"

4. **date**
   ```php
   date($format, $timestamp = null): string
   ```
   - Format local time/date

5. **gmdate**
   ```php
   gmdate($format, $timestamp = null): string
   ```
   - Format GMT/UTC date/time

6. **idate**
   ```php
   idate($format, $timestamp = null): int|false
   ```
   - Format local time/date as integer

7. **strftime**
   ```php
   strftime($format, $timestamp = null): string|false
   ```
   - Format a local time/date according to locale settings
   - Note: Deprecated in PHP 8.1, but still commonly used

8. **gmstrftime**
   ```php
   gmstrftime($format, $timestamp = null): string|false
   ```
   - Format GMT/UTC time/date according to locale settings

9. **checkdate**
   ```php
   checkdate($month, $day, $year): bool
   ```
   - Validate a Gregorian date

10. **sleep**
    ```php
    sleep($seconds): int
    ```
    - Delay execution

11. **usleep**
    ```php
    usleep($microseconds): void
    ```
    - Delay execution in microseconds

12. **time_nanosleep**
    ```php
    time_nanosleep($seconds, $nanoseconds): array|bool
    ```
    - Delay for seconds and nanoseconds

13. **time_sleep_until**
    ```php
    time_sleep_until($timestamp): bool
    ```
    - Make script sleep until specified time

### DateTime Classes

1. **DateTime**
   ```php
   class DateTime implements DateTimeInterface {
       public function __construct($time = "now", $timezone = null)
       public function format($format): string
       public function modify($modifier): self
       public function add(DateInterval $interval): self
       public function sub(DateInterval $interval): self
       public function getTimezone(): DateTimeZone|false
       public function setTimezone(DateTimeZone $timezone): self
       public function getOffset(): int
       public function diff(DateTimeInterface $targetObject, $absolute = false): DateInterval
       public static function createFromFormat($format, $time, $timezone = null): self|false
       public static function createFromInterface(DateTimeInterface $object): self
       public static function getLastErrors(): array|false
       public function setTimestamp($unixtimestamp): self
       public function getTimestamp(): int
       public function setISODate($year, $week, $dayOfWeek = 1): self
       public function setDate($year, $month, $day): self
       public function setTime($hour, $minute, $second = 0, $microseconds = 0): self
       public function __wakeup(): void
       public function __serialize(): array
       public function __unserialize(array $data): void
   }
   ```

2. **DateTimeImmutable**
   ```php
   class DateTimeImmutable implements DateTimeInterface {
       // Same methods as DateTime, but all return new instances
       // modify(), add(), sub(), setTimezone() return new objects
   }
   ```

3. **DateTimeZone**
   ```php
   class DateTimeZone {
       public function __construct($timezone)
       public function getName(): string
       public function getOffset(DateTime $datetime): int
       public static function listIdentifiers(): array
       public function getTransitions($timestampBegin, $timestampEnd = null): array|null
       public function getLocation(): array|false
   }
   ```

4. **DateInterval**
   ```php
   class DateInterval {
       public $y;      // Years
       public $m;      // Months
       public $d;      // Days
       public $h;      // Hours
       public $i;      // Minutes
       public $s;      // Seconds
       public $f;      // Microseconds
       public $weekday;
       public $weekday_behavior;
       public $first_last_day_of;
       public $invert;
       public $days;
       public $special_type;
       public $special_amount;
       public $have_weekday_relative;
       public $have_special_relative;

       public static function createFromDateString($time): DateInterval|false
       public function format($format): string
   }
   ```

5. **DatePeriod**
   ```php
   class DatePeriod implements Iterator {
       public function __construct($start, $interval, $end, $options = 0)
       public static function createFromISO8601($isostring, $start = null, $end = null, $options = 0): self
   }
   ```

### DateTimeInterface Interface

```php
interface DateTimeInterface {
    public function format($format): string;
    public function getTimezone(): DateTimeZone|false;
    public function getOffset(): int;
    public function getTimestamp(): int;
    public function diff(DateTimeInterface $targetObject, $absolute = false): DateInterval;
    public function __wakeup(): void;
}
```

### Format Characters

Support all format characters for `date()`:

**Day:**
- `d` - Day of month, 2 digits with leading zeros
- `D` - Textual representation of day, three letters
- `j` - Day of month without leading zeros
- `l` - Full textual representation of day
- `N` - ISO-8601 numeric representation of day
- `S` - English ordinal suffix
- `w` - Numeric representation of day
- `z` - Day of year

**Week:**
- `W` - ISO-8601 week number

**Month:**
- `F` - Full textual representation of month
- `m` - Numeric representation of month with leading zeros
- `M` - Short textual representation of month
- `n` - Numeric representation of month without leading zeros
- `t` - Number of days in given month

**Year:**
- `L` - Leap year
- `o` - ISO-8601 year number
- `Y` - Full numeric representation of year
- `y` - 2-digit representation of year

**Time:**
- `a` - Lowercase Ante meridiem/Post meridiem
- `A` - Uppercase Ante meridiem/Post meridiem
- `B` - Swatch Internet time
- `g` - 12-hour format without leading zeros
- `G` - 24-hour format without leading zeros
- `h` - 12-hour format with leading zeros
- `H` - 24-hour format with leading zeros
- `i` - Minutes with leading zeros
- `s` - Seconds with leading zeros
- `u` - Microseconds
- `v` - Milliseconds

**Timezone:**
- `e` - Timezone identifier
- `I` - Daylight saving time
- `O` - Difference to Greenwich time in hours
- `P` - Difference to Greenwich time with colon
- `T` - Timezone abbreviation
- `Z` - Timezone offset in seconds

**Full Date/Time:**
- `c` - ISO 8601 date
- `r` - RFC 2822 formatted date
- `U` - Seconds since Unix Epoch

### strtotime Relative Formats

Support common relative formats:
- `+1 day`, `-2 weeks`, `+3 months`
- `next Monday`, `last Friday`
- `tomorrow`, `yesterday`
- `first day of next month`
- `last day of this month`

## Implementation Plan

### Option A: Use Chrono Crate (Recommended)

**Pros:**
- Comprehensive date/time library
- Well-maintained
- Good timezone support
- Rust-native

**Cons:**
- Need to implement PHP-specific format strings
- Need to wrap in PHP-compatible API

### Option B: Use Time Crate

**Pros:**
- Lightweight
- Simple API

**Cons:**
- Less comprehensive than chrono
- Limited timezone support

**Decision: Use chrono crate**

### Phase 1: Dependencies and Basic Infrastructure

**File:** `Cargo.toml`

```toml
[dependencies]
chrono = "0.4"
chrono-tz = "0.8"
```

**File:** `runtime/datetime/mod.rs` (new)

```rust
pub mod datetime;
pub mod datetimezone;
pub mod dateinterval;
pub mod dateperiod;
pub mod procedural;
```

**Tasks:**
- [ ] Add chrono and chrono-tz dependencies
- [ ] Create datetime module structure
- [ ] Set up basic date/time types

### Phase 2: Procedural Functions

**File:** `runtime/datetime/procedural.rs`

**Tasks:**
- [ ] Implement time()
- [ ] Implement date()
- [ ] Implement gmdate()
- [ ] Implement mktime()
- [ ] Implement checkdate()
- [ ] Implement strtotime() (basic formats)
- [ ] Add tests

### Phase 3: Format String Parsing

**File:** `runtime/datetime/format.rs` (new)

**Tasks:**
- [ ] Create format character parser
- [ ] Implement all format characters
- [ ] Handle escape sequences
- [ ] Add tests for each format character

### Phase 4: DateTime Class

**File:** `runtime/datetime/datetime.rs`

**Tasks:**
- [ ] Implement DateTime class structure
- [ ] Implement __construct()
- [ ] Implement format()
- [ ] Implement modify()
- [ ] Implement getTimestamp(), setTimestamp()
- [ ] Implement diff()
- [ ] Implement setDate(), setTime()
- [ ] Implement createFromFormat()
- [ ] Add tests

### Phase 5: DateTimeImmutable Class

**File:** `runtime/datetime/datetime.rs` (extend)

**Tasks:**
- [ ] Implement DateTimeImmutable class
- [ ] Ensure immutability (all methods return new instances)
- [ ] Add tests

### Phase 6: DateTimeZone Class

**File:** `runtime/datetime/datetimezone.rs`

**Tasks:**
- [ ] Implement DateTimeZone class
- [ ] Implement timezone database (using chrono-tz)
- [ ] Implement listIdentifiers()
- [ ] Implement getOffset()
- [ ] Add tests

### Phase 7: DateInterval Class

**File:** `runtime/datetime/dateinterval.rs`

**Tasks:**
- [ ] Implement DateInterval class structure
- [ ] Implement public properties (y, m, d, h, i, s, f)
- [ ] Implement format()
- [ ] Implement createFromDateString()
- [ ] Add tests

### Phase 8: DatePeriod Class

**File:** `runtime/datetime/dateperiod.rs`

**Tasks:**
- [ ] Implement DatePeriod class
- [ ] Implement Iterator interface
- [ ] Implement __construct() with various signatures
- [ ] Add tests

### Phase 9: Advanced Procedural Functions

**File:** `runtime/datetime/procedural.rs` (extend)

**Tasks:**
- [ ] Implement sleep()
- [ ] Implement usleep()
- [ ] Implement idate()
- [ ] Implement strftime() (with deprecation warning)
- [ ] Implement gmstrftime()
- [ ] Implement time_nanosleep()
- [ ] Implement time_sleep_until()
- [ ] Add tests

### Phase 10: strtotime Extended

**File:** `runtime/datetime/strtotime.rs` (new)

**Tasks:**
- [ ] Implement full strtotime parser
- [ ] Support relative formats
- [ ] Support timezone in strtotime
- [ ] Add comprehensive tests

### Phase 11: Integration and Edge Cases

**Tasks:**
- [ ] Handle daylight saving time transitions
- [ ] Handle leap years
- [ ] Handle invalid dates
- [ ] Handle timezone conversions
- [ ] Error messages matching PHP
- [ ] Add edge case tests

### Phase 12: Tests

**File:** `tests/datetime/` (new directory)

Test coverage:
- time(), date(), gmdate()
- mktime(), checkdate()
- strtotime() with various formats
- DateTime construction
- DateTime formatting
- DateTime manipulation (add, sub, modify)
- DateTime timezone handling
- DateTime diff
- DateTimeImmutable immutability
- DateTimeZone timezone operations
- DateInterval operations
- DatePeriod iteration
- Format characters
- Edge cases and errors

## Implementation Details

### Format String Parsing

```rust
fn format_datetime(dt: &DateTime<FixedOffset>, format: &str) -> String {
    let mut result = String::new();
    let mut chars = format.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(escaped) = chars.next() {
                result.push(escaped);
            }
        } else {
            match c {
                'Y' => result.push_str(&format!("{}", dt.year())),
                'm' => result.push_str(&format!("{:02}", dt.month())),
                'd' => result.push_str(&format!("{:02}", dt.day())),
                'H' => result.push_str(&format!("{:02}", dt.hour())),
                'i' => result.push_str(&format!("{:02}", dt.minute())),
                's' => result.push_str(&format!("{:02}", dt.second())),
                // ... all other format characters
                _ => result.push(c),
            }
        }
    }

    result
}
```

### strtotime Implementation

```rust
use chrono::Duration;
use chrono_tz::Tz;

fn strtotime(datetime: &str, base: Option<i64>) -> Result<i64, String> {
    // Parse relative formats
    // Handle "+1 day", "-2 weeks", etc.
    // Use chrono's parsing capabilities
}
```

### DateTime Class Integration

```rust
// In runtime/mod.rs
pub use datetime::{DateTime, DateTimeImmutable, DateTimeZone, DateInterval, DatePeriod};

// Register classes in class loader
```

## Dependencies

- `chrono = "0.4"` - Date/time library
- `chrono-tz = "0.8"` - Timezone database

## Testing Strategy

1. **Unit Tests**: Each function and method
2. **Integration Tests**: DateTime operations
3. **Timezone Tests**: Cross-timezone operations
4. **Edge Cases**: Leap years, DST transitions, invalid dates
5. **Compatibility Tests**: Match PHP 8.x behavior

## Success Criteria

- All procedural functions work correctly
- DateTime and DateTimeImmutable classes implemented
- All format characters supported
- strtotime works with common relative formats
- Timezone handling is correct
- All tests pass

## Performance Considerations

- Cache timezone lookups
- Avoid unnecessary datetime conversions
- Efficient string formatting
- Use chrono's optimizations

## Open Questions

- Should we implement strftime (deprecated) or skip it?
- How to handle locale-dependent formatting?
- Should we support all PHP timezone identifiers?

## References

- PHP DateTime documentation: https://www.php.net/manual/en/book.datetime.php
- Chrono documentation: https://docs.rs/chrono/
- strtotime format reference: https://www.php.net/manual/en/datetime.formats.relative.php

## Related Plans

- Time extension (future)
- Calendar functions (future)
