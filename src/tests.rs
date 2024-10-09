// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::resampler::{Resampler, ResamplingFunction, Sample};
use chrono::{DateTime, TimeDelta, Utc};

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

fn test_resampling(
    resampling_function: ResamplingFunction<f64, TestSample>,
    expected: Vec<TestSample>,
) {
    let mut resampler: Resampler<f64, TestSample> =
        Resampler::new(TimeDelta::seconds(5), resampling_function);

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
    assert_eq!(resampled, expected);
}

#[test]
fn test_resampling_average() {
    test_resampling(
        ResamplingFunction::Average,
        vec![
            TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(3.0)),
            TestSample::new(DateTime::from_timestamp(10, 0).unwrap(), Some(8.0)),
        ],
    );
}

#[test]
fn test_resampling_count() {
    test_resampling(
        ResamplingFunction::Count,
        vec![
            TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(5.0)),
            TestSample::new(DateTime::from_timestamp(10, 0).unwrap(), Some(5.0)),
        ],
    );
}

#[test]
fn test_resampling_sum() {
    test_resampling(
        ResamplingFunction::Sum,
        vec![
            TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(15.0)),
            TestSample::new(DateTime::from_timestamp(10, 0).unwrap(), Some(40.0)),
        ],
    );
}

#[test]
fn test_resampling_min() {
    test_resampling(
        ResamplingFunction::Min,
        vec![
            TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(1.0)),
            TestSample::new(DateTime::from_timestamp(10, 0).unwrap(), Some(6.0)),
        ],
    );
}

#[test]
fn test_resampling_max() {
    test_resampling(
        ResamplingFunction::Max,
        vec![
            TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(5.0)),
            TestSample::new(DateTime::from_timestamp(10, 0).unwrap(), Some(10.0)),
        ],
    );
}

#[test]
fn test_resampling_last() {
    test_resampling(
        ResamplingFunction::Last,
        vec![
            TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(5.0)),
            TestSample::new(DateTime::from_timestamp(10, 0).unwrap(), Some(10.0)),
        ],
    );
}

#[test]
fn test_resampling_custom() {
    test_resampling(
        ResamplingFunction::Custom(&|x: &[&TestSample]| {
            Some(x.iter().map(|s| s.value().unwrap()).sum::<f64>())
        }),
        vec![
            TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(15.0)),
            TestSample::new(DateTime::from_timestamp(10, 0).unwrap(), Some(40.0)),
        ],
    );
}

#[test]
fn test_resampling_with_max_age() {
    let mut resampler: Resampler<f64, TestSample> =
        Resampler::new(TimeDelta::seconds(5), ResamplingFunction::Average);
    resampler.set_max_age(1);
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
        TestSample::new(start + step * 10, Some(11.0)),
        TestSample::new(start + step * 11, Some(12.0)),
        TestSample::new(start + step * 12, Some(13.0)),
        TestSample::new(start + step * 13, Some(14.0)),
        TestSample::new(start + step * 14, Some(15.0)),
    ];

    resampler.extend(&data);

    let expected = vec![
        TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(3.0)),
        TestSample::new(DateTime::from_timestamp(10, 0).unwrap(), Some(5.5)),
        TestSample::new(DateTime::from_timestamp(15, 0).unwrap(), Some(10.5)),
    ];

    let resampled = resampler.resample(start + step * 15);
    assert_eq!(resampled, expected);
}

#[test]
fn test_resampling_with_gap() {
    let mut resampler: Resampler<f64, TestSample> =
        Resampler::new(TimeDelta::seconds(5), ResamplingFunction::Average);
    resampler.set_max_age(0);
    let start = DateTime::from_timestamp(0, 0).unwrap();
    let step = TimeDelta::seconds(1);
    let data = vec![
        TestSample::new(start, Some(1.0)),
        TestSample::new(start + step, Some(2.0)),
        TestSample::new(start + step * 3, Some(4.0)),
        TestSample::new(start + step * 4, Some(5.0)),
        TestSample::new(start + step * 16, Some(6.0)),
        TestSample::new(start + step * 19, Some(10.0)),
    ];

    resampler.extend(&data);

    let expected = vec![
        TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(3.0)),
        TestSample::new(DateTime::from_timestamp(10, 0).unwrap(), None),
        TestSample::new(DateTime::from_timestamp(15, 0).unwrap(), None),
        TestSample::new(DateTime::from_timestamp(20, 0).unwrap(), Some(8.0)),
    ];

    let resampled = resampler.resample(start + step * 20);
    assert_eq!(resampled, expected);
}

