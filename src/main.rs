use bitmatch::bitmatch;
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
    /// Operator overloading
    #[arg(short, long, default_value_t = true)]
    overloads: bool,
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

struct Package {
    address: u16,
    index: Index,
    data: Vec<(u8, u8)>,
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
            data.data.push((
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
                    result = writeln!(f, "    ({:#010b}, {:#010b}), ", i.0, i.1);
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

#[bitmatch]
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
                print!("{:#x}: ", data.address + (i as u16) * 2);
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
                    "0001_00rd_dddd_rrrr" => println!("cpse r{}, r{}", d, r),
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
                    "0011_kkkk_dddd_kkkk" => println!("cpi, r{}, {}", d + 16, k),
                    "0100_kkkk_dddd_kkkk" => println!("sbci r{}, {}", d + 16, k),
                    "0101_kkkk_dddd_kkkk" => println!("subi r{}, {:#x}", d, k),
                    "0110_kkkk_dddd_kkkk" => println!("ori r{}, {:#x}", d + 16, k),
                    "0111_kkkk_dddd_kkkk" => println!("andi r{}, {:#x}", d + 16, k),
                    "1000_000d_dddd_0000" => println!("ld r{}, Z", d),
                    "1000_000d_dddd_1000" => println!("ld r{}, Y", d),
                    "1000_001r_rrrr_0000" => println!("st Z, r{}", r),
                    "1000_001r_rrrr_1000" => println!("st Y, r{}", r),
                    "1000_001r_rrrr_1001" => println!("st Y+, r{}", r),
                    "1000_001r_rrrr_1010" => println!("st -Y, r{}", r),
                    "10q0_qq1r_rrrr_1qqq" => println!("st Y+{}, r{}", q, r),
                    "1001_0101_1010_1000" => println!("wdr"),
                    "1001_000d_dddd_0000" => match iter.next() {
                        Some((_, extension)) => println!(
                            "lds r{}, {:#x}",
                            d,
                            u16::from_be_bytes([extension.0, extension.1])
                        ),
                        None => panic!("error, unexpected command"),
                    },
                    "1001_000d_dddd_0001" => println!("ld r{}, Z+", d),
                    "1001_000d_dddd_0010" => println!("ld r{}, -Z", d),
                    "1001_000d_dddd_0100" => println!("lpm r{}, Z", d),
                    "1001_000d_dddd_0101" => println!("lpm r{}, -Z", d),
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
                        None => panic!("error, unexpected command"),
                    },
                    "1001_001r_rrrr_0001" => println!("st Z+, r{}", r),
                    "1001_001r_rrrr_0010" => println!("st -Z, r{}", r),
                    "10q0_qq1r_rrrr_0qqq" => println!("st Z+{}, r{}", q, r),
                    "1001_001r_rrrr_1100" => println!("st X, r{}", r),
                    "1001_001r_rrrr_1101" => println!("st X+, r{}", r),
                    "1001_001r_rrrr_1110" => println!("st X-, r{}", r),
                    "1001_001d_dddd_1111" => println!("push r{}", d),
                    "1001_010d_dddd_0000" => println!("com r{}", d),
                    "1001_010d_dddd_0001" => println!("neg r{}", d),
                    "1001_010d_dddd_0010" => println!("swap r{}", d),
                    "1001_010d_dddd_0101" => println!("asr r{}", d),
                    "1001_010d_dddd_0110" => println!("lsr r{}", d),
                    "1001_0100_0000_1000" if cli.overloads => println!("sec"),
                    "1001_0100_0001_1000" if cli.overloads => println!("sez"),
                    "1001_0100_0010_1000" if cli.overloads => println!("sen"),
                    "1001_0100_0011_1000" if cli.overloads => println!("sev"),
                    "1001_0100_0100_1000" if cli.overloads => println!("ses"),
                    "1001_0100_0101_1000" if cli.overloads => println!("seh"),
                    "1001_0100_0110_1000" if cli.overloads => println!("set"),
                    "1001_0100_0111_1000" if cli.overloads => println!("sei"),
                    "1001_0100_0sss_1000" => println!("bset {}", s),
                    "1001_0100_0000_1001" => println!("ijmp"),
                    "1001_0100_0001_1001" => println!("eijmp"),
                    "1001_0100_1000_1000" if cli.overloads => println!("clc"),
                    "1001_0100_1001_1000" if cli.overloads => println!("clz"),
                    "1001_0100_1010_1000" if cli.overloads => println!("cln"),
                    "1001_0100_1011_1000" if cli.overloads => println!("clv"),
                    "1001_0100_1100_1000" if cli.overloads => println!("cls"),
                    "1001_0100_1101_1000" if cli.overloads => println!("clh"),
                    "1001_0100_1110_1000" if cli.overloads => println!("clt"),
                    "1001_0100_1111_1000" if cli.overloads => println!("cli"),
                    "1001_0100_1sss_1000" => println!("bclr {}", s),
                    "1001_0101_0000_1001" => println!("icall"),
                    "1001_0101_1000_1000" => println!("sleep"),
                    "1001_0101_1101_1000" => println!("elpm"),
                    "1001_0101_1110_1000" => println!("spm"),
                    "1001_010d_dddd_1010" => println!("dec r{}", d),
                    "1001_010d_dddd_0011" => println!("inc r{}", d),
                    "1001_010d_dddd_0111" => println!("ror r{}", d),
                    "1001_0101_0001_1001" => println!("eicall"),
                    "1001_0101_1100_1000" => println!("lpm"),
                    "1001_010k_kkkk_110k" => match iter.next() {
                        Some((_, extension)) => println!(
                            "jmp {:#x} ; {:#x}",
                            u32::from_be_bytes([0, k as u8, extension.0, extension.1]) * 2,
                            u32::from_be_bytes([0, k as u8, extension.0, extension.1]) * 2
                        ),
                        None => panic!("error, unexpected command"),
                    },
                    "1001_010k_kkkk_111k" => match iter.next() {
                        Some((_, extension)) => println!(
                            "call {:#x} ; {:#x}",
                            u32::from_be_bytes([0, k as u8, extension.0, extension.1]) * 2,
                            u32::from_be_bytes([0, k as u8, extension.0, extension.1]) * 2
                        ),
                        None => panic!("error, unexpected command"),
                    },
                    "1001_0101_0000_1000" => println!("ret"),
                    "1001_0101_0001_1000" => println!("reti"),
                    "1001_0101_1001_1000" => println!("break"),
                    "1001_0110_kkdd_kkkk" => println!("adiw r{}:{}, {}", d * 2 + 25, d * 2 + 24, k),
                    "1001_0111_kkdd_kkkk" => println!("sbiw r{}:{}, {}", d * 2 + 25, d * 2 + 24, k),
                    "1001_1000_aaaa_abbb" => println!("cbi {:#x}, {}", a, b),
                    "1001_1001_aaaa_abbb" => println!("sbic {:#x}, {}", a, b),
                    "1001_1010_aaaa_abbb" => println!("sbi {:#x}, {}", a, b),
                    "1001_1011_aaaa_abbb" => println!("sbis {:#x}, {}", a, b),
                    "1001_11rd_dddd_rrrr" => println!("mul r{}, r{}", d, r),
                    "10q0_qq0d_dddd_0ddd" => println!("ld r{}, Z+{}", d, q),
                    "10q0_qq0d_dddd_1qqq" => println!("ld r{}, Y+{}", d, q),
                    "1011_0aad_dddd_aaaa" => println!("in r{}, {:#x}", d, a),
                    "1011_1aar_rrrr_aaaa" => println!("out {:#x}, r{}", a, r),
                    "1100_ekkk_kkkk_kkkk" => match e == 1 {
                        true => println!(
                            "rjmp .-{:#x} ; {:#x}",
                            (!k & 0b0000_0111_1111_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0111_1111_1111) * 2
                        ),
                        false => println!(
                            "rjmp .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1101_ekkk_kkkk_kkkk" => match e == 1 {
                        true => println!(
                            "rcall .-{:#x} ; {:#x}",
                            (!k & 0b0000_0111_1111_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0111_1111_1111) * 2
                        ),
                        false => println!(
                            "rcall .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1110_kkkk_dddd_kkkk" => match (k == u16::MAX) && cli.overloads {
                        true => println!("ser r{}", d),
                        false => println!("ldi r{}, {:#x}", d + 16, k),
                    },
                    "1111_00ek_kkkk_k000" if cli.overloads => match e == 1 {
                        true => println!(
                            "brcs .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => println!(
                            "brcs .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_00ek_kkkk_k001" if cli.overloads => match e == 1 {
                        true => println!(
                            "breq .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_00000_0011_1111) * 2
                        ),
                        false => println!(
                            "breq .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_00ek_kkkk_k010" if cli.overloads => match e == 1 {
                        true => println!(
                            "brmi .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => println!(
                            "brmi .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_00ek_kkkk_k100" if cli.overloads => match e == 1 {
                        true => println!(
                            "brlt .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => println!(
                            "brlt .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_00ek_kkkk_k101" if cli.overloads => match e == 1 {
                        true => println!(
                            "brhs .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => println!(
                            "brhs .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_00ek_kkkk_k110" if cli.overloads => match e == 1 {
                        true => println!(
                            "brts .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => println!(
                            "brts .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_00ek_kkkk_k011" if cli.overloads => match e == 1 {
                        true => println!(
                            "brvs .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => println!(
                            "brvs .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_00ek_kkkk_k111" if cli.overloads => match e == 1 {
                        true => println!(
                            "brie .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => println!(
                            "brie .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_00ek_kkkk_ksss" => match e == 1 {
                        true => println!(
                            "brbs {}, .-{:#x} ; {:#x}",
                            s,
                            (!k & 0b0000_0000_0011_0000) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => {
                            println!(
                                "brbs {}, .{:#x} ; {:#x}",
                                s,
                                (k + 1) * 2,
                                data.address + (i as u16 + k + 1) * 2
                            )
                        }
                    },
                    "1111_01ek_kkkk_k000" if cli.overloads => match e == 1 {
                        true => println!(
                            "brcc .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => println!(
                            "brcc .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_01ek_kkkk_k001" if cli.overloads => match e == 1 {
                        true => println!(
                            "brne .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => println!(
                            "brne .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_01ek_kkkk_k010" if cli.overloads => match e == 1 {
                        true => println!(
                            "brpl .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => println!(
                            "brpl .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_01ek_kkkk_k011" if cli.overloads => match e == 1 {
                        true => println!(
                            "brvc .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => println!(
                            "brvc .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_01ek_kkkk_k100" if cli.overloads => match e == 1 {
                        true => println!(
                            "brge .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => {
                            println!(
                                "brge .+{:#x} ; {:#x}",
                                (k + 1) * 2,
                                data.address + (i as u16 + k + 1) * 2
                            )
                        }
                    },
                    "1111_01ek_kkkk_k110" if cli.overloads => match e == 1 {
                        true => println!(
                            "brtc .-{:#x} ; {:#x}",
                            (!k & 0b0000_00000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => {
                            println!(
                                "brtc .+{:#x} ; {:#x}",
                                (k + 1) * 2,
                                data.address + (i as u16 + k + 1) * 2
                            )
                        }
                    },
                    "1111_01ek_kkkk_k101" if cli.overloads => match e == 1 {
                        true => println!(
                            "brhc .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_00000_0011_1111) * 2
                        ),
                        false => {
                            println!(
                                "brhc .+{:#x} ; {:#x}",
                                (k + 1) * 2,
                                data.address + (i as u16 + k + 1) * 2
                            )
                        }
                    },
                    "1111_01ek_kkkk_k111" if cli.overloads => match e == 1 {
                        true => println!(
                            "brid .-{:#x} ; {:#x}",
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => println!(
                            "brid .+{:#x} ; {:#x}",
                            (k + 1) * 2,
                            data.address + (i as u16 + k + 1) * 2
                        ),
                    },
                    "1111_01ek_kkkk_ksss" => match e == 1 {
                        true => println!(
                            "brbc {}, .-{:#x} ; {:#x}",
                            s,
                            (!k & 0b0000_0000_0011_1111) * 2,
                            data.address + i as u16 * 2 - (!k & 0b0000_0000_0011_1111) * 2
                        ),
                        false => {
                            println!(
                                "brbc {}, .+{:#x} ; {:#x}",
                                s,
                                (k + 1) * 2,
                                data.address + (i as u16 + k + 1) * 2
                            )
                        }
                    },
                    "1111_100d_dddd_0bbb" => println!("bld r{}, {}", d, b),
                    "1111_101d_dddd_0bbb" => println!("bst r{}, {}", d, b),
                    "1111_110r_rrrr_0bbb" => println!("sbrc r{}, {}", r, b),
                    "1111_111r_rrrr_0bbb" => println!("sbrs r{}, {}", r, b),
                    _ => panic!("error, unexpected command"),
                };
            }
            None => break,
        }
    }
}
