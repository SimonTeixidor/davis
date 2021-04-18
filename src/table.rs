use crate::ansi::FormattedString;
use std::fmt;

pub struct TableRow<'a> {
    key: FormattedString<'a>,
    val: FormattedString<'a>,
}

impl<'a> TableRow<'a> {
    pub fn new(key: FormattedString<'a>, val: FormattedString<'a>) -> TableRow<'a> {
        TableRow { key, val }
    }
}

pub struct Table<'a> {
    pub rows: &'a [TableRow<'a>],
    pub disable_formatting: bool,
}

impl<'a> fmt::Display for Table<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let width = formatter.width().unwrap_or(80);
        let key_width = self
            .rows
            .iter()
            .map(|TableRow { key, .. }| key.string.len())
            .max()
            .unwrap_or(20);

        let val_width = width - (key_width + 1).min(width);

        for (key_idx, TableRow { key, val }) in self.rows.iter().enumerate() {
            if self.disable_formatting {
                if key_idx != 0 {
                    writeln!(formatter,)?;
                }
                write!(formatter, "{}={}", key.string, val.string)?;
            } else {
                for (val_idx, line) in textwrap::fill(val.string, val_width).lines().enumerate() {
                    let line = FormattedString::new(line);
                    let line = val.style.iter().fold(line, |l, s| l.style(*s));
                    let line = val.colour.iter().fold(line, |l, c| l.colour(*c));
                    if !(key_idx == 0 && val_idx == 0) {
                        writeln!(formatter,)?;
                    }

                    let empty_string = FormattedString::new("");
                    let key = if val_idx == 0 { key } else { &empty_string };
                    write!(formatter, "{:width$} {}", key, line, width = key_width)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ansi::{Colour, Style};

    #[test]
    fn basic_functionality() {
        let key1 = FormattedString::new("long_key");
        let val1 = FormattedString::new("val");
        let key2 = FormattedString::new("key");
        let val2 = FormattedString::new("val");
        let rows = [TableRow::new(key1, val1), TableRow::new(key2, val2)];
        let table = Table {
            rows: &rows,
            disable_formatting: false,
        };
        let result = format!("{:100}", table);
        let expected = "long_key val\n\
                        key      val";
        assert_eq!(&*result, expected);
    }

    #[test]
    fn basic_functionality_with_formatting() {
        let key1 = FormattedString::new("long_key")
            .colour(Colour::DarkWhite)
            .style(Style::Faint);
        let val1 = FormattedString::new("val")
            .colour(Colour::DarkGreen)
            .style(Style::Bold);
        let key2 = FormattedString::new("key")
            .colour(Colour::DarkWhite)
            .style(Style::Faint);
        let val2 = FormattedString::new("val")
            .colour(Colour::DarkGreen)
            .style(Style::Bold);
        let rows = [TableRow::new(key1, val1), TableRow::new(key2, val2)];
        let table = Table {
            rows: &rows,
            disable_formatting: false,
        };
        let result = format!("{}", table);
        let expected = "\x1B[2;37mlong_key\x1B[0m \x1B[1;32mval\x1B[0m\n\
                        \x1B[2;37mkey     \x1B[0m \x1B[1;32mval\x1B[0m";
        assert_eq!(&*result, expected);
    }

    // Table should always write the "key" in one line, and then fill out the remaining width with
    // the value. If no width remains, the value will have a width of 1 character.
    #[test]
    fn too_narrow_table() {
        let label = FormattedString::new("some");
        let val = FormattedString::new("table");
        let row = [TableRow::new(label, val)];
        let table = Table {
            rows: &row,
            disable_formatting: false,
        };
        let result = format!("{:1}", table);
        let expected = "some t\n     a\n     b\n     l\n     e";
        assert_eq!(&*result, expected);
    }
}
