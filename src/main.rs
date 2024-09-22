use clap::Parser;
use std::{fmt, ops::RangeToInclusive};

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
                    0b0000_0001 => {
                        println!("movw");
                    }
                    0b0000_0010 => {
                        println!("muls");
                    }
                    0b0000_0011 => match content.1 & 0b1000_1000 {
                        0b0000_0000 => println!("mulsu"),
                        0b0000_1000 => println!("fmul"),
                        0b1000_0000 => println!("fmuls"),
                        0b1000_1000 => println!("fmulsu"),
                    },
                    0b0000_1100..=0b0000_1111 => {
                        println!("add");
                        println!("lsl");
                    }
                    0b0000_0100..=0b0000_0111 => {
                        println!("cpc");
                    }
                    0b0001_0000..=0b0001_0011 => {
                        println!("cpse");
                    }
                    0b0001_0100..=0b0001_0111 => {
                        println!("cp");
                    }
                    0b0010_0000..=0b0010_0011 => {
                        println!("and");
                    }
                    0b0010_0100..=0b0010_0111 => {
                        println!("eor");
                        println!("clr");
                    }
                    0b0010_1100..=0b0010_1111 => {
                        println!("mov");
                    }
                    0b0011_0000..=0b0011_1111 => {
                        println!("cpi");
                    }
                    0b0111_0000..=0b0111_1111 => {
                        println!("andi");
                    }
                    0b1000_0000..=0b1000_0001 => {
                        if content.1 & 0b0000_1111 == 0b0000_1000 {
                            println!("ld");
                        } else if content.1 & 0b0000_1000 == 0b0000_1000 {
                            println!("ld");
                        }
                    }
                    0b1001_0000..=0b1001_0001 => match content.1 & 0b0000_1111 {
                        0b0100 => println!("lpm"),
                        0b0101 => println!("lpm"),
                        0b1001 => println!("ld"),
                        0b1010 => println!("ld"),
                        0b1100 => println!("ld"),
                        0b1101 => println!("ld"),
                        0b1110 => println!("ld"),
                        0b0000 => {
                            println!("lds");
                            iter.next();
                        }
                        _ => panic!(""),
                    },
                    0b1001_0100 => {
                        if content.1 == 0b0000_1001 {
                            println!("ijmp");
                        } else if content.1 == 0b0001_1001 {
                            println!("eijmp");
                        } else if content.1 == 0b1000_1000 {
                            println!("clc");
                        } else if content.1 == 0b1001_1000 {
                            println!("clz");
                        } else if content.1 == 0b1101_1000 {
                            println!("clh");
                        } else if content.1 == 0b1111_1000 {
                            println!("cli");
                        } else if content.1 == 0b1010_1000 {
                            println!("cln");
                        } else if content.1 == 0b1011_1000 {
                            println!("clv");
                        } else if content.1 == 0b1100_1000 {
                            println!("cls");
                        } else if content.1 == 0b1110_1000 {
                            println!("clt");
                        } else if content.1 & 0b1000_1111 == 0b1000_1000 {
                            println!("bclr");
                        } else if content.1 & 0b1000_1111 == 0b0000_1000 {
                            println!("bset");
                        } else if content.1 & 0b0000_1111 == 0b0000_0101 {
                            println!("asr");
                        } else if content.1 & 0b0000_1110 == 0b0000_1100 {
                            println!("jmp");
                            iter.next();
                        } else if content.1 & 0b0000_1110 == 0b0000_1110 {
                            println!("call");
                            iter.next();
                        } else if content.1 & 0b0000_1111 == 0b0000_1111 {
                            println!("com");
                        } else if content.1 & 0b0000_1111 == 0b0000_1010 {
                            println!("dec");
                        } else if content.1 & 0b0000_1111 == 0b0000_0011 {
                            println!("inc");
                        } else if content.1 & 0b0000_1111 == 0b0000_0110 {
                            println!("lsr");
                        }
                    }
                    0b1001_0101 => {
                        if content.1 == 0b0000_1001 {
                            println!("icall");
                        } else if content.1 == 0b0001_1001 {
                            println!("eicall");
                        } else if content.1 == 0b1100_1000 {
                            println!("lpm");
                        } else if content.1 & 0b0000_1111 == 0b0000_0101 {
                            println!("asr");
                        } else if content.1 & 0b1111_1111 == 0b1001_1000 {
                            println!("break");
                        } else if content.1 & 0b0000_1110 == 0b0000_1100 {
                            println!("jmp");
                            iter.next();
                        } else if content.1 & 0b0000_1110 == 0b0000_1110 {
                            println!("call");
                            iter.next();
                        } else if content.1 & 0b0000_1111 == 0b0000_1111 {
                            println!("com");
                        } else if content.1 & 0b0000_1111 == 0b0000_1010 {
                            println!("dec");
                        } else if content.1 & 0b0000_1111 == 0b0000_0011 {
                            println!("inc");
                        }
                    }
                    0b1001_0110 => {
                        println!("adiw");
                    }
                    0b1001_1000 => {
                        println!("cbi");
                    }
                    0b1001_1100..=0b1001_1111 => {
                        println!("mul");
                    }
                    0b1011_0000..=0b1011_0111 => {
                        println!("in");
                    }
                    0b1110_0000..=0b1110_1111 => {
                        println!("ldi");
                    }
                    0b1111_0000..=0b1111_0011 => {
                        if content.1 & 0b0000_0111 == 0b0000_0000 {
                            println!("brcs");
                        } else if content.1 & 0b0000_0111 == 0b0000_0001 {
                            println!("breq");
                        } else if content.1 & 0b0000_0111 == 0b0000_0010 {
                            println!("brmi");
                        } else if content.1 & 0b0000_0111 == 0b0000_0011 {
                            println!("brvs");
                        } else if content.1 & 0b0000_0111 == 0b0000_0100 {
                            println!("brlt");
                        } else if content.1 & 0b0000_0111 == 0b0000_0101 {
                            println!("brhs");
                        } else if content.1 & 0b0000_0111 == 0b0000_0110 {
                            println!("brts");
                        } else if content.1 & 0b0000_0111 == 0b0000_0111 {
                            println!("brie");
                        } else {
                            println!("brbs");
                        }
                    }
                    0b1111_0100..=0b1111_0111 => {
                        if content.1 & 0b0000_0111 == 0b0000_0000 {
                            println!("brcc");
                        } else if content.1 & 0b0000_0111 == 0b0000_0001 {
                            println!("brne");
                        } else if content.1 & 0b0000_0111 == 0b0000_0010 {
                            println!("brpl");
                        } else if content.1 & 0b0000_0111 == 0b0000_0011 {
                            println!("brvc");
                        } else if content.1 & 0b0000_0111 == 0b0000_0100 {
                            println!("brge");
                        } else if content.1 & 0b0000_0111 == 0b0000_0101 {
                            println!("brhc");
                        } else if content.1 & 0b0000_0111 == 0b0000_0110 {
                            println!("brtc");
                        } else if content.1 & 0b0000_0111 == 0b0000_0111 {
                            println!("brid");
                        } else {
                            println!("brbc");
                        }
                    }
                    0b1111_1010..=0b1111_1011 => {
                        println!("bst");
                    }
                    0b1111_1000..=0b1111_1001 => {
                        if content.1 & 0b0000_1000 == 0b0000_0000 {
                            println!("bld");
                        }
                    }
                    _ => {
                        if content.0 & 0b1101_1101 == 0b1000_0000 {
                            if content.1 & 0b0000_1000 == 0b0000_1000 {
                                println!("ld");
                            }
                        } else {
                            break;
                        }
                    }
                };
            }
            None => break,
        }
    }
}
