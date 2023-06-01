use alloc::{borrow::Cow, vec::Vec};

use macroquad::prelude::*;

#[doc(hidden)]
#[macro_export]
macro_rules! log {
    ($id : ident in $logger: expr, $format: expr $(, $($args:tt)*)? ) => {
        $crate::logging::_log_args($logger, Some(stringify!($id)), ::core::format_args!( $format, $($( $args )*)? ), 5.0);
    };
    ($logger: expr, $format: expr $(, $($args:tt)*)? ) => {
        $crate::logging::_log_args($logger, None, ::core::format_args!( $format, $($( $args )*)? ), 5.0);
    };
}
pub use log;

#[doc(hidden)]
pub fn _log_args(
    log: &mut Log,
    id: Option<&'static str>,
    args: ::core::fmt::Arguments,
    timeout: f64,
) {
    log.add_entry(
        id,
        if let Some(that) = args.as_str() {
            ::std::borrow::Cow::Borrowed(that)
        } else {
            ::std::borrow::Cow::Owned(args.to_string())
        },
        timeout,
    );
}

pub struct Log {
    entries: Vec<LogEntry>,
    params: TextParams,
    fade_duration: f64,
}
impl Log {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            params: TextParams::default(),
            fade_duration: 0.3,
        }
    }
    pub fn update(&mut self, dt: f64) {
        self.entries.retain_mut(|entry| {
            entry.timeout -= dt;
            entry.timeout > 0.0
        });
    }
    pub fn add_entry(&mut self, id: Option<&'static str>, string: Cow<'static, str>, timeout: f64) {
        let entry = LogEntry {
            string,
            timeout,
            id,
        };
        let index = if let Some(id) = id {
            self.entries.iter().position(|entry| entry.id == Some(id))
        } else {
            None
        };
        if let Some(index) = index {
            self.entries[index] = entry;
        } else {
            self.entries.push(entry);
        }
    }
    pub fn display(&self) {
        let mut pos = [0.0; 2];
        for entry in &self.entries {
            let mut params = self.params;
            let size = measure_text(
                &entry.string,
                Some(params.font),
                params.font_size,
                params.font_scale,
            );
            pos[1] += size.offset_y * 1.2;

            let fade_mul = entry.timeout.min(self.fade_duration) / self.fade_duration;
            params.color.a *= fade_mul as f32;
            draw_text_ex(&entry.string, pos[0] + 10.0, pos[1], params);

            pos[0] = 0.0;
        }
    }
}

struct LogEntry {
    string: Cow<'static, str>,
    timeout: f64,
    id: Option<&'static str>,
}