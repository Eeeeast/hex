use clap::Parser;
use std::fmt;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Hex code
    hex: String,
    /// Advanced
    #[arg(short, long, default_value_t = false)]
    advanced: bool,
}

#[derive(Debug)]
enum Index {
    Data = 0,
    End = 1,
    AddressSegment = 2,
    StartAddress80x86 = 3,
    ExtendedAddress = 4,
    LinearAdrres = 5,
}

struct Word(u8, u8);

struct Package {
    address: u16,
    index: Index,
    data: Vec<Word>,
    checksum: u8,
}

impl Package {
    fn from_str(hex: &String) -> Package {
        let mut data: Package = Package {
            address: match u16::from_str_radix(&hex[3..7], 16) {
                Ok(content) => content,
                Err(error) => panic!("Can't deal with {}, just exit here", error),
            },
            index: match u8::from_str_radix(&hex[7..9], 16) {
                Ok(content) => match content {
                    0 => Index::Data,
                    1 => Index::End,
                    2 => Index::AddressSegment,
                    3 => Index::StartAddress80x86,
                    4 => Index::ExtendedAddress,
                    5 => Index::LinearAdrres,
                    _ => panic!("Can't deal with unexpected index"),
                },
                Err(error) => panic!("Can't deal with {}, just exit here", error),
            },
            data: vec![],
            checksum: 0,
        };
        data.data
            .reserve(match usize::from_str_radix(&hex[1..3], 16) {
                Ok(content) => content,
                Err(error) => panic!("Can't deal with {}, just exit here", error),
            });
        for i in (9..9 + (data.data.capacity() * 2)).step_by(4) {
            data.data.push(Word(
                match u8::from_str_radix(&hex[i + 2..i + 4], 16) {
                    Ok(content) => content,
                    Err(error) => panic!("Can't deal with {}, just exit here", error),
                },
                match u8::from_str_radix(&hex[i..i + 2], 16) {
                    Ok(content) => content,
                    Err(error) => panic!("Can't deal with {}, just exit here", error),
                },
            ));
        }
        data.checksum = match u8::from_str_radix(
            &hex[9 + data.data.len() * 2..9 + data.data.len() * 2 + 2],
            16,
        ) {
            Ok(content) => content,
            Err(error) => panic!("Can't deal with {}, just exit here", error),
        };
        return data;
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:#010b}, {:#010b})", self.0, self.1)
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = writeln!(
            f,
            "size: {}, address: {:#x}, index: {:?},",
            self.data.len(),
            self.address,
            self.index
        );
        if self.data.len() > 0 && result == Ok(()) {
            result = writeln!(f, "data: ");
            for i in &self.data {
                if result == Ok(()) {
                    result = writeln!(f, "    {}, ", i);
                } else {
                    return result;
                }
            }
        }
        if result == Ok(()) {
            result = write!(f, "checksum: {}", self.checksum);
        }
        result
    }
}

fn main() {
    let cli: Cli = Cli::parse();
    let data = Package::from_str(&cli.hex);

    if cli.advanced {
        println!("{:?}", cli);
        println!("{}", data);
        println!();
    }

    if data.data.len() == 0 {
        return;
    }
    let mut iter = data.data.into_iter().enumerate();
    loop {
        match iter.next() {
            Some((i, content)) => {
                print!("{:#x}: ", data.address as usize + i);
                match content.0 {
                    0b0000_1100..=0b0000_1111 => {
                        println!("add");
                    }
                    0b0010_0000..=0b0010_0011 => {
                        println!("and");
                    }
                    0b0111_0000..=0b0111_1111 => {
                        println!("andi");
                    }
                    0b1001_0100 => {
                        if content.1 & 0b1000_1111 == 0b1000_1000 {
                            println!("bclr");
                        } else if content.1 & 0b0000_1111 == 0b0000_0101 {
                            println!("asr");
                        }
                    }
                    0b1001_0101 => {
                        if content.1 & 0b0000_1111 == 0b0000_0101 {
                            println!("asr");
                        } else if content.1 & 0b1111_1111 == 0b1001_1000 {
                            println!("break");
                        }
                    }
                    0b1001_0110 => {
                        println!("adiw");
                    }
                    0b1111_0000..=0b1111_0011 => {
                        if content.1 & 0b0000_0111 == 0b0000_0000 {
                            println!("brcs");
                        } else if content.1 & 0b0000_0111 == 0b0000_0001 {
                            println!("breq");
                        } else {
                            println!("brbs");
                        }
                    }
                    0b1111_0100..=0b1111_0111 => {
                        if content.1 & 0b0000_0111 == 0b0000_0000 {
                            println!("brcc");
                        } else if content.1 & 0b0000_0111 == 0b0000_0100 {
                            println!("brge");
                        } else {
                            println!("brbc");
                        }
                    }
                    0b1111_1000..=0b1111_1001 => {
                        if content.1 & 0b0000_1000 == 0b0000_0000 {
                            println!("bld");
                        }
                    }
                    _ => break,
                };
            }
            None => break,
        }
    }
}
