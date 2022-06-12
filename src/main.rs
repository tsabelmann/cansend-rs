use clap::{arg, Arg, Command};
use pcan_basic;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::slice::Chunks;
use std::str::FromStr;

#[derive(Debug, Clone)]
enum CanId {
    Standard(u32),
    Extended(u32),
}

#[derive(Debug, Clone)]
struct CanData {
    can_id: CanId,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct CanDataError();

impl Display for CanDataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CanDataError()")
    }
}

impl FromStr for CanData {
    type Err = CanDataError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split('#');
        let splits = splits.collect::<Vec<_>>();

        if splits.len() != 2 {
            return Err(CanDataError());
        }

        let can_id = match splits.get(0).copied() {
            None => return Err(CanDataError()),
            Some(can_id) => {
                let value = u32::from_str_radix(can_id, 16);
                match (can_id.len(), value) {
                    (3, Ok(value)) => {
                        if value <= 0x7_FF {
                            CanId::Standard(value)
                        } else {
                            return Err(CanDataError());
                        }
                    }
                    (8, Ok(value)) => {
                        if value <= 0x3F_FF_FF_FF {
                            CanId::Extended(value)
                        } else {
                            return Err(CanDataError());
                        }
                    }
                    _ => return Err(CanDataError()),
                }
            }
        };

        let data = match splits.get(1).copied() {
            None => return Err(CanDataError()),
            Some(data) => {
                let mut can_data = Vec::new();
                for d in data
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks_exact(2)
                    .map(|c| c.iter().collect::<String>())
                {
                    match u8::from_str_radix(&d, 16) {
                        Ok(value) => can_data.push(value),
                        Err(_) => return Err(CanDataError()),
                    }
                }
                can_data
            }
        };

        Ok(CanData { can_id, data })
    }
}

fn main() {
    let matches = Command::new("cansend")
        .version("0.1.0")
        .author("Tim Lucas Sabelmann")
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            #[cfg(feature = "peak")]
            {
                Command::new("add").about("Adds files to myapp").arg(
                    Arg::new("baudrate")
                        .help("The CAN-bus baudrate")
                        // .short('b')
                        // .long("baudrate")
                        .possible_values([
                            "5000", "10000", "20000", "33000", "47000", "50000", "83000", "95000",
                            "100000", "125000", "250000", "500000", "800000", "1000000",
                        ])
                        .required(true)
                        .validator(|s| s.parse::<u32>()),
                )
            },
        )
        .arg(
            Arg::new("can-bus-data")
                .help("The CAN-bus data")
                // .short('d')
                // .long("data")
                .required(true),
        )
        .get_matches();

    let data: CanData = matches.value_of_t("can-bus-data").unwrap();
    println!("can-bus-data={:?}", data);

    match matches.subcommand() {
        Some(("add", sub_matches)) => println!(
            "'myapp add' was used, name is: {:?}",
            sub_matches.value_of("baudrate")
        ),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}
