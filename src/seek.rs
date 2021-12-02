use crate::error::Error;
use std::convert::TryFrom;
use std::str::FromStr;

pub fn seek(client: &mut mpdrs::Client, seek_arg: Arg) -> Result<(), Error> {
    let status = client.status()?;
    let currentsongid = if let Some(place) = client.currentsong()?.and_then(|s| s.place) {
        place.id
    } else {
        println!("Not plaing.");
        std::process::exit(1);
    };

    match (seek_arg.direction, status.elapsed) {
        (SeekDirection::Absolute, _) => client.seek_id(currentsongid, seek_arg.seconds)?,
        (SeekDirection::Forward, Some(e)) => {
            client.seek_id(
                currentsongid,
                u32::try_from(e.as_secs()).expect("Time does not fit in u32") + seek_arg.seconds,
            )?;
        }
        (SeekDirection::Back, Some(e)) => {
            client.seek_id(
                currentsongid,
                u32::try_from(e.as_secs()).expect("Time does not fit in u32") - seek_arg.seconds,
            )?;
        }
        _ => (),
    }

    Ok(())
}

#[derive(PartialEq, Eq, Debug)]
enum SeekDirection {
    Forward,
    Back,
    Absolute,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Arg {
    direction: SeekDirection,
    seconds: u32,
}

impl FromStr for Arg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Error::ParseSeek("Empty string."));
        }

        let (direction, rest) = match s.chars().next() {
            Some('+') => (SeekDirection::Forward, &s[1..]),
            Some('-') => (SeekDirection::Back, &s[1..]),
            _ => (SeekDirection::Absolute, s),
        };

        let sections = rest
            .split(':')
            .map(u32::from_str)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Error::ParseSeek("Field is not integer."))?;

        let seconds = sections
            .iter()
            .rev()
            .enumerate()
            .map(|(i, v)| v * 60_u32.pow(u32::try_from(i).expect("Counter does not fit in u32")))
            .sum();

        Ok(Arg { direction, seconds })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hms() {
        assert_eq!(
            Arg::from_str("+1:2:3").unwrap(),
            Arg {
                direction: SeekDirection::Forward,
                seconds: 1 * 60 * 60 + 2 * 60 + 3
            }
        );

        assert_eq!(
            Arg::from_str("0:2:3").unwrap(),
            Arg {
                direction: SeekDirection::Absolute,
                seconds: 2 * 60 + 3
            }
        );
    }

    #[test]
    fn test_ms() {
        assert_eq!(
            Arg::from_str("+2:3").unwrap(),
            Arg {
                direction: SeekDirection::Forward,
                seconds: 2 * 60 + 3
            }
        );
    }

    #[test]
    fn test_s() {
        assert_eq!(
            Arg::from_str("3").unwrap(),
            Arg {
                direction: SeekDirection::Absolute,
                seconds: 3
            }
        );
    }

    #[test]
    fn test_leading_zeroes() {
        assert_eq!(
            Arg::from_str("01:01:01").unwrap(),
            Arg {
                direction: SeekDirection::Absolute,
                seconds: 60 * 60 * 1 + 60 * 1 + 1
            }
        );
    }
}
