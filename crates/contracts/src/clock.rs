pub trait Clock {
    fn now_rfc3339(&self) -> String;
    fn today_iso(&self) -> String;
}
