#[cfg(not(feature = "std"))]
use crate::alloc_prelude::*;
#[cfg(feature = "std")]
use crate::Sign;
use crate::{
    format::parse::{parse, ParseResult, ParsedItems},
    time, Date, DeferredFormat, Duration, OffsetDateTime, Time, UtcOffset, Weekday,
};
#[cfg(feature = "std")]
use core::convert::{From, TryFrom};
use core::{
    cmp::Ordering,
    ops::{Add, AddAssign, Sub, SubAssign},
    time::Duration as StdDuration,
};
#[cfg(feature = "std")]
use std::time::SystemTime;

/// Combined date and time.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(
        try_from = "crate::serde::PrimitiveDateTime",
        into = "crate::serde::PrimitiveDateTime"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PrimitiveDateTime {
    #[allow(clippy::missing_docs_in_private_items)]
    pub(crate) date: Date,
    #[allow(clippy::missing_docs_in_private_items)]
    pub(crate) time: Time,
}

impl PrimitiveDateTime {
    /// Create a new `PrimitiveDateTime` from the provided `Date` and `Time`.
    ///
    /// ```rust
    /// # use time::{PrimitiveDateTime, time, date};
    /// assert_eq!(
    ///     PrimitiveDateTime::new(date!(2019-01-01), time!(0:00)),
    ///     date!(2019-01-01).midnight(),
    /// );
    /// ```
    #[inline(always)]
    pub const fn new(date: Date, time: Time) -> Self {
        Self { date, time }
    }

    /// Create a new `PrimitiveDateTime` with the current date and time (UTC).
    ///
    /// ```rust
    /// # use time::PrimitiveDateTime;
    /// assert!(PrimitiveDateTime::now().year() >= 2019);
    /// ```
    #[inline(always)]
    #[cfg(feature = "std")]
    #[cfg_attr(doc, doc(cfg(feature = "std")))]
    pub fn now() -> Self {
        SystemTime::now().into()
    }

    /// Midnight, 1 January, 1970 (UTC).
    ///
    /// ```rust
    /// # use time::{PrimitiveDateTime, date};
    /// assert_eq!(
    ///     PrimitiveDateTime::unix_epoch(),
    ///     date!(1970-01-01).midnight()
    /// );
    /// ```
    #[inline(always)]
    pub const fn unix_epoch() -> Self {
        Self {
            // TODO Use `date!(1970-001)` when rustfmt can handle it.
            date: Date {
                year: 1970,
                ordinal: 1,
            },
            time: time!(0:00),
        }
    }

    /// Create a `PrimitiveDateTime` from the provided [Unix timestamp](https://en.wikipedia.org/wiki/Unix_time).
    ///
    /// ```rust
    /// # use time::{date, PrimitiveDateTime};
    /// assert_eq!(
    ///     PrimitiveDateTime::from_unix_timestamp(0),
    ///     PrimitiveDateTime::unix_epoch()
    /// );
    /// assert_eq!(
    ///     PrimitiveDateTime::from_unix_timestamp(1_546_300_800),
    ///     date!(2019-01-01).midnight(),
    /// );
    /// ```
    #[inline(always)]
    pub fn from_unix_timestamp(timestamp: i64) -> Self {
        Self::unix_epoch() + Duration::seconds(timestamp)
    }

    /// Get the [Unix timestamp](https://en.wikipedia.org/wiki/Unix_time)
    /// representing the `PrimitiveDateTime`.
    ///
    /// ```rust
    /// # use time::{date, PrimitiveDateTime};
    /// assert_eq!(PrimitiveDateTime::unix_epoch().timestamp(), 0);
    /// assert_eq!(date!(2019-01-01).midnight().timestamp(), 1_546_300_800);
    /// ```
    #[inline(always)]
    pub fn timestamp(self) -> i64 {
        (self - Self::unix_epoch()).whole_seconds()
    }

    /// Get the `Date` component of the `PrimitiveDateTime`.
    ///
    /// ```rust
    /// # use time::date;
    /// assert_eq!(
    ///     date!(2019-01-01).midnight().date(),
    ///     date!(2019-01-01)
    /// );
    /// ```
    #[inline(always)]
    pub const fn date(self) -> Date {
        self.date
    }

    /// Get the `Time` component of the `PrimitiveDateTime`.
    ///
    /// ```rust
    /// # use time::{date, time};
    /// assert_eq!(date!(2019-01-01).midnight().time(), time!(0:00));
    #[inline(always)]
    pub const fn time(self) -> Time {
        self.time
    }

    /// Get the year of the date.
    ///
    /// ```rust
    /// # use time::date;
    /// assert_eq!(date!(2019-01-01).midnight().year(), 2019);
    /// assert_eq!(date!(2019-12-31).midnight().year(), 2019);
    /// assert_eq!(date!(2020-01-01).midnight().year(), 2020);
    /// ```
    #[inline(always)]
    pub fn year(self) -> i32 {
        self.date().year()
    }

    /// Get the month of the date. If fetching both the month and day, it is
    /// more efficient to use [`PrimitiveDateTime::month_day`].
    ///
    /// The returned value will always be in the range `1..=12`.
    ///
    /// ```rust
    /// # use time::date;
    /// assert_eq!(date!(2019-01-01).midnight().month(), 1);
    /// assert_eq!(date!(2019-12-31).midnight().month(), 12);
    /// ```
    #[inline(always)]
    pub fn month(self) -> u8 {
        self.date().month()
    }

    /// Get the day of the date.  If fetching both the month and day, it is
    /// more efficient to use [`PrimitiveDateTime::month_day`].
    ///
    /// The returned value will always be in the range `1..=31`.
    ///
    /// ```rust
    /// # use time::date;
    /// assert_eq!(date!(2019-1-1).midnight().day(), 1);
    /// assert_eq!(date!(2019-12-31).midnight().day(), 31);
    /// ```
    #[inline(always)]
    pub fn day(self) -> u8 {
        self.date().day()
    }

