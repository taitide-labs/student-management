use nsh_core::{platform::Idiom, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformFamily {
    DesktopWeb,
    Mobile,
    Terminal,
}

#[must_use]
pub fn family() -> PlatformFamily {
    if idiom() == Idiom::Terminal {
        PlatformFamily::Terminal
    } else if is_desktop() {
        PlatformFamily::DesktopWeb
    } else {
        PlatformFamily::Mobile
    }
}

#[must_use]
pub fn is_terminal() -> bool {
    family() == PlatformFamily::Terminal
}

#[must_use]
pub fn is_mobile() -> bool {
    family() == PlatformFamily::Mobile
}

#[must_use]
pub fn is_compact_desktop() -> bool {
    family() == PlatformFamily::DesktopWeb
}
