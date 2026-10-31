//! Rate limiting for diagnostics that can fire every frame.

/// Reports a recurring condition at most once per `interval` seconds without
/// losing count of what happened in between.
///
/// Use it as a system `Local` for warnings such as "entity cap reached": the
/// first occurrence is reported immediately, later ones are folded into the
/// next report, so a problem is never silent and never floods the log.
#[derive(Debug, Clone)]
pub struct LogThrottle {
    interval: f64,
    last_report: Option<f64>,
    pending: u64,
}

impl LogThrottle {
    /// A throttle that reports at most once every `interval` seconds.
    pub fn new(interval: f64) -> Self {
        Self {
            interval,
            last_report: None,
            pending: 0,
        }
    }

    /// Records `count` occurrences at time `now` (seconds, monotonic).
    ///
    /// Returns `Some(total)` when a report is due, where `total` is every
    /// occurrence since the previous report including these; otherwise the
    /// occurrences are kept for the next report. `count == 0` records nothing.
    pub fn record(&mut self, now: f64, count: u64) -> Option<u64> {
        if count == 0 {
            return None;
        }
        self.pending += count;
        let due = match self.last_report {
            None => true,
            Some(last) => now - last >= self.interval,
        };
        if due {
            self.last_report = Some(now);
            Some(std::mem::take(&mut self.pending))
        } else {
            None
        }
    }
}

impl Default for LogThrottle {
    /// One report per second.
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_occurrence_reports_immediately() {
        let mut throttle = LogThrottle::new(1.0);
        assert_eq!(throttle.record(10.0, 1), Some(1));
    }

    #[test]
    fn occurrences_inside_interval_are_folded_into_next_report() {
        let mut throttle = LogThrottle::new(1.0);
        assert_eq!(throttle.record(0.0, 1), Some(1));
        assert_eq!(throttle.record(0.4, 2), None);
        assert_eq!(throttle.record(0.9, 3), None);
        assert_eq!(
            throttle.record(1.0, 1),
            Some(6),
            "interval boundary is inclusive"
        );
        assert_eq!(throttle.record(1.5, 1), None);
        assert_eq!(throttle.record(2.5, 0), None, "zero records nothing");
        assert_eq!(throttle.record(2.6, 1), Some(2));
    }

    #[test]
    fn default_interval_is_one_second() {
        let mut throttle = LogThrottle::default();
        assert_eq!(throttle.record(0.0, 1), Some(1));
        assert_eq!(throttle.record(0.99, 1), None);
        assert_eq!(throttle.record(1.0, 1), Some(2));
    }
}
