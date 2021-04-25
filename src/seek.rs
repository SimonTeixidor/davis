use crate::error::Error;
use std::str::FromStr;

pub fn seek(client: &mut mpd::Client, seek_arg: SeekArg) -> Result<(), Error> {
    let status = client.status()?;
    let currentsongid = match client.currentsong()?.and_then(|s| s.place) {
        Some(place) => place.id,
        None => {
            println!("Not plaing.");
            std::process::exit(1);
        }
    };

    match (seek_arg.direction, status.elapsed) {
        (SeekDirection::Absolute, _) => client.seek_id(currentsongid, seek_arg.seconds)?,
        (SeekDirection::Forward, Some(e)) => {
            client.seek_id(currentsongid, e.as_secs() as u32 + seek_arg.seconds)?
        }
        (SeekDirection::Back, Some(e)) => {
            client.seek_id(currentsongid, e.as_secs() as u32 - seek_arg.seconds)?
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
pub struct SeekArg {
    direction: SeekDirection,
    seconds: u32,
}

impl FromStr for SeekArg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 0 {
            return Err(Error::ParseSeekError("Empty string."));
        }

        let (direction, rest) = match s.chars().next() {
            Some('+') => (SeekDirection::Forward, &s[1..]),
            Some('-') => (SeekDirection::Back, &s[1..]),
            _ => (SeekDirection::Absolute, &s[..]),
        };

        let sections = rest
            .split(':')
            .map(u32::from_str)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Error::ParseSeekError("Field is not integer."))?;

        let seconds = sections
            .iter()
            .rev()
            .enumerate()
            .map(|(i, v)| v * 60u32.pow(i as u32))
            .sum();

        Ok(SeekArg { direction, seconds })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hms() {
        assert_eq!(
            SeekArg::from_str("+1:2:3").unwrap(),
            SeekArg {
                direction: SeekDirection::Forward,
                seconds: 1 * 60 * 60 + 2 * 60 + 3
            }
        );

        assert_eq!(
            SeekArg::from_str("0:2:3").unwrap(),
            SeekArg {
                direction: SeekDirection::Absolute,
                seconds: 2 * 60 + 3
            }
        );
    }

    #[test]
    fn test_ms() {
        assert_eq!(
            SeekArg::from_str("+2:3").unwrap(),
            SeekArg {
                direction: SeekDirection::Forward,
                seconds: 2 * 60 + 3
            }
        );
    }

    #[test]
    fn test_s() {
        assert_eq!(
            SeekArg::from_str("3").unwrap(),
            SeekArg {
                direction: SeekDirection::Absolute,
                seconds: 3
            }
        );
    }

    #[test]
    fn test_leading_zeroes() {
        assert_eq!(
            SeekArg::from_str("01:01:01").unwrap(),
            SeekArg {
                direction: SeekDirection::Absolute,
                seconds: 60 * 60 * 1 + 60 * 1 + 1
            }
        );
    }
}
