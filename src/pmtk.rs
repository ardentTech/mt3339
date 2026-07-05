use core::fmt::{Display, Formatter};
use defmt::Format;
use heapless::{format, String};
use crate::pmtk::PmtkError::InvalidInput;

#[derive(Debug)]
pub enum PmtkError {
    InvalidInput
}

const SET_NMEA_UPDATE_RATE: u16 = 220;
const SET_NMEA_UPDATE_RATE_MIN: u16 = 100;
const SET_NMEA_UPDATE_RATE_MAX: u16 = 10000;

const SET_NMEA_OUTPUT: u16 = 314;

pub struct NmeaPacket<const N: usize> {
    pub(crate) pmtk_command: PmtkCommand<N>
}
impl<const N: usize> Display for NmeaPacket<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // TODO can capacity be calculated dynamically? just set to 255?
        let cmd: String<51> = format!("{}", self.pmtk_command)?;
        let checksum = generate_checksum(cmd.as_bytes());
        write!(f, "${}*{:X?}\r\n", cmd, checksum)
    }
}

#[derive(Debug)]
pub struct PmtkCommand<const N: usize> {
    data_fields: [u16; N],
    pkt_type: u16
}
impl<const N: usize> Display for PmtkCommand<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "PMTK{}", self.pkt_type)?;
        for c in self.data_fields {
            write!(f, ",{}", c)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Format)]
pub struct SetNmeaUpdateRate(pub(crate) u16);
impl SetNmeaUpdateRate {
    pub fn new(ms: u16) -> Result<Self, PmtkError> {
        if !Self::validate(ms) {
            return Err(InvalidInput)
        }
        Ok(Self(ms))
    }

    fn validate(ms: u16) -> bool {
        SET_NMEA_UPDATE_RATE_MIN < ms && ms < SET_NMEA_UPDATE_RATE_MAX
    }
}
impl Into<PmtkCommand<1>> for SetNmeaUpdateRate {
    fn into(self) -> PmtkCommand<1> {
        PmtkCommand {
            data_fields: [self.0],
            pkt_type: SET_NMEA_UPDATE_RATE
        }
    }
}

// TODO builder pattern?
#[derive(Debug, Default, Format)]
pub struct SetNmeaOutput {
    pub gll: FrequencySetting,
    pub rmc: FrequencySetting,
    pub vtg: FrequencySetting,
    pub gga: FrequencySetting,
    pub gsa: FrequencySetting,
    pub gsv: FrequencySetting,
    pub zda: FrequencySetting,
    pub mchn: FrequencySetting,
}
impl Into<PmtkCommand<19>> for SetNmeaOutput {
    fn into(self) -> PmtkCommand<19> {
        PmtkCommand {
            data_fields: [
                self.gll as u16,
                self.rmc as u16,
                self.vtg as u16,
                self.gga as u16,
                self.gsa as u16,
                self.gsv as u16,
                0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                self.zda as u16,
                self.mchn as u16,
            ],
            pkt_type: SET_NMEA_OUTPUT
        }
    }
}
// TODO fn new

pub(crate) fn generate_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0, |acc, &x| acc ^ x)
}

#[derive(Debug, Default, Format)]
pub enum FrequencySetting {
    #[default]
    Disabled = 0x0,
    OnePositionFix = 0x1,
    TwoPositionFixes = 0x2,
    ThreePositionFixes = 0x3,
    FourPositionFixes = 0x4,
    FivePositionFixes = 0x5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_checksum_ok() {
        assert_eq!(generate_checksum("PMTK314,1,1,1,1,1,5,0,0,0,0,0,0,0,0,0,0,0,0,0".as_bytes()), 0x2c);
    }

    #[test]
    fn set_nmea_output_display_ok() {
        let mut data = SetNmeaOutput::default();
        data.gll = FrequencySetting::OnePositionFix;
        data.rmc = FrequencySetting::OnePositionFix;
        data.vtg = FrequencySetting::OnePositionFix;
        data.gga = FrequencySetting::OnePositionFix;
        data.gsa = FrequencySetting::OnePositionFix;
        data.gsv = FrequencySetting::FivePositionFixes;
        let pmtk_command: PmtkCommand<19> = data.into();
        let res = format!(45; "{}", pmtk_command).unwrap();
        assert_eq!(res, "PMTK314,1,1,1,1,1,5,0,0,0,0,0,0,0,0,0,0,0,0,0");
    }

    #[test]
    fn set_nmea_update_rate_invalid_low() {
        match SetNmeaUpdateRate::new(SET_NMEA_UPDATE_RATE_MIN - 1) {
            Ok(_) => panic!(),
            Err(_) => {}
        }
    }

    #[test]
    fn set_nmea_update_rate_invalid_high() {
        match SetNmeaUpdateRate::new(SET_NMEA_UPDATE_RATE_MAX + 1) {
            Ok(_) => panic!(),
            Err(_) => {}
        }
    }

    #[test]
    fn set_nmea_update_rate_display_ok() {
        let data = SetNmeaUpdateRate::new(1000).unwrap();
        let pmtk_command: PmtkCommand<1> = data.into();
        let res = format!(15; "{}", pmtk_command).unwrap();
        assert_eq!(res, "PMTK220,1000");
    }

    #[test]
    fn nmea_packet_display_ok() {
        let mut data = SetNmeaOutput::default();
        data.gll = FrequencySetting::OnePositionFix;
        data.rmc = FrequencySetting::OnePositionFix;
        data.vtg = FrequencySetting::OnePositionFix;
        data.gga = FrequencySetting::OnePositionFix;
        data.gsa = FrequencySetting::OnePositionFix;
        data.gsv = FrequencySetting::FivePositionFixes;
        let pmtk_command: PmtkCommand<19> = data.into();
        let nmea_packet = NmeaPacket { pmtk_command };
        let res = format!(51; "{}", nmea_packet).unwrap();
        assert_eq!(res, "$PMTK314,1,1,1,1,1,5,0,0,0,0,0,0,0,0,0,0,0,0,0*2C\r\n");
    }
}