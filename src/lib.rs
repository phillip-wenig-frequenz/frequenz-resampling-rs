// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

/*!
# frequenz-resampling-rs

This project is the rust resampler for resampling a stream of samples to a
given interval.

## Usage

An instance of the [`Resampler`] can be created with the
[`new`][Resampler::new] method.
Raw data can be added to the resampler either through the
[`push`][Resampler::push] or [`extend`][Resampler::extend] methods, and the
[`resample`][Resampler::resample] method resamples the data that was added to
the buffer.
By default, the resampler keeps samples that are not older than
`now - interval * 3` after resampling the buffer.
This can be changed by calling `set_max_age`.
Setting it to 0 will remove all samples after resampling.

```rust
use chrono::{DateTime, TimeDelta, Utc};
use frequenz_resampling::{Resampler, ResamplingFunction, Sample};

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub(crate) struct TestSample {
    timestamp: DateTime<Utc>,
    value: Option<f64>,
}

impl Sample for TestSample {
    type Value = f64;

    fn new(timestamp: DateTime<Utc>, value: Option<f64>) -> Self {
        Self { timestamp, value }
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn value(&self) -> Option<f64> {
        self.value
    }
}

let start = DateTime::from_timestamp(0, 0).unwrap();
let mut resampler: Resampler<f64, TestSample> =
    Resampler::new(TimeDelta::seconds(5), ResamplingFunction::Average, 0, start);

let step = TimeDelta::seconds(1);
let data = vec![
    TestSample::new(start, Some(1.0)),
    TestSample::new(start + step, Some(2.0)),
    TestSample::new(start + step * 2, Some(3.0)),
    TestSample::new(start + step * 3, Some(4.0)),
    TestSample::new(start + step * 4, Some(5.0)),
    TestSample::new(start + step * 5, Some(6.0)),
    TestSample::new(start + step * 6, Some(7.0)),
    TestSample::new(start + step * 7, Some(8.0)),
    TestSample::new(start + step * 8, Some(9.0)),
    TestSample::new(start + step * 9, Some(10.0)),
];

resampler.extend(data);

let resampled = resampler.resample(start + step * 10);

let expected = vec![
    TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(3.0)),
    TestSample::new(DateTime::from_timestamp(10, 0).unwrap(), Some(8.0)),
];

assert_eq!(resampled, expected);
```
*/

mod resampler;

#[cfg(test)]
mod tests;

pub use resampler::{Resampler, ResamplingFunction, Sample};
