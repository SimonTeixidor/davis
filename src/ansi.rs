use std::env;
use std::fmt;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Colour {
    Black = 30,
    DarkRed = 31,
    DarkGreen = 32,
    DarkYellow = 33,
    DarkBlue = 34,
    DarkMagenta = 35,
    DarkCyan = 36,
    DarkWhite = 37,
    BrightBlack = 90,
    BrightRed = 91,
    BrightGreen = 92,
    BrightYellow = 93,
    BrightBlue = 94,
    BrightMagenta = 95,
    BrightCyan = 96,
    White = 97,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Style {
    Bold = 1,
    Faint = 2,
    Underline = 4,
    Default = 0,
}

pub struct FormattedString<'a> {
    pub colour: Option<Colour>,
    pub style: Option<Style>,
    pub string: &'a str,
}

impl<'a> FormattedString<'a> {
    pub fn colour(self, colour: Colour) -> FormattedString<'a> {
        FormattedString {
            colour: Some(colour),
            ..self
        }
    }

    pub fn style(self, style: Style) -> FormattedString<'a> {
        FormattedString {
            style: Some(style),
            ..self
        }
    }

    pub fn new(string: &'a str) -> FormattedString<'a> {
        FormattedString {
            colour: None,
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

        match (self.style, self.colour) {
            (Some(colour), Some(style)) => {
                write!(formatter, "\x1B[{};{}m", colour as u8, style as u8)?;
                formatter.pad(&*self.string)?;
                write!(formatter, "\x1B[{}m", Style::Default as u8)?;
            }
            (Some(colour), None) => {
                write!(formatter, "\x1B[{}m", colour as u8)?;
                formatter.pad(&*self.string)?;
                write!(formatter, "\x1B[{}m", Style::Default as u8)?;
            }
            (None, Some(style)) => {
                write!(formatter, "\x1B[{}m", style as u8)?;
                formatter.pad(&*self.string)?;
                write!(formatter, "\x1B[{}m", Style::Default as u8)?;
            }
            (None, None) => {
                formatter.pad(&*self.string)?;
            }
        };
        Ok(())
    }
}

pub fn is_dumb_terminal() -> bool {
    let is_dumb = match env::var("TERM") {
        Ok(s) if s == "dumb" => true,
        _ => false,
    };
    let has_no_color = env::var("NO_COLOR").is_ok();

    is_dumb || has_no_color
}
