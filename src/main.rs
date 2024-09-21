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

enum Instuction {
    Adc,
    Add,
    Adiw,
    And,
    Andi,
    Asr,
    Bclr,
    Bld,
    Brbc,
    Brbs,
    Brcc,
    Brcs,
    Break,
    Breq,
    Brge,
    Brhc,
    Brhs,
    Brid,
    Brie,
    Brlo,
    Brlt,
    Brmi,
    Brne,
    Brpl,
    Brsh,
    Brtc,
    Brts,
    Brvc,
    Brvs,
    Bset,
    Bst,
    Call,
    Cbi,
    Cbr,
    Clc,
    Clh,
    Cli,
    Cln,
    Clr,
    Cls,
    Clt,
    Clv,
    Clz,
    Com,
    Cp,
    Cpc,
    Cpi,
    Cpse,
    Dec,
    Eicall,
    Eijmp,
    Elpm,
    Eor,
    Fmul,
    Fmuls,
    Fmulsu,
    Icall,
    Ijmp,
    In,
    Inc,
    Jmp,
    Ld,
    Ldi,
    Lds,
    Lpm,
    Lsl,
    Lsr,
    Mov,
    Movw,
    Mul,
    Muls,
    Mulsu,
    Neg,
    Nop,
    Or,
    Ori,
    Out,
    Pop,
    Push,
    Rcall,
    Ret,
    Reti,
    Rjmp,
    Rol,
    Ror,
    Sbc,
    Sbci,
    Sbi,
    Sbic,
    Sbis,
    Sbiw,
    Sbr,
    Sbrs,
    Sec,
    Seh,
    Sei,
    Sen,
    Ser,
    Ses,
    Set,
    Sev,
    Sez,
    Sleep,
    Spm,
    St,
    Sts,
    Sub,
    Subi,
    Swap,
    Tst,
    Wdr,
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

    let mut iter = data.data.iter().enumerate();
    loop {
        match iter.next() {
            Some((i, content)) => match content.0 {
                0b00001100..=0b00001111 => {}
                _ => {}
            },
            None => break,
        }
    }
}