    /// Get the month and day of the date. This is more efficient than fetching
    /// the components individually.
    ///
    /// The month component will always be in the range `1..=12`;
    /// the day component in `1..=31`.
    ///
    /// ```rust
    /// # use time::date;
    /// assert_eq!(date!(2019-01-01).midnight().month_day(), (1, 1));
    /// assert_eq!(date!(2019-12-31).midnight().month_day(), (12, 31));
    /// ```
    #[inline(always)]
    pub fn month_day(self) -> (u8, u8) {
        self.date().month_day()
    }

    /// Get the day of the year.
    ///
    /// The returned value will always be in the range `1..=366` (`1..=365` for
    /// common years).
    ///
    /// ```rust
    /// # use time::date;
    /// assert_eq!(date!(2019-01-01).midnight().ordinal(), 1);
    /// assert_eq!(date!(2019-12-31).midnight().ordinal(), 365);
    /// ```
    #[inline(always)]
    pub fn ordinal(self) -> u16 {
        self.date().ordinal()
    }

    /// Get the ISO 8601 year and week number.
    ///
    /// ```rust
    /// # use time::date;
    /// assert_eq!(date!(2019-01-01).midnight().iso_year_week(), (2019, 1));
    /// assert_eq!(date!(2019-10-04).midnight().iso_year_week(), (2019, 40));
    /// assert_eq!(date!(2020-01-01).midnight().iso_year_week(), (2020, 1));
    /// assert_eq!(date!(2020-12-31).midnight().iso_year_week(), (2020, 53));
    /// assert_eq!(date!(2021-01-01).midnight().iso_year_week(), (2020, 53));
    /// ```
    #[inline]
    pub fn iso_year_week(self) -> (i32, u8) {
        self.date().iso_year_week()
    }

    /// Get the ISO week number.
    ///
    /// The returned value will always be in the range `1..=53`.
    ///
    /// ```rust
    /// # use time::date;
    /// assert_eq!(date!(2019-01-01).midnight().week(), 1);
    /// assert_eq!(date!(2019-10-04).midnight().week(), 40);
    /// assert_eq!(date!(2020-01-01).midnight().week(), 1);
    /// assert_eq!(date!(2020-12-31).midnight().week(), 53);
    /// assert_eq!(date!(2021-01-01).midnight().week(), 53);
    /// ```
    #[inline(always)]
    pub fn week(self) -> u8 {
        self.date().week()
    }

    /// Get the week number where week 1 begins on the first Sunday.
    ///
    /// The returned value will always be in the range `0..=53`.
    ///
    /// ```rust
    /// # use time::date;
    /// assert_eq!(date!(2019-01-01).midnight().sunday_based_week(), 0);
    /// assert_eq!(date!(2020-01-01).midnight().sunday_based_week(), 0);
    /// assert_eq!(date!(2020-12-31).midnight().sunday_based_week(), 52);
    /// assert_eq!(date!(2021-01-01).midnight().sunday_based_week(), 0);
    /// ```
    #[inline(always)]
    pub fn sunday_based_week(self) -> u8 {
        self.date().sunday_based_week()
    }

    /// Get the week number where week 1 begins on the first Monday.
    ///
    /// The returned value will always be in the range `0..=53`.
    ///
    /// ```rust
    /// # use time::date;
    /// assert_eq!(date!(2019-01-01).midnight().monday_based_week(), 0);
    /// assert_eq!(date!(2020-01-01).midnight().monday_based_week(), 0);
    /// assert_eq!(date!(2020-12-31).midnight().monday_based_week(), 52);
    /// assert_eq!(date!(2021-01-01).midnight().monday_based_week(), 0);
    /// ```
    #[inline(always)]
    pub fn monday_based_week(self) -> u8 {
        self.date().monday_based_week()
    }

    /// Get the weekday.
    ///
    /// This current uses [Zeller's congruence](https://en.wikipedia.org/wiki/Zeller%27s_congruence)
    /// internally.
    ///
    /// ```rust
    /// # use time::{date, Weekday::*};
    /// assert_eq!(date!(2019-01-01).midnight().weekday(), Tuesday);
    /// assert_eq!(date!(2019-02-01).midnight().weekday(), Friday);
    /// assert_eq!(date!(2019-03-01).midnight().weekday(), Friday);
    /// assert_eq!(date!(2019-04-01).midnight().weekday(), Monday);
    /// assert_eq!(date!(2019-05-01).midnight().weekday(), Wednesday);
    /// assert_eq!(date!(2019-06-01).midnight().weekday(), Saturday);
    /// assert_eq!(date!(2019-07-01).midnight().weekday(), Monday);
    /// assert_eq!(date!(2019-08-01).midnight().weekday(), Thursday);
    /// assert_eq!(date!(2019-09-01).midnight().weekday(), Sunday);
    /// assert_eq!(date!(2019-10-01).midnight().weekday(), Tuesday);
    /// assert_eq!(date!(2019-11-01).midnight().weekday(), Friday);
    /// assert_eq!(date!(2019-12-01).midnight().weekday(), Sunday);
    /// ```
    #[inline(always)]
    pub fn weekday(self) -> Weekday {
        self.date().weekday()
    }

    /// Get the clock hour.
    ///
    /// The returned value will always be in the range `0..24`.
    ///
    /// ```rust
    /// # use time::{date, time};
    /// assert_eq!(date!(2019-01-01).midnight().hour(), 0);
    /// assert_eq!(date!(2019-01-01).with_time(time!(23:59:59)).hour(), 23);
    /// ```
    #[inline(always)]
    pub const fn hour(self) -> u8 {
        self.time().hour()
    }

    /// Get the minute within the hour.
    ///
    /// The returned value will always be in the range `0..60`.
    ///
    /// ```rust
    /// # use time::{date, time};
    /// assert_eq!(date!(2019-01-01).midnight().minute(), 0);
    /// assert_eq!(date!(2019-01-01).with_time(time!(23:59:59)).minute(), 59);
    /// ```
    #[inline(always)]
    pub const fn minute(self) -> u8 {
        self.time().minute()
    }

    /// Get the second within the minute.
    ///
    /// The returned value will always be in the range `0..60`.
    ///
    /// ```rust
    /// # use time::{date, time};
    /// assert_eq!(date!(2019-01-01).midnight().second(), 0);
    /// assert_eq!(date!(2019-01-01).with_time(time!(23:59:59)).second(), 59);
    /// ```
    #[inline(always)]
    pub const fn second(self) -> u8 {
        self.time().second()
    }

