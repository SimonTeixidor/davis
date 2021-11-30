use std::env;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Style {
    Bold = 1,
    Faint = 2,
    Underline = 4,
    Default = 0,
}

pub struct FormattedString<'a> {
    pub style: Option<Style>,
    pub string: &'a str,
}

impl<'a> FormattedString<'a> {
    pub fn style(self, style: Style) -> FormattedString<'a> {
        FormattedString {
            style: Some(style),
            ..self
        }
    }

    pub fn new(string: &'a str) -> FormattedString<'a> {
        FormattedString {
            style: None,
            string,
        }
    }
}

impl<'a> fmt::Display for FormattedString<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if is_dumb_terminal() {
            formatter.pad(&*self.string)?;
            return Ok(());
        }

        if let Some(style) = self.style {
            write!(formatter, "\x1B[{}m", style as u8)?;
            formatter.pad(&*self.string)?;
            write!(formatter, "\x1B[{}m", Style::Default as u8)?;
        } else {
            formatter.pad(&*self.string)?;
        }
        Ok(())
    }
}

pub static PLAIN_FORMATTING: AtomicBool = AtomicBool::new(false);
pub fn is_dumb_terminal() -> bool {
    let is_dumb = matches!(env::var("TERM"), Ok(s) if s == "dumb");
    let has_no_color = env::var("NO_COLOR").is_ok();

    is_dumb || has_no_color || PLAIN_FORMATTING.load(Ordering::Relaxed)
}
