#![no_std]

use defmt::{debug, info, Format};
use embedded_io_async::{ErrorType, Read, Write};
use heapless::{format, String};

const PREAMBLE: u8 = 0x24; // &
const DATA_FIELD_TERMINATOR: u8 = 0x2a; // *
const CARRIAGE_RETURN: u8 = 0x0d; // \r
const LINE_FEED: u8 = 0x0a; // \n

#[derive(Debug, Format)]
pub enum MT3339Error<UART> {
    Uart(UART),
}

#[derive(Debug)]
pub struct MT3339<UART> {
    uart: UART,
}
//impl<UART: Read + ReadReady + Write + ErrorType> MT3339<UART> {
impl<UART: Read + Write + ErrorType> MT3339<UART> {
    pub fn new(uart: UART) -> Self {
        Self { uart }
    }

    pub async fn send_cmd(&mut self, cmd: &[u8]) -> Result<(), MT3339Error<UART::Error>> {
        debug!("MT3339.send_cmd(): {:?}", cmd);
        self.write(&[PREAMBLE]).await?;
        self.write(cmd).await?;
        self.write(&[DATA_FIELD_TERMINATOR]).await?;
        self.write(&checksum(cmd)).await?;
        self.write(&[CARRIAGE_RETURN, LINE_FEED]).await?;
        Ok(())
    }

    ///// Check for updated data from the GPS module and process it accordingly.  Returns True if new data was processed, and False if nothing new was received.
    // pub async fn update(&mut self) -> Result<(), MT3339Error<UART::Error>> {
    //     debug!("MT3339.update()");
    //     // Grab a sentence and check its data type to call the appropriate parsing function.
    //     // parse_sentence()
    //     if self.uart.read_ready().unwrap() {
    //         info!("data to read!");
    //     } else {
    //         info!("nothing to read");
    //     }
    //     // let mut buf = [0u8; 128];
    //     // let len = self.uart.read_to_break(&mut buf).await.map_err(MT3339Error::Uart)?;
    //     // info!("bytes read: {}", len);
    //     Ok(())
    // }

    async fn write(&mut self, data: &[u8]) -> Result<(), MT3339Error<UART::Error>> {
        debug!("MT3339.write(): {:?}", data);
        self.uart.write(data).await.map_err(MT3339Error::Uart)?;
        Ok(())
    }
}


pub(crate) fn checksum(data: &[u8]) -> [u8; 2] {
    let mut checksum = 0;
    for c in data {
        checksum ^= *c;
    }
    let msg: String<2> = format!(2; "{:X?}", checksum).unwrap();
    msg.into_bytes().into_array().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_nmea_out() {
        assert_eq!(checksum("PMTK314,1,1,1,1,1,5,0,0,0,0,0,0,0,0,0,0,0,0,0".as_bytes()), [0x31, 0x39]);
    }

    #[test]
    fn checksum_pmtk_ack() {
        assert_eq!(checksum("PMTK001,604,3".as_bytes()), [0x33, 0x32]);
    }
}