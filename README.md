# frequenz-resampling-rs

This project is the rust resampler for resampling a stream of samples to a given interval.

## Usage

To resample a vector of samples to a given interval, you can use the `Resampler` struct.
The construction of a resampler expects an interval (`TimeDelta`) and a `ResamplingFunction`.
By default, the resampler keeps samples that are not older than `now - interval * 3` after resampling the buffer.
This can be changed by calling `set_max_age`.
Setting it to 0 will remove all samples after resampling.

```rust
use chrono::{DateTime, TimeDelta};
use frequenz_resampling::{Resampler, ResamplingFunction, Sample};

let mut resampler: Resampler<f64, TestSample> =
    Resampler::new(TimeDelta::seconds(5), ResamplingFunction::Average);

resampler.set_max_age(0);
let start = DateTime::from_timestamp(0, 0).unwrap();
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

resampler.extend(&data);

let resampled = resampler.resample(start + step * 10);

let expected = vec![
    TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(3.0)),
    TestSample::new(DateTime::from_timestamp(10, 0).unwrap(), Some(8.0)),
];

assert_eq!(resampled, expected);
```
