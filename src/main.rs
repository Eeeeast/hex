use clap::Parser;
use std::ops::{Shl, Shr};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Hex code
    hex: String,
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

#[derive(Debug)]
struct Package {
    size: u8,
    address: u16,
    index: Index,
    data: Vec<(u8, u8)>,
    checksum: u8,
}

fn main() {
    let cli: Cli = Cli::parse();

    let mut data = Package {
        size: 0,
        address: 0,
        index: Index::Data,
        data: vec![],
        checksum: 0,
    };

    data.size = match u8::from_str_radix(&cli.hex[1..3], 16) {
        Ok(content) => content,
        Err(error) => panic!("Can't deal with {}, just exit here", error),
    };
    data.address = match u16::from_str_radix(&cli.hex[3..7], 16) {
        Ok(content) => content,
        Err(error) => panic!("Can't deal with {}, just exit here", error),
    };
    data.index = match u8::from_str_radix(&cli.hex[7..9], 16) {
        Ok(content) => match content {
            0 => Index::Data,
            1 => Index::End,
            2 => Index::AddressSegment,
            3 => Index::StartAddress80x86,
            4 => Index::ExtendedAddress,
            5 => Index::LinearAdrres,
            _ => panic!("Cant deal with unexpected index"),
        },
        Err(error) => panic!("Can't deal with {}, just exit here", error),
    };
    data.data.reserve(data.size as usize * 2);
    for i in (9..9 + (data.size as usize * 2)).step_by(4) {
        data.data.push((
            match u8::from_str_radix(&cli.hex[i + 2..i + 4], 16) {
                Ok(content) => content,
                Err(error) => panic!("Can't deal with {}, just exit here", error),
            },
            match u8::from_str_radix(&cli.hex[i..i + 2], 16) {
                Ok(content) => content,
                Err(error) => panic!("Can't deal with {}, just exit here", error),
            },
        ));
    }
    data.checksum = match u8::from_str_radix(
        &cli.hex[9 + (data.size as usize) * 2..9 + (data.size as usize) * 2 + 2],
        16,
    ) {
        Ok(content) => content,
        Err(error) => panic!("Can't deal with {}, just exit here", error),
    };

    println!("{:?}", cli);

    println!(
        "size: {}, address: {:#x}, index: {:?}",
        data.size, data.address, data.index
    );

    for i in &data.data {
        println!("{:#010b} {:#010b}", i.0, i.1);
    }

    println!();

    let mut iter = data.data.iter().enumerate();
    loop {
        match iter.next() {
            Some((i, content)) => match content.0 {
                0b00000000 => {
                    if content.1 == 0b00000000 {
                        println!("nop");
                    } else {
                        break;
                    }
                }
                0b00100100..=0b00100111 => {
                    let r: u8 = (content.0 & 0b00000010).shl(3) + (content.1 & 0b00001111);
                    let d: u8 = (content.0 & 0b00000001).shl(4) + (content.1 & 0b11110000).shr(4);
                    if d == r {
                        println!("clr r{}", r);
                    } else {
                        println!("eor r{}, r{}", d, r);
                    }
                }
                0b01000000..=0b01001111 => {
                    let k: u8 = (content.0 & 0b00001111).shl(4) + (content.1 & 0b00001111);
                    let d: u8 = 16u8 + (content.1 & 0b11110000).shr(4);
                    println!("sbci r{}, {:#x}", d, k);
                }
                0b01010000..=0b01011111 => {
                    let k: u8 = (content.0 & 0b00001111).shl(4) + (content.1 & 0b00001111);
                    let d: u8 = 16u8 + (content.1 & 0b11110000).shr(4);
                    println!("subi r{}, {:#x}", d, k);
                }
                0b10010100..=0b10010101 => {
                    if content.1 & 0b00001110 == 0b00001100 {
                        let t: u32 = ((content.0 & 0b00000001) as u32).shl(22)
                            + ((content.1 & 0b11110001) as u32).shl(17)
                            + match iter.next() {
                                Some((_, content)) => {
                                    ((content.0 as u32).shl(9) + (content.1 as u32).shl(1)) as u32
                                }
                                None => panic!("Can't deal with jmp decode"),
                            };

                        println!("jmp {:#x} ; {:#x}", t, t);
                    } else if content.1 & 0b00001110 == 0b00001110 {
                        let t: u32 = ((content.0 & 0b00000001) as u32).shl(22)
                            + ((content.1 & 0b11110001) as u32).shl(17)
                            + match iter.next() {
                                Some((_, content)) => {
                                    ((content.0 as u32).shl(9) + (content.1 as u32).shl(1)) as u32
                                }
                                None => panic!("Can't deal with call decode"),
                            };
                        println!("call {:#x}; {:#x}", t, t);
                    } else if content.0 == 0b10010100 && content.1 == 0b11111000 {
                        println!("cli");
                    } 
                    else {
                        break;
                    }
                }
                0b10011000 => {
                    let a: u8 = (content.1 & 0b11111000).shr(3);
                    let b: u8 = content.1 & 0b00000111;
                    println!("cbi {:#x}, {}", a, b);
                }
                0b10011010 => {
                    let a: u8 = (content.1 & 0b11111000).shr(3);
                    let b: u8 = content.1 & 0b00000111;
                    println!("sbi {:#x}, {}", a, b);
                }
                0b10111000..=0b10111111 => {
                    let a: u8 = (content.0 & 0b00000110).shl(3) + (content.1 & 0b00001111);
                    let r: u8 = (content.0 & 0b00000001).shl(4) + (content.1 & 0b11110000).shr(4);
                    println!("out {:#x}, r{}", a, r);
                }
                0b11000000..=0b11001111 => {
                    let k: i16 = (((content.0 & 0b00001111) as i16).shl(12) | (content.1 as i16).shl(4)) / 8i16;
                     if k < 0 {
                            print!("rjmp .-{:#x}", k.abs());
                        } else {
                            print!("rjmp .+{:#x}", k)
                        }
                        println!(
                            " ; {:#x}",
                            (data.address as usize + i * 2) as i32 + k as i32 + 2i32,
                        );
                }
                0b11100000..=0b11101111 => {
                    let d: u8 = 16u8 + (content.1 & 0b11110000).shr(4);
                    let k: u8 = (content.0 & 0b00001111).shl(4) + (content.1 & 0b00001111);
                    println!("ldi r{}, {:#x}", d, k);
                }
                0b11110000..=0b11110011 => {
                    if (content.1 & 0b00000111) == 0b00000001 {
                        let k: i8 = (content.0.shl(6) | (content.1 & 0b11111000).shr(2)) as i8;
                         if k < 0 {
                            print!("breq .-{:#x}", k.abs());
                        } else {
                            print!("breq .+{:#x}", k)
                        }
                        println!(
                            " ; {:#x}",
                            (data.address as usize + i * 2) as i32 + k as i32 + 2i32,
                        );
                    } else {
                        break;
                    }
                }
                0b11110100..=0b11110111 => {
                    if (content.1 & 0b00000111) ==  0b00000001 {
                        let k: i8 = (content.0.shl(6) | (content.1 & 0b11111000).shr(2)) as i8;
                        if k < 0 {
                            print!("brne .-{:#x}", k.abs());
                        } else {
                            print!("brne .+{:#x}", k)
                        }
                        println!(
                            " ; {:#x}",
                            (data.address as usize + i * 2) as i32 + k as i32 + 2i32,
                        );
                    } else {
                        break;
                    }
                }
                _ => break,
            },
            None => break,
        }
    }
}