    /// Get the milliseconds within the second.
    ///
    /// The returned value will always be in the range `0..1_000`.
    ///
    /// ```rust
    /// # use time::{date, time};
    /// assert_eq!(date!(2019-01-01).midnight().millisecond(), 0);
    /// assert_eq!(date!(2019-01-01).with_time(time!(23:59:59.999)).millisecond(), 999);
    /// ```
    #[inline(always)]
    pub const fn millisecond(self) -> u16 {
        self.time().millisecond()
    }

    /// Get the microseconds within the second.
    ///
    /// The returned value will always be in the range `0..1_000_000`.
    ///
    /// ```rust
    /// # use time::{date, time};
    /// assert_eq!(date!(2019-01-01).midnight().microsecond(), 0);
    /// assert_eq!(date!(2019-01-01).with_time(time!(23:59:59.999_999)).microsecond(), 999_999);
    /// ```
    #[inline(always)]
    pub const fn microsecond(self) -> u32 {
        self.time().microsecond()
    }

    /// Get the nanoseconds within the second.
    ///
    /// The returned value will always be in the range `0..1_000_000_000`.
    ///
    /// ```rust
    /// # use time::{date, time};
    /// assert_eq!(date!(2019-01-01).midnight().nanosecond(), 0);
    /// assert_eq!(
    ///     date!(2019-01-01).with_time(time!(23:59:59.999_999_999)).nanosecond(),
    ///     999_999_999,
    /// );
    /// ```
    #[inline(always)]
    pub const fn nanosecond(self) -> u32 {
        self.time().nanosecond()
    }

    /// Create an `OffsetDateTime` from the existing `PrimitiveDateTime` and provided
    /// `UtcOffset`.
    ///
    /// ```rust
    /// # use time::{date, offset};
    /// assert_eq!(
    ///     date!(2019-01-01).midnight().using_offset(offset!(UTC)).timestamp(),
    ///     1_546_300_800,
    /// );
    /// ```
    #[inline(always)]
    pub const fn using_offset(self, offset: UtcOffset) -> OffsetDateTime {
        OffsetDateTime {
            datetime: self,
            offset,
        }
    }
}

/// Methods that allow formatting the `PrimitiveDateTime`.
impl PrimitiveDateTime {
    /// Format the `PrimitiveDateTime` using the provided string.
    ///
    /// ```rust
    /// # use time::date;
    /// assert_eq!(
    ///     date!(2019-01-02).midnight().format("%F %r"),
    ///     "2019-01-02 12:00:00 am"
    /// );
    /// ```
    #[inline(always)]
    pub fn format(self, format: &str) -> String {
        DeferredFormat {
            date: Some(self.date()),
            time: Some(self.time()),
            offset: None,
            format: crate::format::parse_fmt_string(format),
        }
        .to_string()
    }

    /// Attempt to parse a `PrimitiveDateTime` using the provided string.
    ///
    /// ```rust
    /// # use time::{date, PrimitiveDateTime, Weekday::Wednesday, time};
    /// assert_eq!(
    ///     PrimitiveDateTime::parse("2019-01-02 00:00:00", "%F %T"),
    ///     Ok(date!(2019-01-02).midnight()),
    /// );
    /// assert_eq!(
    ///     PrimitiveDateTime::parse("2019-002 23:59:59", "%Y-%j %T"),
    ///     Ok(date!(2019-002).with_time(time!(23:59:59)))
    /// );
    /// assert_eq!(
    ///     PrimitiveDateTime::parse("2019-W01-3 12:00:00 pm", "%G-W%V-%u %r"),
    ///     Ok(date!(2019-W01-3).with_time(time!(12:00))),
    /// );
    /// ```
    #[inline(always)]
    pub fn parse(s: &str, format: &str) -> ParseResult<Self> {
        Self::try_from_parsed_items(parse(s, format)?)
    }

    /// Given the items already parsed, attempt to create a `PrimitiveDateTime`.
    #[inline(always)]
    pub(crate) fn try_from_parsed_items(items: ParsedItems) -> ParseResult<Self> {
        Ok(Self {
            date: Date::try_from_parsed_items(items)?,
            time: Time::try_from_parsed_items(items)?,
        })
    }
}

impl Add<Duration> for PrimitiveDateTime {
    type Output = Self;

    #[inline]
    fn add(self, duration: Duration) -> Self::Output {
        #[allow(clippy::cast_possible_truncation)]
        let nanos = self.time.nanoseconds_since_midnight() as i64
            + (duration.whole_nanoseconds() % 86_400_000_000_000) as i64;

        let date_modifier = if nanos < 0 {
            -Duration::day()
        } else if nanos >= 86_400_000_000_000 {
            Duration::day()
        } else {
            Duration::zero()
        };

        Self::new(self.date + duration + date_modifier, self.time + duration)
    }
}

#[cfg(feature = "std")]
impl Add<Duration> for SystemTime {
    type Output = Self;

    #[inline(always)]
    fn add(self, duration: Duration) -> Self::Output {
        match duration.sign_abs_std() {
            (Sign::Zero, _) => self,
            (Sign::Positive, duration) => self + duration,
            (Sign::Negative, duration) => self - duration,
        }
    }
}

impl Add<StdDuration> for PrimitiveDateTime {
    type Output = Self;

    #[inline(always)]
    fn add(self, duration: StdDuration) -> Self::Output {
        #[allow(clippy::cast_possible_truncation)]
        let nanos = self.time.nanoseconds_since_midnight()
            + (duration.as_nanos() % 86_400_000_000_000) as u64;

        let date_modifier = if nanos >= 86_400_000_000_000 {
            Duration::day()
        } else {
            Duration::zero()
        };

        Self::new(self.date + duration + date_modifier, self.time + duration)
    }
}

impl AddAssign<Duration> for PrimitiveDateTime {
    #[inline(always)]
    fn add_assign(&mut self, duration: Duration) {
        *self = *self + duration;
    }
}

impl AddAssign<StdDuration> for PrimitiveDateTime {
    #[inline(always)]
    fn add_assign(&mut self, duration: StdDuration) {
        *self = *self + duration;
    }
}

