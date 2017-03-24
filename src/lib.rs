extern crate sysfs_gpio;

use std::thread::sleep;
use std::time::Duration;

use sysfs_gpio::{Direction, Error, Pin};


#[derive(Clone, Debug)]
pub struct TM1637 {
    clk: Pin,
    dio: Pin,
    brightness: u8,
    bit_delay: Duration,
}

// TODO
#[derive(Debug)]
pub enum Brightness {
    Off,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

// TODO: Rename
const I2C_COMM1: u8 = 0x40;
const I2C_COMM2: u8 = 0xC0;
const I2C_COMM3: u8 = 0x80;


pub fn byte_to_segments(b: u8) -> Option<u8> {
    match b {
        b'0' => Some(0b0111111),
        b'1' => Some(0b0000110),
        b'2' => Some(0b1011011),
        b'3' => Some(0b1001111),
        b'4' => Some(0b1100110),
        b'5' => Some(0b1101101),
        b'6' => Some(0b1111101),
        b'7' => Some(0b0000111),
        b'8' => Some(0b1111111),
        b'9' => Some(0b1101111),
        b'A' => Some(0b1110111),
        b'b' => Some(0b1111100),
        b'C' => Some(0b0111001),
        b'd' => Some(0b1011110),
        b'E' => Some(0b1111001),
        b' ' => Some(0b0000000),
        _ => None,
    }
}

pub fn char_to_segments(c: char) -> Option<u8> {
    match c {
        '0' => Some(0b0111111),
        '1' => Some(0b0000110),
        '2' => Some(0b1011011),
        '3' => Some(0b1001111),
        '4' => Some(0b1100110),
        '5' => Some(0b1101101),
        '6' => Some(0b1111101),
        '7' => Some(0b0000111),
        '8' => Some(0b1111111),
        '9' => Some(0b1101111),
        'A' => Some(0b1110111),
        'b' => Some(0b1111100),
        'C' => Some(0b0111001),
        'd' => Some(0b1011110),
        'E' => Some(0b1111001),
        ' ' => Some(0b0000000),
        _ => None,
    }
}

pub fn string_to_segments(string: &str) -> Result<Vec<u8>, String> {
    let mut chars = string.chars().peekable();
    let mut segments = Vec::new();
    while let Some(c) = chars.next() {
        if let Some(s) = char_to_segments(c) {
            segments.push(if chars.peek() == Some(&':') {
                chars.next();
                s | 0x80
            } else {
                s
            });
        } else {
            return Err(format!("Invalid char {} in string", c));
        }
    }
    return Ok(segments);
}

pub fn bytestring_to_segments(string: &[u8]) -> Result<Vec<u8>, String> {
    let mut bytes = string.iter().map(|&b| b).peekable();
    let mut segments = Vec::new();
    while let Some(b) = bytes.next() {
        if let Some(s) = byte_to_segments(b) {
            segments.push(if bytes.peek() == Some(&b':') {
                bytes.next();
                s | 0x80
            } else {
                s
            });
        } else {
            return Err(format!("Invalid byte {} in string", b));
        }
    }
    return Ok(segments);
}

impl TM1637 {
    /// Creates a TM1637 object using the given pins and sets the pins to output.
    ///
    /// The pins must already be exported.
    pub fn new(clk: Pin, dio: Pin) -> Result<Self, Error> {
        let tm = TM1637 {
            clk: clk,
            dio: dio,
            brightness: 0xf,
            bit_delay: Duration::new(0, 1000), // 1 microsecond
        };
        tm.setup()?;
        Ok(tm)
    }

    fn setup(&self) -> Result<(), Error> {
        self.clk.set_direction(Direction::High)?;
        self.dio.set_direction(Direction::High)
    }

    pub fn set_string(&self, s: &str) -> Result<(), Error> {
        // TODO: Return error instead of panicking
        self.set_segments(&string_to_segments(s).expect("Undisplayable chars in string"))
    }

    pub fn set_bytestring(&self, s: &[u8]) -> Result<(), Error> {
        // TODO: Return error instead of panicking
        self.set_segments(&bytestring_to_segments(s).expect("Undisplayable bytes in bytestring"))
    }


    fn bit_delay(&self) {
        sleep(self.bit_delay);
    }

    pub fn set_segments(&self, segments: &[u8]) -> Result<(), Error> {
        // Write COMM1
        self.start()?;
        self.write_byte(I2C_COMM1)?;
        self.stop()?;

        let pos = 0;  // Starting index

        // Write COMM2 + first digit address
        self.start()?;
        self.write_byte(I2C_COMM2 + pos)?;
        for &s in segments.iter() {
            self.write_byte(s)?;
        }
        self.stop()?;

        // Write COMM3 + brightness
        self.start()?;
        self.write_byte(I2C_COMM3 + self.brightness)?;
        self.stop()
    }

    fn start(&self) -> Result<(), Error> {
        self.dio.set_value(0)?;
        self.bit_delay();
        Ok(())
    }

    fn stop(&self) -> Result<(), Error> {
        self.dio.set_value(0)?;
        self.bit_delay();
        self.clk.set_value(1)?;
        self.bit_delay();
        self.dio.set_value(1)?;
        self.bit_delay();
        Ok(())
    }

    fn write_byte(&self, mut b: u8) -> Result<(), Error> {
        for _ in 0..8 {
            // clk low
            self.clk.set_value(0)?;
            self.bit_delay();

            self.dio.set_value(b & 1)?;

            self.bit_delay();

            self.clk.set_value(1)?;
            self.bit_delay();
            b >>= 1;
        }

        self.clk.set_value(0)?;
        self.bit_delay();
        self.clk.set_value(1)?;
        self.bit_delay();
        self.clk.set_value(0)?;
        self.bit_delay();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
