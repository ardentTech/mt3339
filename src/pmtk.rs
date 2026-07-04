use core::fmt::{Display, Formatter};
use heapless::{format, String};

const PACKET_LEN: usize = 255;

fn generate_checksum(data: &[u8]) -> [u8; 2] {
    let mut checksum = 0;
    for c in data {
        checksum ^= *c;
    }
    let msg = format!(2; "{:X?}", checksum).unwrap();
    msg.into_bytes().into_array().unwrap()
}

#[derive(Debug, Default)]
enum FrequencySetting {
    #[default]
    Disabled = 0x0,
    OnePositionFix = 0x1,
    TwoPositionFix = 0x2,
    ThreePositionFix = 0x3,
    FourPositionFix = 0x4,
    FivePositionFix = 0x5,
}

// TODO builder pattern?
#[derive(Debug, Default)]
pub struct SetNmeaOutput {
    gll: FrequencySetting,
    rmc: FrequencySetting,
    vtg: FrequencySetting,
    gga: FrequencySetting,
    gsa: FrequencySetting,
    gsv: FrequencySetting,
    zda: FrequencySetting,
    mchn: FrequencySetting,
}

impl Into<DataField> for SetNmeaOutput {
    fn into(self) -> DataField {
        DataField([
            self.gll as u8,
            self.rmc as u8,
            self.vtg as u8,
            self.gga as u8,
            self.gsa as u8,
            self.gsv as u8,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            self.zda as u8,
            self.mchn as u8,
        ])
    }
}

// TODO define supertrait for Into
impl Into<Command> for SetNmeaOutput {
    fn into(self) -> Command {
        // TODO figure out where to put PktType
        Command::new(self.into(), PktType([0x33, 0x31, 0x34]))
    }
}

#[derive(Debug, Default)]
struct DataField([u8; 19]);
impl Display for DataField {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for c in self.0 {
            write!(f, ",{}", c)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
struct PktType([u8; 3]);
impl Display for PktType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}{}", self.0[0] as char, self.0[1] as char, self.0[2] as char)
    }
}

#[derive(Debug, Default)]
struct Command {
    data_field: DataField,
    pkt_type: PktType
}

impl Command {
    fn new(data_field: DataField, pkt_type: PktType) -> Self {
        Self {
            data_field,
            pkt_type,
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let payload: String<45> = format!("PMTK{}{}", self.pkt_type, self.data_field)?;
        let checksum = generate_checksum(payload.as_bytes());
        write!(f, "$PMTK{}{}*{}{}\r\n", self.pkt_type, self.data_field, checksum[0] as char, checksum[1] as char)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkt_type_display_ok() {
        let pkt_type = PktType([0x33, 0x31, 0x34]);
        let res = format!(3; "{}", pkt_type).unwrap();
        assert_eq!(res, "314");
    }

    #[test]
    fn set_nmea_output_ok() {
        let mut data = SetNmeaOutput::default();
        data.gll = FrequencySetting::OnePositionFix;
        let df: DataField = data.into();
        assert_eq!(df.0, [0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0])
    }

    #[test]
    fn data_field_display_ok() {
        let mut data = SetNmeaOutput::default();
        data.gll = FrequencySetting::OnePositionFix;
        let df: DataField = data.into();
        let res = format!(38; "{}", df).unwrap();
        assert_eq!(res, ",1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0");
    }

    #[test]
    fn command_display_ok() {
        let mut data = SetNmeaOutput::default();
        data.gll = FrequencySetting::OnePositionFix;
        data.rmc = FrequencySetting::OnePositionFix;
        data.vtg = FrequencySetting::OnePositionFix;
        data.gga = FrequencySetting::OnePositionFix;
        data.gsa = FrequencySetting::OnePositionFix;
        data.gsv = FrequencySetting::FivePositionFix;
        let cmd: Command = data.into();
        let res = format!(51; "{}", cmd).unwrap();
        assert_eq!(res, "$PMTK314,1,1,1,1,1,5,0,0,0,0,0,0,0,0,0,0,0,0,0*2C\r\n");
    }
}