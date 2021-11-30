use crate::ansi::{is_dumb_terminal, FormattedString};
use std::collections::HashMap;
use std::fmt;

pub struct TableRow<'a> {
    fields: Vec<FormattedString<'a>>,
}

impl<'a> TableRow<'a> {
    pub fn new(fields: Vec<FormattedString<'a>>) -> TableRow<'a> {
        TableRow { fields }
    }
}

pub struct Table<'a> {
    pub rows: &'a [TableRow<'a>],
}

impl<'a> fmt::Display for Table<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let widths = self
            .rows
            .iter()
            .flat_map(|TableRow { fields }| {
                fields.iter().enumerate().map(|(i, f)| (i, f.string.len()))
            })
            .fold(HashMap::<usize, usize>::new(), |mut m, (i, len)| {
                m.entry(i)
                    .and_modify(|cur| *cur = (*cur).max(len))
                    .or_insert(len);
                m
            });

        for (i, TableRow { fields }) in self.rows.iter().enumerate() {
            if i != 0 {
                writeln!(formatter)?;
            }
            for (i, f) in fields.iter().enumerate() {
                if i + 1 == fields.len() {
                    write!(formatter, "{}", f)?;
                } else if is_dumb_terminal() {
                    write!(formatter, "{}:", f)?;
                } else {
                    write!(formatter, "{:width$}", f, width = widths[&i] + 1)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ansi::Style;

    #[test]
    fn basic_functionality() {
        let key1 = FormattedString::new("long_key");
        let val1 = FormattedString::new("val");
        let key2 = FormattedString::new("key");
        let val2 = FormattedString::new("val");
        let rows = [
            TableRow::new(vec![key1, val1]),
            TableRow::new(vec![key2, val2]),
        ];
        let table = Table { rows: &rows };
        let result = format!("{:100}", table);
        let expected = "long_key val\n\
                        key      val";
        assert_eq!(&*result, expected);
    }

    #[test]
    fn basic_functionality_with_formatting() {
        let key1 = FormattedString::new("long_key").style(Style::Faint);
        let val1 = FormattedString::new("val").style(Style::Bold);
        let key2 = FormattedString::new("key").style(Style::Faint);
        let val2 = FormattedString::new("val").style(Style::Bold);
        let rows = [
            TableRow::new(vec![key1, val1]),
            TableRow::new(vec![key2, val2]),
        ];
        let table = Table { rows: &rows };
        let result = format!("{}", table);
        let expected = "\x1B[2mlong_key \x1B[0m\x1B[1mval\x1B[0m\n\
                        \x1B[2mkey      \x1B[0m\x1B[1mval\x1B[0m";
        assert_eq!(&*result, expected);
    }
}
