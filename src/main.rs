use clap::Parser;

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
    data: Vec<u8>,
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
        Err(error) => {
            panic!("Can't deal with {}, just exit here", error);
        }
    };
    data.address = match u16::from_str_radix(&cli.hex[3..7], 16) {
        Ok(content) => content,
        Err(error) => {
            panic!("Can't deal with {}, just exit here", error);
        }
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
        Err(error) => {
            panic!("Can't deal with {}, just exit here", error)
        }
    };
    data.data.reserve(data.size as usize * 2);
    for i in (9..9 + (data.size as usize * 2)).step_by(2) {
        data.data
            .push(match u8::from_str_radix(&cli.hex[i..i + 2], 16) {
                Ok(content) => content,
                Err(error) => panic!("Can't deal with {}, just exit here", error),
            });
    }
    if data.data.len() > 0 {
        for i in (0..data.data.len() - 1).step_by(2) {
            data.data.swap(i, i + 1);
        }
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

    for i in data.data {
        println!("{:#010b}", i);
    }
}
