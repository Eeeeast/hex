use bitmatch::bitmatch;
use clap::Parser;
use std::{
    fmt::{self, Debug},
    ops::BitXor,
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Hex code
    hex: Vec<String>,
    /// Advanced
    #[arg(short, long, default_value_t = false)]
    advanced: bool,
    /// Operator overloading
    #[arg(short, long, default_value_t = true)]
    overloads: bool,
}

#[derive(Debug, PartialEq)]
enum Index {
    Data = 0,
    End = 1,
    AddressSegment = 2,
    StartAddress80x86 = 3,
    ExtendedAddress = 4,
    LinearAdrres = 5,
}

struct Record {
    address: u16,
    index: Index,
    data: Vec<(u8, u8)>,
}

enum RecordParseError {
    BeginningOfRecord,
    CalculatingTheSize,
    CalculatingTheAddress,
    CalculatingIndex,
    CalculatingData,
    CalculatingChecksum,
}

impl Record {
    fn from_str(hex: &String) -> Result<Self, RecordParseError> {
        if &hex[0..1] != ":" {
            return Err(RecordParseError::BeginningOfRecord);
        }
        let mut data: Self = Record {
            address: match u16::from_str_radix(&hex[3..7], 16) {
                Ok(content) => content,
                _ => return Err(RecordParseError::CalculatingTheAddress),
            },
            index: match u8::from_str_radix(&hex[7..9], 16) {
                Ok(content) => match content {
                    0 => Index::Data,
                    1 => Index::End,
                    2 => Index::AddressSegment,
                    3 => Index::StartAddress80x86,
                    4 => Index::ExtendedAddress,
                    5 => Index::LinearAdrres,
                    _ => return Err(RecordParseError::CalculatingIndex),
                },
                _ => return Err(RecordParseError::CalculatingIndex),
            },
            data: vec![],
        };
        data.data
            .reserve(match usize::from_str_radix(&hex[1..3], 16) {
                Ok(content) => content,
                _ => return Err(RecordParseError::CalculatingTheSize),
            });
        for i in (9..9 + (data.data.capacity() * 2)).step_by(4) {
            data.data.push((
                match u8::from_str_radix(&hex[i + 2..i + 4], 16) {
                    Ok(content) => content,
                    _ => return Err(RecordParseError::CalculatingData),
                },
                match u8::from_str_radix(&hex[i..i + 2], 16) {
                    Ok(content) => content,
                    _ => return Err(RecordParseError::CalculatingData),
                },
            ));
        }
        match u8::from_str_radix(
            &hex[9 + data.data.len() * 4..9 + data.data.len() * 4 + 2],
            16,
        ) {
            Ok(_) => Ok(data),
            _ => return Err(RecordParseError::CalculatingChecksum),
        }
    }
}

impl fmt::Display for Record {
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
                    result = writeln!(f, "    ({:#010b}, {:#010b}), ", i.0, i.1);
                } else {
                    return result;
                }
            }
        }
        result
    }
}

fn from_additional_code(sign: bool, number: u16, mask: u16) -> i16 {
    match sign {
        true => (number.bitxor(mask) + 1) as i16 * -1,
        false => number as i16,
    }
}

