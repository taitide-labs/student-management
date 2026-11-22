use parking_lot::Mutex;
use student_management_contracts::clock::Clock;

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    #[expect(
        clippy::disallowed_methods,
        reason = "infra SystemClock 封装系统唯一真实时间来源"
    )]
    fn now_rfc3339(&self) -> String {
        jiff::Timestamp::now().to_string()
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "infra SystemClock 封装系统唯一真实日期来源"
    )]
    fn today_iso(&self) -> String {
        jiff::Timestamp::now()
            .to_string()
            .chars()
            .take(10)
            .collect()
    }
}

#[derive(Debug)]
pub struct FixedClock {
    now: Mutex<String>,
    today: Mutex<String>,
}

impl FixedClock {
    #[must_use]
    pub fn new(now_rfc3339: impl Into<String>, today_iso: impl Into<String>) -> Self {
        Self {
            now: Mutex::new(now_rfc3339.into()),
            today: Mutex::new(today_iso.into()),
        }
    }

    pub fn set_now(&self, now_rfc3339: impl Into<String>, today_iso: impl Into<String>) {
        *self.now.lock() = now_rfc3339.into();
        *self.today.lock() = today_iso.into();
    }
}

impl Clock for FixedClock {
    fn now_rfc3339(&self) -> String {
        self.now.lock().clone()
    }

    fn today_iso(&self) -> String {
        self.today.lock().clone()
    }
}
