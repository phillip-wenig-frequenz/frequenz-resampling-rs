// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use chrono::{DateTime, TimeDelta, Utc};
use log::warn;
use num_traits::{Float, FromPrimitive};
use std::fmt::Debug;
use std::ops::Div;

use itertools::Itertools;

/// The DataType trait represents the data type of the samples.
pub trait DataType:
    Div<Output = Self> + std::iter::Sum + FromPrimitive + Float + Default + Debug
{
}
impl DataType for f32 {}
impl DataType for f64 {}

/// The Sample trait represents a single sample in a time series.
pub trait Sample: Clone + Debug + Copy + Default {
    type Value;
    fn new(timestamp: DateTime<Utc>, value: Option<Self::Value>) -> Self;
    fn timestamp(&self) -> DateTime<Utc>;
    fn value(&self) -> Option<Self::Value>;
}

/// The ResamplingFunction enum represents the different resampling functions that can be used to resample a channel.
#[derive(Clone)]
pub enum ResamplingFunction<'a, T: DataType, S: Sample<Value = T>> {
    /// Calculates the average of all samples in the time step (ignoring None values)
    Average,
    /// Calculates the sum of all samples in the time step (ignoring None values)
    Sum,
    /// Calculates the maximum value of all samples in the time step (ignoring None values)
    Max,
    /// Calculates the minimum value of all samples in the time step (ignoring None values)
    Min,
    /// Uses the last sample in the time step. If the last sample is None, the resampling function will return None.
    Last,
    /// Counts the number of samples in the time step (ignoring None values)
    Count,
    /// A custom resampling function that takes a closure that takes a slice of samples and returns an optional value.
    Custom(&'a dyn Fn(&[&S]) -> Option<T>),
}

impl<T: DataType, S: Sample<Value = T>> ResamplingFunction<'_, T, S> {
    pub fn apply(&self, samples: &[&S]) -> Option<T> {
        match self {
            Self::Average => Self::Sum
                .apply(samples)
                .and_then(|sum| Self::Count.apply(samples).map(|count| sum.div(count))),
            Self::Sum => samples.iter().filter_map(|s| s.value()).sum1(),
            Self::Max => samples.iter().filter_map(|s| s.value()).max_by(|a, b| {
                a.partial_cmp(b).unwrap_or_else(|| {
                    if a.is_finite() {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Less
                    }
                })
            }),
            Self::Min => samples.iter().filter_map(|s| s.value()).min_by(|a, b| {
                a.partial_cmp(b).unwrap_or_else(|| {
                    if a.is_finite() {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                })
            }),
            Self::Last => samples.last().and_then(|s| s.value()),
            Self::Count => Some(
                T::from_usize(samples.iter().filter_map(|s| s.value()).count())
                    .unwrap_or_else(|| T::zero()),
            ),
            Self::Custom(f) => f(samples),
        }
    }
}

impl<T: DataType, S: Sample<Value = T>> Default for ResamplingFunction<'_, T, S> {
    fn default() -> Self {
        Self::Average
    }
}

impl<T: DataType, S: Sample<Value = T>> Debug for ResamplingFunction<'_, T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Average => write!(f, "Average"),
            Self::Sum => write!(f, "Sum"),
            Self::Max => write!(f, "Max"),
            Self::Min => write!(f, "Min"),
            Self::Last => write!(f, "Last"),
            Self::Count => write!(f, "Count"),
            Self::Custom(_) => write!(f, "Custom"),
        }
    }
}

/// The Resampler struct is used to resample a time series of samples.
/// It stores the samples in a buffer and resamples the samples in the buffer when the resample method is called.
/// A resampler can be configured with a resampling function and a resampling interval.
#[derive(Debug, Default)]
pub struct Resampler<'a, T: DataType, S: Sample<Value = T>> {
    /// The time step between each resampled sample
    interval: TimeDelta,
    /// The resampling functions to use for each channel
    resampling_function: ResamplingFunction<'a, T, S>,
    /// The buffer that stores the samples
    buffer: Vec<S>,
    /// Resample the data in the buffer that is older than max_age. Number of intervals.
    max_age: i32,
    /// The start time of the resampling. If None, the start time is the minimum timestamp of the buffer
    start: Option<DateTime<Utc>>,
    /// The timestamp of the first sample in the buffer
    input_start: Option<DateTime<Utc>>,
    /// The interval between the first and the second sample in the buffer
    input_interval: Option<TimeDelta>,
}