#[bitmatch]
fn main() {
    let cli: Cli = Cli::parse();
    let mut records: Vec<Record> = vec![];
    for record in cli.hex {
        let mut data = match Record::from_str(&record) {
            Ok(content) => content,
            Err(error) => {
                match error {
                    RecordParseError::BeginningOfRecord => {
                        panic!("Each record in the Intel HEX file must start with a colon")
                    }
                    RecordParseError::CalculatingTheSize => {
                        panic!("Error when calculating the data size, you need one byte (two hexadecimal digits), which in decimal is between 0 and 255")
                    }
                    RecordParseError::CalculatingTheAddress => {
                        panic!("Error when calculating the starting address, data block which is 2 bytes and indicates the absolute position of the record data in the binary file")
                    }
                    RecordParseError::CalculatingIndex => {
                        panic!("The field type is expected to take the following values: 0, 1, 2, 3, 4, 5")
                    }
                    RecordParseError::CalculatingData => {
                        panic!("Error when reading data bytes for writing to EPROM, the number of bytes to be written is specified at the beginning, in the range from 0 to 255 bytes")
                    }
                    RecordParseError::CalculatingChecksum => {
                        panic!("Error when reading the last byte of a record, a checksum calculated so that the sum of all bytes in the record is zero")
                    }
                }
            }
        };
        match records.last_mut() {
            Some(record) => {
                if ((record.address + record.data.len() as u16 * 2) == data.address)
                    && (record.index == data.index)
                {
                    record.data.append(&mut data.data);
                } else {
                    records.push(data);
                }
            }
            None => records.push(data),
        };
    }
    for data in records {
        if cli.advanced {
            println!("{}", data);
        }
        let mut iter = data.data.into_iter().enumerate();
        loop {
            match iter.next() {
                Some((mut i, content)) => {
                    i = i * 2 + data.address as usize;
                    print!("{:#x}: ", i);
                    #[bitmatch]
                    match u16::from_be_bytes([content.0, content.1]) {
                        "0000_0000_0000_0000" => println!("nop"),
                        "0000_0001_dddd_rrrr" => {
                            println!("movw r{}:{}, r{}:{}", d * 2 + 1, d * 2, r * 2 + 1, r * 2)
                        }
                        "0000_0010_dddd_rrrr" => println!("muls r{}, r{}", d + 16, r + 16),
                        "0000_0011_0ddd_0rrr" => println!("mulsu r{}, r{}", d + 16, r + 16),
                        "0000_0011_0ddd_1rrr" => println!("fmul r{}, r{}", d + 16, r + 16),
                        "0000_0011_1ddd_0rrr" => println!("fmuls r{}, r{}", d + 16, r + 16),
                        "0000_0011_1ddd_1rrr" => println!("fmulsu r{}, r{}", d + 16, r + 16),
                        "0000_01rd_dddd_rrrr" => println!("cpc r{}, r{}", d, r),
                        "0000_10rd_dddd_rrrr" => println!("sbc r{}, r{}", d, r),
                        "0000_11rd_dddd_rrrr" => match (d == r) && cli.overloads {
                            true => println!("lsl r{}", d),
                            false => println!("add r{}, r{}", d, r),
                        },
                        "0001_00rd_dddd_rrrr" => println!(
                            "cpse r{}, r{} ; {:#x} (or {:#x})",
                            d,
                            r,
                            i + 2 * 2,
                            i + 3 * 2
                        ),
                        "0001_01rd_dddd_rrrr" => println!("cp r{}, r{}", d, r),
                        "0001_10rd_dddd_rrrr" => println!("sub r{}, r{}", d, r),
                        "0001_11rd_dddd_rrrr" => match (d == r) && cli.overloads {
                            true => println!("rol r{}", d),
                            false => println!("adc r{}, r{}", d, r),
                        },
                        "0010_00rd_dddd_rrrr" => match (d == r) && cli.overloads {
                            true => println!("tst r{}", d),
                            false => println!("and r{}, r{}", d, r),
                        },
                        "0010_01rd_dddd_rrrr" => match (d == r) && cli.overloads {
                            true => println!("clr r{}", d),
                            false => println!("eor r{}, r{}", d, r),
                        },
                        "0010_10rd_dddd_rrrr" => println!("or r{}, r{}", d, r),
                        "0010_11rd_dddd_rrrr" => println!("mov r{}, r{}", d, r),
                        "0011_kkkk_dddd_kkkk" => println!("cpi r{}, {}", d + 16, k),
                        "0100_kkkk_dddd_kkkk" => println!("sbci r{}, {}", d + 16, k),
                        "0101_kkkk_dddd_kkkk" => println!("subi r{}, {:#x}", d + 16, k),
                        "0110_kkkk_dddd_kkkk" => println!("ori r{}, {:#x}", d + 16, k),
                        "0111_kkkk_dddd_kkkk" => println!("andi r{}, {:#x}", d + 16, k),
                        "1000_000d_dddd_0000" => println!("ld r{}, Z", d),
                        "1000_000d_dddd_1000" => println!("ld r{}, Y", d),
                        "1000_001r_rrrr_0000" => println!("st Z, r{}", r),
                        "1000_001r_rrrr_1000" => println!("st Y, r{}", r),
                        "1000_001r_rrrr_1001" => println!("st Y+, r{}", r),
                        "1000_001r_rrrr_1010" => println!("st -Y, r{}", r),
                        "1001_000d_dddd_0000" => match iter.next() {
                            Some((_, extension)) => println!(
                                "lds r{}, {:#x}",
                                d,
                                u16::from_be_bytes([extension.0, extension.1])
                            ),
                            _ => panic!("instruction lds was expected"),
                        },
                        "1001_000d_dddd_0001" => println!("ld r{}, Z+", d),
                        "1001_000d_dddd_0010" => println!("ld r{}, -Z", d),
                        "1001_000d_dddd_0100" => println!("lpm r{}, Z", d),
                        "1001_000d_dddd_0101" => println!("lpm r{}, Z+", d),
                        "1001_000d_dddd_0110" => println!("elpm r{}, Z", d),
                        "1001_000d_dddd_0111" => println!("elpm r{}, z+", d),
                        "1001_000d_dddd_1001" => println!("ld r{}, Y+", d),
                        "1001_000d_dddd_1010" => println!("ld r{}, -Y", d),
                        "1001_000d_dddd_1100" => println!("ld r{}, X", d),
                        "1001_000d_dddd_1101" => println!("ld r{}, X+", d),
                        "1001_000d_dddd_1110" => println!("ld r{}, -X", d),
                        "1001_000d_dddd_1111" => println!("pop r{}", d),
                        "1001_001d_dddd_0000" => match iter.next() {
                            Some((_, extension)) => println!(
                                "sts {}, r{}",
                                u16::from_be_bytes([extension.0, extension.1]),
                                d
                            ),
                            _ => panic!("instruction sts was expected"),
                        },
                        "1001_001r_rrrr_0001" => println!("st Z+, r{}", r),
                        "1001_001r_rrrr_0010" => println!("st -Z, r{}", r),
                        "1001_001r_rrrr_1100" => println!("st X, r{}", r),
                        "1001_001r_rrrr_1101" => println!("st X+, r{}", r),
                        "1001_001r_rrrr_1110" => println!("st X-, r{}", r),
                        "1001_001d_dddd_1111" => println!("push r{}", d),
                        "1001_0100_0000_1000" if cli.overloads => println!("sec"),
                        "1001_0100_0001_1000" if cli.overloads => println!("sez"),
                        "1001_0100_0010_1000" if cli.overloads => println!("sen"),
                        "1001_0100_0011_1000" if cli.overloads => println!("sev"),
                        "1001_0100_0100_1000" if cli.overloads => println!("ses"),
                        "1001_0100_0101_1000" if cli.overloads => println!("seh"),
                        "1001_0100_0110_1000" if cli.overloads => println!("set"),
                        "1001_0100_0111_1000" if cli.overloads => println!("sei"),
                        "1001_0100_0000_1001" => println!("ijmp"),
                        "1001_0100_0001_1001" => println!("eijmp"),
                        "1001_0100_0sss_1000" => println!("bset {}", s),
                        "1001_0100_1000_1000" if cli.overloads => println!("clc"),
                        "1001_0100_1001_1000" if cli.overloads => println!("clz"),
                        "1001_0100_1010_1000" if cli.overloads => println!("cln"),
                        "1001_0100_1011_1000" if cli.overloads => println!("clv"),
                        "1001_0100_1100_1000" if cli.overloads => println!("cls"),
                        "1001_0100_1101_1000" if cli.overloads => println!("clh"),
                        "1001_0100_1110_1000" if cli.overloads => println!("clt"),
                        "1001_0100_1111_1000" if cli.overloads => println!("cli"),
                        "1001_0100_1sss_1000" => println!("bclr {}", s),
                        "1001_0101_0000_1000" => println!("ret"),
                        "1001_0101_0000_1001" => println!("icall"),
                        "1001_0101_0001_1000" => println!("reti"),
                        "1001_0101_0001_1001" => println!("eicall"),
                        "1001_0101_1000_1000" => println!("sleep"),
                        "1001_0101_1001_1000" => println!("break"),
                        "1001_0101_1010_1000" => println!("wdr"),
                        "1001_0101_1100_1000" => println!("lpm"),
                        "1001_0101_1101_1000" => println!("elpm"),
                        "1001_0101_1110_1000" => println!("spm"),
                        "1001_010d_dddd_0000" => println!("com r{}", d),
                        "1001_010d_dddd_0001" => println!("neg r{}", d),
                        "1001_010d_dddd_0010" => println!("swap r{}", d),
                        "1001_010d_dddd_0011" => println!("inc r{}", d),
                        "1001_010d_dddd_0101" => println!("asr r{}", d),
                        "1001_010d_dddd_0110" => println!("lsr r{}", d),
                        "1001_010d_dddd_0111" => println!("ror r{}", d),
                        "1001_010d_dddd_1010" => println!("dec r{}", d),
                        "1001_010k_kkkk_110k" => match iter.next() {
                            Some((_, extension)) => println!(
                                "jmp {:#x} ; {:#x}",
                                u32::from_be_bytes([0, k as u8, extension.0, extension.1]) * 2,
                                u32::from_be_bytes([0, k as u8, extension.0, extension.1]) * 2
                            ),
                            _ => panic!("instruction jmp was expected"),
                        },
                        "1001_010k_kkkk_111k" => match iter.next() {
                            Some((_, extension)) => println!(
                                "call {:#x} ; {:#x}",
                                u32::from_be_bytes([0, k as u8, extension.0, extension.1]) * 2,
                                u32::from_be_bytes([0, k as u8, extension.0, extension.1]) * 2
                            ),
                            _ => panic!("instruction call was expected"),
                        },
                        "1001_0110_kkdd_kkkk" => {
                            println!("adiw r{}:{}, {}", d * 2 + 25, d * 2 + 24, k)
                        }
                        "1001_0111_kkdd_kkkk" => {
                            println!("sbiw r{}:{}, {}", d * 2 + 25, d * 2 + 24, k)
                        }
                        "1001_1000_aaaa_abbb" => println!("cbi {:#x}, {}", a, b),
                        "1001_1001_aaaa_abbb" => println!(
                            "sbic {:#x}, {} ; {:#x} (or {:#x})",
                            a,
                            b,
                            i + 2 * 2,
                            i + 3 * 2
                        ),
                        "1001_1010_aaaa_abbb" => println!("sbi {:#x}, {}", a, b),
                        "1001_1011_aaaa_abbb" => println!(
                            "sbis {:#x}, {} ; {:#x} (or {:#x})",
                            a,
                            b,
                            i + 2 * 2,
                            i + 3 * 2
                        ),
                        "1001_11rd_dddd_rrrr" => println!("mul r{}, r{}", d, r),
                        "1011_0aad_dddd_aaaa" => println!("in r{}, {:#x}", d, a),
                        "1011_1aar_rrrr_aaaa" => println!("out {:#x}, r{}", a, r),
                        "10q0_qq1r_rrrr_0qqq" => println!("std Z+{}, r{}", q, r),
                        "10q0_qq1r_rrrr_1qqq" => println!("std Y+{}, r{}", q, r),
                        "10q0_qq0d_dddd_0qqq" => println!("ldd r{}, Z+{}", d, q),
                        "10q0_qq0d_dddd_1qqq" => println!("ldd r{}, Y+{}", d, q),
                        "1100_ekkk_kkkk_kkkk" => println!(
                            "rjmp .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0111_1111_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0111_1111_1111) * 2
                                + 2
                        ),
                        "1101_ekkk_kkkk_kkkk" => println!(
                            "rcall .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0111_1111_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0111_1111_1111) * 2
                                + 2
                        ),
                        "1110_kkkk_dddd_kkkk" => match (k == u16::MAX) && cli.overloads {
                            true => println!("ser r{}", d),
                            false => println!("ldi r{}, {:#x}", d + 16, k),
                        },
                        "1111_00ek_kkkk_k000" if cli.overloads => println!(
                            "brcs .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_00ek_kkkk_k001" if cli.overloads => println!(
                            "breq .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_00ek_kkkk_k010" if cli.overloads => println!(
                            "brmi .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_00ek_kkkk_k011" if cli.overloads => println!(
                            "brvs .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_00ek_kkkk_k100" if cli.overloads => println!(
                            "brlt .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_00ek_kkkk_k101" if cli.overloads => println!(
                            "brhs .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_00ek_kkkk_k110" if cli.overloads => println!(
                            "brts .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_00ek_kkkk_k111" if cli.overloads => println!(
                            "brie .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_00ek_kkkk_ksss" => println!(
                            "brbs {}, .{:+} ; {:#x}",
                            s,
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_01ek_kkkk_k000" if cli.overloads => println!(
                            "brcc .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_01ek_kkkk_k001" if cli.overloads => println!(
                            "brne .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_01ek_kkkk_k010" if cli.overloads => println!(
                            "brpl .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_01ek_kkkk_k011" if cli.overloads => println!(
                            "brvc .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_01ek_kkkk_k100" if cli.overloads => println!(
                            "brge .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_01ek_kkkk_k101" if cli.overloads => println!(
                            "brhc .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_01ek_kkkk_k110" if cli.overloads => println!(
                            "brtc .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_01ek_kkkk_k111" if cli.overloads => println!(
                            "brid .{:+} ; {:#x}",
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_01ek_kkkk_ksss" => println!(
                            "brid {}, .{:+} ; {:#x}",
                            s,
                            from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2,
                            i as i16
                                + from_additional_code(e == 1, k, 0b0000_0000_0011_1111) * 2
                                + 2
                        ),
                        "1111_100d_dddd_0bbb" => println!("bld r{}, {}", d, b),
                        "1111_101d_dddd_0bbb" => println!("bst r{}, {}", d, b),
                        "1111_110r_rrrr_0bbb" => println!(
                            "sbrc r{}, {} ; {:#x} (or {:#x})",
                            r,
                            b,
                            i + 2 * 2,
                            i + 3 * 2
                        ),
                        "1111_111r_rrrr_0bbb" => println!(
                            "sbrs r{}, {} ; {:#x} (or {:#x})",
                            r,
                            b,
                            i + 2 * 2,
                            i + 3 * 2
                        ),
                        _ => panic!(
                            "error, unexpected command (0b{:0>8b}_{:0>8b})",
                            content.0, content.1
                        ),
                    };
                }
                _ => break,
            }
        }
    }
}