#[cfg(feature = "std")]
impl AddAssign<Duration> for SystemTime {
    #[inline(always)]
    fn add_assign(&mut self, duration: Duration) {
        *self = *self + duration;
    }
}

impl Sub<Duration> for PrimitiveDateTime {
    type Output = Self;

    #[inline(always)]
    fn sub(self, duration: Duration) -> Self::Output {
        self + -duration
    }
}

impl Sub<StdDuration> for PrimitiveDateTime {
    type Output = Self;

    #[inline(always)]
    fn sub(self, duration: StdDuration) -> Self::Output {
        #[allow(clippy::cast_possible_truncation)]
        let nanos = self.time.nanoseconds_since_midnight() as i64
            - (duration.as_nanos() % 86_400_000_000_000) as i64;

        let date_modifier = if nanos < 0 {
            -Duration::day()
        } else {
            Duration::zero()
        };

        Self::new(self.date - duration + date_modifier, self.time - duration)
    }
}

#[cfg(feature = "std")]
impl Sub<Duration> for SystemTime {
    type Output = Self;

    #[inline(always)]
    fn sub(self, duration: Duration) -> Self::Output {
        (PrimitiveDateTime::from(self) - duration).into()
    }
}

impl SubAssign<Duration> for PrimitiveDateTime {
    #[inline(always)]
    fn sub_assign(&mut self, duration: Duration) {
        *self = *self - duration;
    }
}

impl SubAssign<StdDuration> for PrimitiveDateTime {
    #[inline(always)]
    fn sub_assign(&mut self, duration: StdDuration) {
        *self = *self - duration;
    }
}

#[cfg(feature = "std")]
impl SubAssign<Duration> for SystemTime {
    #[inline(always)]
    fn sub_assign(&mut self, duration: Duration) {
        *self = *self - duration;
    }
}

impl Sub<PrimitiveDateTime> for PrimitiveDateTime {
    type Output = Duration;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        (self.date - rhs.date) + (self.time - rhs.time)
    }
}

#[cfg(feature = "std")]
impl Sub<SystemTime> for PrimitiveDateTime {
    type Output = Duration;

    #[inline(always)]
    fn sub(self, rhs: SystemTime) -> Self::Output {
        self - Self::from(rhs)
    }
}

#[cfg(feature = "std")]
impl Sub<PrimitiveDateTime> for SystemTime {
    type Output = Duration;

    #[inline(always)]
    fn sub(self, rhs: PrimitiveDateTime) -> Self::Output {
        PrimitiveDateTime::from(self) - rhs
    }
}

impl PartialOrd for PrimitiveDateTime {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "std")]
impl PartialEq<SystemTime> for PrimitiveDateTime {
    #[inline(always)]
    fn eq(&self, rhs: &SystemTime) -> bool {
        self == &Self::from(*rhs)
    }
}

#[cfg(feature = "std")]
impl PartialEq<PrimitiveDateTime> for SystemTime {
    #[inline(always)]
    fn eq(&self, rhs: &PrimitiveDateTime) -> bool {
        &PrimitiveDateTime::from(*self) == rhs
    }
}

#[cfg(feature = "std")]
impl PartialOrd<SystemTime> for PrimitiveDateTime {
    #[inline(always)]
    fn partial_cmp(&self, other: &SystemTime) -> Option<Ordering> {
        self.partial_cmp(&Self::from(*other))
    }
}

#[cfg(feature = "std")]
impl PartialOrd<PrimitiveDateTime> for SystemTime {
    #[inline(always)]
    fn partial_cmp(&self, other: &PrimitiveDateTime) -> Option<Ordering> {
        PrimitiveDateTime::from(*self).partial_cmp(other)
    }
}

impl Ord for PrimitiveDateTime {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match self.date.cmp(&other.date) {
            Ordering::Equal => match self.time.hour.cmp(&other.time.hour) {
                Ordering::Equal => match self.time.minute.cmp(&other.time.minute) {
                    Ordering::Equal => match self.time.second.cmp(&other.time.second) {
                        Ordering::Equal => self.time.nanosecond.cmp(&other.time.nanosecond),
                        other => other,
                    },
                    other => other,
                },
                other => other,
            },
            other => other,
        }
    }
}

#[cfg(feature = "std")]
impl From<SystemTime> for PrimitiveDateTime {
    // There is definitely some way to have this conversion be infallible, but
    // it won't be an issue for over 500 years.
    #[inline(always)]
    fn from(system_time: SystemTime) -> Self {
        let duration = match system_time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => Duration::try_from(duration)
                .expect("overflow converting `std::time::Duration` to `time::Duration`"),
            Err(err) => -Duration::try_from(err.duration())
                .expect("overflow converting `std::time::Duration` to `time::Duration`"),
        };

        Self::unix_epoch() + duration
    }
}

#[cfg(feature = "std")]
#[allow(clippy::fallible_impl_from)]
impl From<PrimitiveDateTime> for SystemTime {
    #[inline]
    fn from(datetime: PrimitiveDateTime) -> Self {
        let duration = datetime - PrimitiveDateTime::unix_epoch();

        match duration.sign_abs_std() {
            (Sign::Positive, duration) => Self::UNIX_EPOCH + duration,
            (Sign::Negative, duration) => Self::UNIX_EPOCH - duration,
            (Sign::Zero, _) => Self::UNIX_EPOCH,
        }
    }
}

#[cfg(test)]
#[allow(clippy::result_unwrap_used)]
#[rustfmt::skip::macros(date)]
mod test {
    use super::*;
    use crate::{date, offset, prelude::*, time};