impl<'a, T: DataType, S: Sample<Value = T>> Resampler<'a, T, S> {
    /// Creates a new Resampler with the given resampling interval and resampling function.
    pub fn new(interval: TimeDelta, resampling_function: ResamplingFunction<'a, T, S>) -> Self {
        Self {
            interval,
            resampling_function,
            max_age: 3,
            ..Default::default()
        }
    }

    /// Adds a sample to the buffer.
    pub fn push(&mut self, sample: S) {
        self.buffer.push(sample);
    }

    /// Extends the buffer with the given samples.
    pub fn extend(&mut self, samples: &[S]) {
        self.buffer.extend(samples);
    }

    /// Returns a reference to the buffer.
    pub fn get_buffer(&self) -> &Vec<S> {
        &self.buffer
    }

    /// Sets the start time of the resampling.
    pub fn set_start(&mut self, start: DateTime<Utc>) {
        self.start = Some(start);
    }

    /// Sets the maximum age of the samples in the buffer.
    pub fn set_max_age(&mut self, max_age: i32) {
        self.max_age = max_age;
    }

    /// Resamples the samples in the buffer and returns the resampled samples until the given end time.
    pub fn resample(&mut self, end: DateTime<Utc>) -> Vec<S> {
        if let Some(mut start) = self.find_start_time() {
            if start >= end {
                warn!("start time is greater or equal to end time");
                return vec![];
            }
            let mut res = vec![];
            let mut interval_buffer = vec![];
            let mut buffer_iter = self.buffer.iter();
            let mut next_sample: Option<&S> = buffer_iter.next();
            self.input_start = next_sample.map(|s| s.timestamp());

            // loop over the intervals
            while start < end {
                // loop over the samples in the buffer
                while next_sample
                    .map(|s| s.timestamp() < start + self.interval)
                    .unwrap_or(false)
                {
                    // next sample is not newer than the current interval
                    if let Some(s) = next_sample {
                        if s.timestamp() >= start && s.timestamp() < start + self.interval {
                            // sample is within the current interval, add it to the interval_buffer
                            interval_buffer.push(s);
                        } else if s.timestamp() < start {
                            // sample is in the past of the current interval, ignore it
                            warn!("sample is out of order");
                        }
                        // get the next sample
                        next_sample = buffer_iter.next();
                        // update the input_start and input_interval to adapt the resampling interval to the input data
                        if let Some(input_start) = self.input_start {
                            if self.input_interval.is_none() {
                                self.input_interval =
                                    Some((s.timestamp() - input_start).max(self.interval));
                            }
                        }
                    }
                }

                // resample the interval_buffer
                res.push(Sample::new(
                    start + self.interval,
                    self.resampling_function.apply(interval_buffer.as_slice()),
                ));

                // Remove samples from interval_buffer that are older than max_age
                let input_interval = self.input_interval.unwrap_or(self.interval);
                let drain_end_date = start + self.interval - input_interval * self.max_age;
                interval_buffer.retain(|s| s.timestamp() >= drain_end_date);

                // Go to the next interval
                start += self.interval;
            }

            // Remove samples from buffer that are older than max_age
            let interval = self.input_interval.unwrap_or(self.interval);
            let drain_end_date = end - interval * self.max_age;
            self.buffer.retain(|s| s.timestamp() >= drain_end_date);
            self.set_start(drain_end_date);

            res
        } else {
            warn!("no start time was found");
            vec![]
        }
    }

    /// Resamples the samples in the buffer and returns the resampled samples until now.
    pub fn resample_now(&mut self) -> Vec<S> {
        self.resample(Utc::now())
    }

    /// Finds the start time of the resampling.
    fn find_start_time(&self) -> Option<DateTime<Utc>> {
        self.start.or_else(|| {
            self.buffer
                .iter()
                .map(|s| s.timestamp())
                .min()
                .map(|first_timestamp| self.epoch_align(first_timestamp, None))
        })
    }

    /// Aligns a timestamp to the epoch of the resampling interval.
    pub(crate) fn epoch_align(
        &self,
        timestamp: DateTime<Utc>,
        alignment_timestamp: Option<DateTime<Utc>>,
    ) -> DateTime<Utc> {
        let alignment_timestamp =
            alignment_timestamp.unwrap_or_else(|| DateTime::from_timestamp_millis(0).unwrap());
        DateTime::from_timestamp_millis(
            (timestamp.timestamp_millis() / self.interval.num_milliseconds())
                * self.interval.num_milliseconds()
                + alignment_timestamp.timestamp_millis(),
        )
        .unwrap_or(timestamp)
    }
}