#[test]
fn test_resampling_with_slow_data() {
    let mut resampler: Resampler<f64, TestSample> =
        Resampler::new(TimeDelta::seconds(1), ResamplingFunction::Average);
    resampler.set_max_age(1);
    let start = DateTime::from_timestamp(0, 0).unwrap();
    let offset = TimeDelta::milliseconds(500);
    let step = TimeDelta::seconds(2);
    let data = vec![
        TestSample::new(start + step * 1 - offset, Some(3.0)),
        TestSample::new(start + step * 2 - offset, Some(4.0)),
        TestSample::new(start + step * 3 - offset, Some(5.0)),
        TestSample::new(start + step * 4 - offset, Some(6.0)),
    ];

    resampler.extend(&data);

    let expected = vec![
        TestSample::new(DateTime::from_timestamp(1, 0).unwrap(), None),
        TestSample::new(DateTime::from_timestamp(2, 0).unwrap(), Some(3.0)),
        TestSample::new(DateTime::from_timestamp(3, 0).unwrap(), Some(3.0)),
        TestSample::new(DateTime::from_timestamp(4, 0).unwrap(), Some(4.0)),
        TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(4.0)),
        TestSample::new(DateTime::from_timestamp(6, 0).unwrap(), Some(5.0)),
        TestSample::new(DateTime::from_timestamp(7, 0).unwrap(), Some(5.0)),
        TestSample::new(DateTime::from_timestamp(8, 0).unwrap(), Some(6.0)),
    ];

    resampler.set_start(start);
    let resampled = resampler.resample(start + step * 4);
    assert_eq!(resampled, expected);
}

#[test]
fn test_resampling_with_gap_early_end_date() {
    let mut resampler: Resampler<f64, TestSample> =
        Resampler::new(TimeDelta::seconds(5), ResamplingFunction::Average);
    resampler.set_max_age(0);
    let start = DateTime::from_timestamp(0, 0).unwrap();
    let step = TimeDelta::seconds(1);
    let data = vec![
        TestSample::new(start, Some(1.0)),
        TestSample::new(start + step, Some(2.0)),
        TestSample::new(start + step * 3, Some(4.0)),
        TestSample::new(start + step * 4, Some(5.0)),
        TestSample::new(start + step * 16, Some(6.0)),
        TestSample::new(start + step * 19, Some(10.0)),
    ];

    resampler.extend(&data);

    let expected = vec![
        TestSample::new(DateTime::from_timestamp(5, 0).unwrap(), Some(3.0)),
        TestSample::new(DateTime::from_timestamp(10, 0).unwrap(), None),
    ];

    let resampled = resampler.resample(start + step * 10);
    assert_eq!(resampled, expected);

    let expected2 = vec![
        TestSample::new(DateTime::from_timestamp(15, 0).unwrap(), None),
        TestSample::new(DateTime::from_timestamp(20, 0).unwrap(), Some(8.0)),
    ];

    let resampled2 = resampler.resample(start + step * 20);
    assert_eq!(resampled2, expected2);
}

#[test]
fn test_empty_buffer() {
    let mut resampler: Resampler<f64, TestSample> =
        Resampler::new(TimeDelta::seconds(5), ResamplingFunction::Average);
    let start = DateTime::from_timestamp(0, 0).unwrap();
    resampler.set_start(start);

    let resampled = resampler.resample(start + TimeDelta::seconds(10));
    assert_eq!(
        resampled,
        vec![
            TestSample::new(start + TimeDelta::seconds(5), None),
            TestSample::new(start + TimeDelta::seconds(10), None),
        ]
    );
}

#[test]
fn test_epoch_alignment() {
    let resampler =
        Resampler::<f64, TestSample>::new(TimeDelta::seconds(5), ResamplingFunction::Average);
    let test_time = DateTime::from_timestamp(3, 0).unwrap();
    assert_eq!(
        resampler.epoch_align(test_time, None),
        DateTime::from_timestamp(0, 0).unwrap()
    );
    assert_eq!(
        resampler.epoch_align(test_time, Some(DateTime::from_timestamp(1, 0).unwrap())),
        DateTime::from_timestamp(1, 0).unwrap()
    );
}