    #[test]
    fn new() {
        assert_eq!(
            PrimitiveDateTime::new(date!(2019-01-01), time!(0:00)),
            date!(2019-01-01).midnight(),
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn now() {
        assert!(PrimitiveDateTime::now().year() >= 2019);
    }

    #[test]
    fn unix_epoch() {
        assert_eq!(
            PrimitiveDateTime::unix_epoch(),
            date!(1970-01-01).midnight()
        );
    }

    #[test]
    fn from_unix_timestamp() {
        assert_eq!(
            PrimitiveDateTime::from_unix_timestamp(0),
            PrimitiveDateTime::unix_epoch()
        );
        assert_eq!(
            PrimitiveDateTime::from_unix_timestamp(1_546_300_800),
            date!(2019-01-01).midnight(),
        );
    }

    #[test]
    fn timestamp() {
        assert_eq!(PrimitiveDateTime::unix_epoch().timestamp(), 0);
        assert_eq!(date!(2019-01-01).midnight().timestamp(), 1_546_300_800);
    }

    #[test]
    fn date() {
        assert_eq!(date!(2019-01-01).midnight().date(), date!(2019-01-01));
    }

    #[test]
    fn time() {
        assert_eq!(date!(2019-01-01).midnight().time(), time!(0:00));
    }

    #[test]
    fn year() {
        assert_eq!(date!(2019-01-01).midnight().year(), 2019);
        assert_eq!(date!(2019-12-31).midnight().year(), 2019);
        assert_eq!(date!(2020-01-01).midnight().year(), 2020);
    }

    #[test]
    fn month() {
        assert_eq!(date!(2019-01-01).midnight().month(), 1);
        assert_eq!(date!(2019-12-31).midnight().month(), 12);
    }

    #[test]
    fn day() {
        assert_eq!(date!(2019-01-01).midnight().day(), 1);
        assert_eq!(date!(2019-12-31).midnight().day(), 31);
    }

    #[test]
    fn month_day() {
        assert_eq!(date!(2019-01-01).midnight().month_day(), (1, 1));
        assert_eq!(date!(2019-12-31).midnight().month_day(), (12, 31));
    }

    #[test]
    fn ordinal() {
        assert_eq!(date!(2019-01-01).midnight().ordinal(), 1);
        assert_eq!(date!(2019-12-31).midnight().ordinal(), 365);
    }

    #[test]
    fn week() {
        assert_eq!(date!(2019-01-01).midnight().week(), 1);
        assert_eq!(date!(2019-10-04).midnight().week(), 40);
        assert_eq!(date!(2020-01-01).midnight().week(), 1);
        assert_eq!(date!(2020-12-31).midnight().week(), 53);
        assert_eq!(date!(2021-01-01).midnight().week(), 53);
    }

    #[test]
    fn sunday_based_week() {
        assert_eq!(date!(2019-01-01).midnight().sunday_based_week(), 0);
        assert_eq!(date!(2020-01-01).midnight().sunday_based_week(), 0);
        assert_eq!(date!(2020-12-31).midnight().sunday_based_week(), 52);
        assert_eq!(date!(2021-01-01).midnight().sunday_based_week(), 0);
    }

    #[test]
    fn monday_based_week() {
        assert_eq!(date!(2019-01-01).midnight().monday_based_week(), 0);
        assert_eq!(date!(2020-01-01).midnight().monday_based_week(), 0);
        assert_eq!(date!(2020-12-31).midnight().monday_based_week(), 52);
        assert_eq!(date!(2021-01-01).midnight().monday_based_week(), 0);
    }

    #[test]
    fn weekday() {
        use Weekday::*;
        assert_eq!(date!(2019-01-01).midnight().weekday(), Tuesday);
        assert_eq!(date!(2019-02-01).midnight().weekday(), Friday);
        assert_eq!(date!(2019-03-01).midnight().weekday(), Friday);
        assert_eq!(date!(2019-04-01).midnight().weekday(), Monday);
        assert_eq!(date!(2019-05-01).midnight().weekday(), Wednesday);
        assert_eq!(date!(2019-06-01).midnight().weekday(), Saturday);
        assert_eq!(date!(2019-07-01).midnight().weekday(), Monday);
        assert_eq!(date!(2019-08-01).midnight().weekday(), Thursday);
        assert_eq!(date!(2019-09-01).midnight().weekday(), Sunday);
        assert_eq!(date!(2019-10-01).midnight().weekday(), Tuesday);
        assert_eq!(date!(2019-11-01).midnight().weekday(), Friday);
        assert_eq!(date!(2019-12-01).midnight().weekday(), Sunday);
    }

    #[test]
    fn hour() {
        assert_eq!(date!(2019-01-01).with_time(time!(0:00)).hour(), 0);
        assert_eq!(date!(2019-01-01).with_time(time!(23:59:59)).hour(), 23);
    }

    #[test]
    fn minute() {
        assert_eq!(date!(2019-01-01).with_time(time!(0:00)).minute(), 0);
        assert_eq!(date!(2019-01-01).with_time(time!(23:59:59)).minute(), 59);
    }

    #[test]
    fn second() {
        assert_eq!(date!(2019-01-01).with_time(time!(0:00)).second(), 0);
        assert_eq!(date!(2019-01-01).with_time(time!(23:59:59)).second(), 59);
    }

    #[test]
    fn millisecond() {
        assert_eq!(date!(2019-01-01).midnight().millisecond(), 0);
        assert_eq!(
            date!(2019-01-01)
                .with_time(time!(23:59:59.999))
                .millisecond(),
            999
        );
    }

    #[test]
    fn microsecond() {
        assert_eq!(date!(2019-01-01).midnight().microsecond(), 0);
        assert_eq!(
            date!(2019-01-01)
                .with_time(time!(23:59:59.999_999))
                .microsecond(),
            999_999
        );
    }

    #[test]
    fn nanosecond() {
        assert_eq!(date!(2019-01-01).midnight().nanosecond(), 0);
        assert_eq!(
            date!(2019-01-01)
                .with_time(time!(23:59:59.999_999_999))
                .nanosecond(),
            999_999_999
        );
    }

    #[test]
    fn using_offset() {
        assert_eq!(
            date!(2019-01-01)
                .midnight()
                .using_offset(offset!(UTC))
                .timestamp(),
            1_546_300_800,
        );
    }

    #[test]
    fn format() {
        assert_eq!(
            date!(2019-01-02).midnight().format("%F %r"),
            "2019-01-02 12:00:00 am"
        );
    }

    #[test]
    fn parse() {
        assert_eq!(
            PrimitiveDateTime::parse("2019-01-02 00:00:00", "%F %T"),
            Ok(date!(2019-01-02).midnight()),
        );
        assert_eq!(
            PrimitiveDateTime::parse("2019-002 23:59:59", "%Y-%j %T"),
            Ok(date!(2019-002).with_time(time!(23:59:59)))
        );
        assert_eq!(
            PrimitiveDateTime::parse("2019-W01-3 12:00:00 pm", "%G-W%V-%u %r"),
            Ok(date!(2019-W01-3).with_time(time!(12:00))),
        );
    }

    #[test]
    fn add_duration() {
        assert_eq!(
            date!(2019-01-01).midnight() + 5.days(),
            date!(2019-01-06).midnight(),
        );
        assert_eq!(
            date!(2019-12-31).midnight() + 1.days(),
            date!(2020-01-01).midnight(),
        );
        assert_eq!(
            date!(2019-12-31).with_time(time!(23:59:59)) + 2.seconds(),
            date!(2020-01-01).with_time(time!(0:00:01)),
        );
        assert_eq!(
            date!(2020-01-01).with_time(time!(0:00:01)) + (-2).seconds(),
            date!(2019-12-31).with_time(time!(23:59:59)),
        );
        assert_eq!(
            date!(1999-12-31).with_time(time!(23:00)) + 1.hours(),
            date!(2000-01-01).midnight(),
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn std_add_duration() {
        assert_eq!(
            SystemTime::from(date!(2019-01-01).midnight()) + 5.days(),
            SystemTime::from(date!(2019-01-06).midnight()),
        );
        assert_eq!(
            SystemTime::from(date!(2019-12-31).midnight()) + 1.days(),
            SystemTime::from(date!(2020-01-01).midnight()),
        );
        assert_eq!(
            SystemTime::from(date!(2019-12-31).with_time(time!(23:59:59))) + 2.seconds(),
            SystemTime::from(date!(2020-01-01).with_time(time!(0:00:01))),
        );
        assert_eq!(
            SystemTime::from(date!(2020-01-01).with_time(time!(0:00:01))) + (-2).seconds(),
            SystemTime::from(date!(2019-12-31).with_time(time!(23:59:59))),
        );
    }

    #[test]
    fn add_std_duration() {
        assert_eq!(
            date!(2019-01-01).midnight() + 5.std_days(),
            date!(2019-01-06).midnight(),
        );
        assert_eq!(
            date!(2019-12-31).midnight() + 1.std_days(),
            date!(2020-01-01).midnight(),
        );
        assert_eq!(
            date!(2019-12-31).with_time(time!(23:59:59)) + 2.std_seconds(),
            date!(2020-01-01).with_time(time!(0:00:01)),
        );
    }

    #[test]
    fn add_assign_duration() {
        let mut ny19 = date!(2019-01-01).midnight();
        ny19 += 5.days();
        assert_eq!(ny19, date!(2019-01-06).midnight());

        let mut nye20 = date!(2019-12-31).midnight();
        nye20 += 1.days();
        assert_eq!(nye20, date!(2020-01-01).midnight());

        let mut nye20t = date!(2019-12-31).with_time(time!(23:59:59));
        nye20t += 2.seconds();
        assert_eq!(nye20t, date!(2020-01-01).with_time(time!(0:00:01)));

        let mut ny20t = date!(2020-01-01).with_time(time!(0:00:01));
        ny20t += (-2).seconds();
        assert_eq!(ny20t, date!(2019-12-31).with_time(time!(23:59:59)));
    }

    #[test]
    fn add_assign_std_duration() {
        let mut ny19 = date!(2019-01-01).midnight();
        ny19 += 5.std_days();
        assert_eq!(ny19, date!(2019-01-06).midnight());

        let mut nye20 = date!(2019-12-31).midnight();
        nye20 += 1.std_days();
        assert_eq!(nye20, date!(2020-01-01).midnight());

        let mut nye20t = date!(2019-12-31).with_time(time!(23:59:59));
        nye20t += 2.std_seconds();
        assert_eq!(nye20t, date!(2020-01-01).with_time(time!(0:00:01)));
    }

    #[test]
    #[cfg(feature = "std")]
    fn std_add_assign_duration() {
        let mut ny19 = SystemTime::from(date!(2019-01-01).midnight());
        ny19 += 5.days();
        assert_eq!(ny19, date!(2019-01-06).midnight());

        let mut nye20 = SystemTime::from(date!(2019-12-31).midnight());
        nye20 += 1.days();
        assert_eq!(nye20, date!(2020-01-01).midnight());

        let mut nye20t = SystemTime::from(date!(2019-12-31).with_time(time!(23:59:59)));
        nye20t += 2.seconds();
        assert_eq!(nye20t, date!(2020-01-01).with_time(time!(0:00:01)));

        let mut ny20t = SystemTime::from(date!(2020-01-01).with_time(time!(0:00:01)));
        ny20t += (-2).seconds();
        assert_eq!(ny20t, date!(2019-12-31).with_time(time!(23:59:59)));
    }

    #[test]
    fn sub_duration() {
        assert_eq!(
            date!(2019-01-06).midnight() - 5.days(),
            date!(2019-01-01).midnight(),
        );
        assert_eq!(
            date!(2020-01-01).midnight() - 1.days(),
            date!(2019-12-31).midnight(),
        );
        assert_eq!(
            date!(2020-01-01).with_time(time!(0:00:01)) - 2.seconds(),
            date!(2019-12-31).with_time(time!(23:59:59)),
        );
        assert_eq!(
            date!(2019-12-31).with_time(time!(23:59:59)) - (-2).seconds(),
            date!(2020-01-01).with_time(time!(0:00:01)),
        );
        assert_eq!(
            date!(1999-12-31).with_time(time!(23:00)) - (-1).hours(),
            date!(2000-01-01).midnight(),
        );
    }

    #[test]
    fn sub_std_duration() {
        assert_eq!(
            date!(2019-01-06).midnight() - 5.std_days(),
            date!(2019-01-01).midnight(),
        );
        assert_eq!(
            date!(2020-01-01).midnight() - 1.std_days(),
            date!(2019-12-31).midnight(),
        );
        assert_eq!(
            date!(2020-01-01).with_time(time!(0:00:01)) - 2.std_seconds(),
            date!(2019-12-31).with_time(time!(23:59:59)),
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn std_sub_duration() {
        assert_eq!(
            SystemTime::from(date!(2019-01-06).midnight()) - 5.days(),
            SystemTime::from(date!(2019-01-01).midnight()),
        );
        assert_eq!(
            SystemTime::from(date!(2020-01-01).midnight()) - 1.days(),
            SystemTime::from(date!(2019-12-31).midnight()),
        );
        assert_eq!(
            SystemTime::from(date!(2020-01-01).with_time(time!(0:00:01))) - 2.seconds(),
            SystemTime::from(date!(2019-12-31).with_time(time!(23:59:59))),
        );
        assert_eq!(
            SystemTime::from(date!(2019-12-31).with_time(time!(23:59:59))) - (-2).seconds(),
            SystemTime::from(date!(2020-01-01).with_time(time!(0:00:01))),
        );
    }

    #[test]
    fn sub_assign_duration() {
        let mut ny19 = date!(2019-01-06).midnight();
        ny19 -= 5.days();
        assert_eq!(ny19, date!(2019-01-01).midnight());

        let mut ny20 = date!(2020-01-01).midnight();
        ny20 -= 1.days();
        assert_eq!(ny20, date!(2019-12-31).midnight());

        let mut ny20t = date!(2020-01-01).with_time(time!(0:00:01));
        ny20t -= 2.seconds();
        assert_eq!(ny20t, date!(2019-12-31).with_time(time!(23:59:59)));

        let mut nye20t = date!(2019-12-31).with_time(time!(23:59:59));
        nye20t -= (-2).seconds();
        assert_eq!(nye20t, date!(2020-01-01).with_time(time!(0:00:01)));
    }

    #[test]
    fn sub_assign_std_duration() {
        let mut ny19 = date!(2019-01-06).midnight();
        ny19 -= 5.std_days();
        assert_eq!(ny19, date!(2019-01-01).midnight());

        let mut ny20 = date!(2020-01-01).midnight();
        ny20 -= 1.std_days();
        assert_eq!(ny20, date!(2019-12-31).midnight());

        let mut ny20t = date!(2020-01-01).with_time(time!(0:00:01));
        ny20t -= 2.std_seconds();
        assert_eq!(ny20t, date!(2019-12-31).with_time(time!(23:59:59)));
    }

    #[test]
    #[cfg(feature = "std")]
    fn std_sub_assign_duration() {
        let mut ny19 = SystemTime::from(date!(2019-01-06).midnight());
        ny19 -= 5.days();
        assert_eq!(ny19, date!(2019-01-01).midnight());

        let mut ny20 = SystemTime::from(date!(2020-01-01).midnight());
        ny20 -= 1.days();
        assert_eq!(ny20, date!(2019-12-31).midnight());

        let mut ny20t = SystemTime::from(date!(2020-01-01).with_time(time!(0:00:01)));
        ny20t -= 2.seconds();
        assert_eq!(ny20t, date!(2019-12-31).with_time(time!(23:59:59)));

        let mut nye20t = SystemTime::from(date!(2019-12-31).with_time(time!(23:59:59)));
        nye20t -= (-2).seconds();
        assert_eq!(nye20t, date!(2020-01-01).with_time(time!(0:00:01)));
    }

    #[test]
    fn sub_datetime() {
        assert_eq!(
            date!(2019-01-02).midnight() - date!(2019-01-01).midnight(),
            1.days()
        );
        assert_eq!(
            date!(2019-01-01).midnight() - date!(2019-01-02).midnight(),
            (-1).days()
        );
        assert_eq!(
            date!(2020-01-01).midnight() - date!(2019-12-31).midnight(),
            1.days()
        );
        assert_eq!(
            date!(2019-12-31).midnight() - date!(2020-01-01).midnight(),
            (-1).days()
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn std_sub_datetime() {
        assert_eq!(
            SystemTime::from(date!(2019-01-02).midnight()) - date!(2019-01-01).midnight(),
            1.days()
        );
        assert_eq!(
            SystemTime::from(date!(2019-01-01).midnight()) - date!(2019-01-02).midnight(),
            (-1).days()
        );
        assert_eq!(
            SystemTime::from(date!(2020-01-01).midnight()) - date!(2019-12-31).midnight(),
            1.days()
        );
        assert_eq!(
            SystemTime::from(date!(2019-12-31).midnight()) - date!(2020-01-01).midnight(),
            (-1).days()
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn sub_std() {
        assert_eq!(
            date!(2019-01-02).midnight() - SystemTime::from(date!(2019-01-01).midnight()),
            1.days()
        );
        assert_eq!(
            date!(2019-01-01).midnight() - SystemTime::from(date!(2019-01-02).midnight()),
            (-1).days()
        );
        assert_eq!(
            date!(2020-01-01).midnight() - SystemTime::from(date!(2019-12-31).midnight()),
            1.days()
        );
        assert_eq!(
            date!(2019-12-31).midnight() - SystemTime::from(date!(2020-01-01).midnight()),
            (-1).days()
        );
    }

    #[test]
    fn ord() {
        use Ordering::*;
        assert_eq!(
            date!(2019-01-01)
                .midnight()
                .partial_cmp(&date!(2019-01-01).midnight()),
            Some(Equal)
        );
        assert_eq!(
            date!(2019-01-01)
                .midnight()
                .partial_cmp(&date!(2020-01-01).midnight()),
            Some(Less)
        );
        assert_eq!(
            date!(2019-01-01)
                .midnight()
                .partial_cmp(&date!(2019-02-01).midnight()),
            Some(Less)
        );
        assert_eq!(
            date!(2019-01-01)
                .midnight()
                .partial_cmp(&date!(2019-01-02).midnight()),
            Some(Less)
        );
        assert_eq!(
            date!(2019-01-01)
                .midnight()
                .partial_cmp(&date!(2019-01-01).with_time(time!(1:00))),
            Some(Less)
        );
        assert_eq!(
            date!(2019-01-01)
                .midnight()
                .partial_cmp(&date!(2019-01-01).with_time(time!(0:01))),
            Some(Less)
        );
        assert_eq!(
            date!(2019-01-01)
                .midnight()
                .partial_cmp(&date!(2019-01-01).with_time(time!(0:00:01))),
            Some(Less)
        );
        assert_eq!(
            date!(2019-01-01)
                .midnight()
                .partial_cmp(&date!(2019-01-01).with_time(time!(0:00:00.000_000_001))),
            Some(Less)
        );
        assert_eq!(
            date!(2020-01-01)
                .midnight()
                .partial_cmp(&date!(2019-01-01).midnight()),
            Some(Greater)
        );
        assert_eq!(
            date!(2019-02-01)
                .midnight()
                .partial_cmp(&date!(2019-01-01).midnight()),
            Some(Greater)
        );
        assert_eq!(
            date!(2019-01-02)
                .midnight()
                .partial_cmp(&date!(2019-01-01).midnight()),
            Some(Greater)
        );
        assert_eq!(
            date!(2019-01-01)
                .with_time(time!(1:00))
                .partial_cmp(&date!(2019-01-01).midnight()),
            Some(Greater)
        );
        assert_eq!(
            date!(2019-01-01)
                .with_time(time!(0:01))
                .partial_cmp(&date!(2019-01-01).midnight()),
            Some(Greater)
        );
        assert_eq!(
            date!(2019-01-01)
                .with_time(time!(0:00:01))
                .partial_cmp(&date!(2019-01-01).midnight()),
            Some(Greater)
        );
        assert_eq!(
            date!(2019-01-01)
                .with_time(time!(0:00:00.000_000_001))
                .partial_cmp(&date!(2019-01-01).midnight()),
            Some(Greater)
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn eq_std() {
        let now_datetime = PrimitiveDateTime::now();
        let now_systemtime = SystemTime::from(now_datetime);
        assert_eq!(now_datetime, now_systemtime);
    }

    #[test]
    #[cfg(feature = "std")]
    fn std_eq() {
        #[cfg(feature = "std")]
        let now_datetime = PrimitiveDateTime::now();
        let now_systemtime = SystemTime::from(now_datetime);
        assert_eq!(now_datetime, now_systemtime);
    }

    #[test]
    #[cfg(feature = "std")]
    fn ord_std() {
        assert_eq!(
            date!(2019-01-01).midnight(),
            SystemTime::from(date!(2019-01-01).midnight())
        );
        assert!(date!(2019-01-01).midnight() < SystemTime::from(date!(2020-01-01).midnight()));
        assert!(date!(2019-01-01).midnight() < SystemTime::from(date!(2019-02-01).midnight()));
        assert!(date!(2019-01-01).midnight() < SystemTime::from(date!(2019-01-02).midnight()));
        assert!(
            date!(2019-01-01).midnight()
                < SystemTime::from(date!(2019-01-01).with_time(time!(1:00:00)))
        );
        assert!(
            date!(2019-01-01).midnight()
                < SystemTime::from(date!(2019-01-01).with_time(time!(0:01:00)))
        );
        assert!(
            date!(2019-01-01).midnight()
                < SystemTime::from(date!(2019-01-01).with_time(time!(0:00:01)))
        );
        assert!(
            date!(2019-01-01).midnight()
                < SystemTime::from(date!(2019-01-01).with_time(time!(0:00:00.001)))
        );
        assert!(date!(2020-01-01).midnight() > SystemTime::from(date!(2019-01-01).midnight()));
        assert!(date!(2019-02-01).midnight() > SystemTime::from(date!(2019-01-01).midnight()));
        assert!(date!(2019-01-02).midnight() > SystemTime::from(date!(2019-01-01).midnight()));
        assert!(
            date!(2019-01-01).with_time(time!(1:00:00))
                > SystemTime::from(date!(2019-01-01).midnight())
        );
        assert!(
            date!(2019-01-01).with_time(time!(0:01:00))
                > SystemTime::from(date!(2019-01-01).midnight())
        );
        assert!(
            date!(2019-01-01).with_time(time!(0:00:01))
                > SystemTime::from(date!(2019-01-01).midnight())
        );
        assert!(
            date!(2019-01-01).with_time(time!(0:00:00.000_000_001))
                > SystemTime::from(date!(2019-01-01).midnight())
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn std_ord() {
        assert_eq!(
            SystemTime::from(date!(2019-01-01).midnight()),
            date!(2019-01-01).midnight()
        );
        assert!(SystemTime::from(date!(2019-01-01).midnight()) < date!(2020-01-01).midnight());
        assert!(SystemTime::from(date!(2019-01-01).midnight()) < date!(2019-02-01).midnight());
        assert!(SystemTime::from(date!(2019-01-01).midnight()) < date!(2019-01-02).midnight());
        assert!(
            SystemTime::from(date!(2019-01-01).midnight())
                < date!(2019-01-01).with_time(time!(1:00:00))
        );
        assert!(
            SystemTime::from(date!(2019-01-01).midnight())
                < date!(2019-01-01).with_time(time!(0:01:00))
        );
        assert!(
            SystemTime::from(date!(2019-01-01).midnight())
                < date!(2019-01-01).with_time(time!(0:00:01))
        );
        assert!(
            SystemTime::from(date!(2019-01-01).midnight())
                < date!(2019-01-01).with_time(time!(0:00:00.000_000_001))
        );
        assert!(SystemTime::from(date!(2020-01-01).midnight()) > date!(2019-01-01).midnight());
        assert!(SystemTime::from(date!(2019-02-01).midnight()) > date!(2019-01-01).midnight());
        assert!(SystemTime::from(date!(2019-01-02).midnight()) > date!(2019-01-01).midnight());
        assert!(
            SystemTime::from(date!(2019-01-01).with_time(time!(1:00:00)))
                > date!(2019-01-01).midnight()
        );
        assert!(
            SystemTime::from(date!(2019-01-01).with_time(time!(0:01:00)))
                > date!(2019-01-01).midnight()
        );
        assert!(
            SystemTime::from(date!(2019-01-01).with_time(time!(0:00:01)))
                > date!(2019-01-01).midnight()
        );
        assert!(
            SystemTime::from(date!(2019-01-01).with_time(time!(0:00:00.001)))
                > date!(2019-01-01).midnight()
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn from_std() {
        assert_eq!(
            PrimitiveDateTime::from(SystemTime::UNIX_EPOCH),
            PrimitiveDateTime::unix_epoch()
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn to_std() {
        assert_eq!(
            SystemTime::from(PrimitiveDateTime::unix_epoch()),
            SystemTime::UNIX_EPOCH
        );
    }
}
