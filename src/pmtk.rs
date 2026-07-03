use core::fmt::{Display, Formatter};
use heapless::{format, String};

const PACKET_LEN: usize = 255;

enum FrequencySetting {
    Disabled = 0x0,
    OnePositionFix = 0x1,
    TwoPositionFix = 0x2,
    ThreePositionFix = 0x3,
}

struct SetNmeaOutput {
    gll: FrequencySetting
    // TODO add remaining 18
}

impl Into<DataField> for SetNmeaOutput {
    fn into(self) -> DataField {
        DataField([self.gll as u8])
    }
}

struct DataField([u8; 1]); // TODO 1 becomes 19
impl Display for DataField {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for c in self.0 {
            write!(f, ",{}", c)?;
        }
        Ok(())
    }
}

struct PktType([u8; 3]);
impl Display for PktType {
    // "314" should be [0x33, 0x31, 0x34], or [51, 49, 52]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}{}", self.0[0] as char, self.0[1] as char, self.0[2] as char)
    }
}

// struct Command {
//     checksum: u8,
//     data_field: DataField,
//     pkt_type: PktType
// }
//
// impl Into<String<PACKET_LEN>> for Command {
//     fn into(self) -> String<PACKET_LEN> {
//         format!("$PMTK{}{}*<checksum>\r\n", self.pkt_type, self.data_field).unwrap()
//     }
// }

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
        let data = SetNmeaOutput { gll: FrequencySetting::OnePositionFix };
        let df: DataField = data.into();
        assert_eq!(df.0, [0x1])
    }

    #[test]
    fn data_field_display_ok() {
        let data = SetNmeaOutput { gll: FrequencySetting::OnePositionFix };
        let df: DataField = data.into();
        let res = format!(2; "{}", df).unwrap();
        assert_eq!(res, ",1");
    }
}