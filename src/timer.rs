/// Abstraction over wall-clock time. Inject `SystemClock` in production,
/// `FakeClock` in tests. Required by the spec for Multiplayer prep.
pub trait Clock: Send + Sync {
    /// Returns elapsed milliseconds since an arbitrary epoch.
    fn now_ms(&self) -> u64;
}

/// Production clock backed by `std::time::SystemTime`.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// Test double with a fixed timestamp.
pub struct FakeClock {
    pub ms: u64,
}

impl Clock for FakeClock {
    fn now_ms(&self) -> u64 {
        self.ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_clock_returns_configured_ms() {
        let clock = FakeClock { ms: 42_000 };
        assert_eq!(clock.now_ms(), 42_000);
    }

    #[test]
    fn fake_clock_is_consistent() {
        let clock = FakeClock { ms: 0 };
        assert_eq!(clock.now_ms(), clock.now_ms());
    }

    #[test]
    fn system_clock_is_nonzero() {
        let clock = SystemClock;
        assert!(clock.now_ms() > 0);
    }
}
