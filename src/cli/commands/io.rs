use std::io::{self, BufRead};

use clap::{ArgMatches, Values};

use crate::parser::parse_color;
use crate::{PastelError, Result};

use pastel::Color;

pub fn number_arg(matches: &ArgMatches, name: &str) -> Result<f64> {
    let value_str = matches.value_of(name).unwrap();
    value_str
        .parse::<f64>()
        .map_err(|_| PastelError::CouldNotParseNumber(value_str.into()))
}

pub enum ColorArgIterator<'a> {
    FromPositionalArguments(Values<'a>),
    FromStdin,
}

impl<'a> ColorArgIterator<'a> {
    pub fn from_args(args: Option<Values<'a>>) -> Result<Self> {
        match args {
            Some(positionals) => Ok(ColorArgIterator::FromPositionalArguments(positionals)),
            None => {
                use atty::Stream;
                if atty::is(Stream::Stdin) {
                    return Err(PastelError::ColorArgRequired);
                }
                Ok(ColorArgIterator::FromStdin)
            }
        }
    }

    pub fn color_from_stdin() -> Result<Color> {
        let stdin = io::stdin();
        let mut lock = stdin.lock();

        let mut line = String::new();
        let size = lock
            .read_line(&mut line)
            .map_err(|_| PastelError::ColorInvalidUTF8)?;

        if size == 0 {
            return Err(PastelError::CouldNotReadFromStdin);
        }

        let line = line.trim();

        parse_color(&line).ok_or(PastelError::ColorParseError(line.to_string()))
    }

    pub fn from_color_arg(arg: &str) -> Result<Color> {
        match arg {
            "-" => Self::color_from_stdin(),
            color_str => {
                parse_color(color_str).ok_or(PastelError::ColorParseError(color_str.into()))
            }
        }
    }
}

impl<'a> Iterator for ColorArgIterator<'a> {
    type Item = Result<Color>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ColorArgIterator::FromPositionalArguments(ref mut args) => match args.next() {
                Some(color_arg) => Some(Self::from_color_arg(color_arg)),
                None => None,
            },
            ColorArgIterator::FromStdin => match Self::color_from_stdin() {
                Ok(color) => Some(Ok(color)),
                Err(PastelError::CouldNotReadFromStdin) => None,
                err @ Err(_) => Some(err),
            },
        }
    }
}