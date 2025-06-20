use crate::attr::{Attrs, Nl80211Attr, Nl80211RateInfo, Nl80211StaInfo};

use neli::attr::Attribute;
use neli::err::DeError;

/// A struct representing a remote station (Access Point)
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Station {
    /// Signal strength average (dBm)
    pub average_signal: Option<i8>,
    /// Count of times beacon loss was detected
    pub beacon_loss: Option<u32>,
    /// Station bssid (u8)
    pub bssid: Option<Vec<u8>>,
    /// Time since the station is last connected in seconds
    pub connected_time: Option<u32>,
    /// Reception bitrate
    pub rx_bitrate: Option<u32>,
    /// Total received packets (MSDUs and MMPDUs) from this station
    pub rx_packets: Option<u32>,
    /// Signal strength of last received PPDU (dBm)
    pub signal: Option<i8>,
    /// Transmission bitrate
    pub tx_bitrate: Option<u32>,
    /// Total failed packets (MPDUs) to this station
    pub tx_failed: Option<u32>,
    /// Total transmitted packets (MSDUs and MMPDUs) to this station
    pub tx_packets: Option<u32>,
    /// Total retries (MPDUs) to this station
    pub tx_retries: Option<u32>,
    /// High Throughput Modulation and Coding Scheme (HT-MCS) (802.11n)
    pub ht_mcs: Option<u8>,
    /// Very High Throughput Modulation and Coding Scheme (VHT-MCS) (802.11ac)
    pub vht_mcs: Option<u8>,
    /// High Efficiency Modulation and Coding Scheme. (HE-MCS) (802.11ax)
    pub he_mcs: Option<u8>,
    /// Extremely High Throughput-Modulation and Coding Scheme (EHT-MCS) (802.11be)
    pub eht_mcs: Option<u8>,
}

impl TryFrom<Attrs<'_, Nl80211Attr>> for Station {
    type Error = DeError;

    fn try_from(attrs: Attrs<'_, Nl80211Attr>) -> Result<Self, Self::Error> {
        let mut res = Self::default();
        if let Some(bssid) = attrs.get_attribute(Nl80211Attr::AttrMac) {
            res.bssid = Some(Vec::from(bssid.nla_payload.as_ref()));
        }

        if let Some(info) = attrs.get_attribute(Nl80211Attr::AttrStaInfo) {
            let attrs = info.get_attr_handle::<Nl80211StaInfo>().unwrap();
            for attr in attrs.iter() {
                match attr.nla_type.nla_type {
                    Nl80211StaInfo::StaInfoSignal => res.signal = Some(attr.get_payload_as()?),
                    Nl80211StaInfo::StaInfoSignalAvg => {
                        res.average_signal = Some(attr.get_payload_as()?)
                    }
                    Nl80211StaInfo::StaInfoBeaconLoss => {
                        res.beacon_loss = Some(attr.get_payload_as()?)
                    }
                    Nl80211StaInfo::StaInfoConnectedTime => {
                        res.connected_time = Some(attr.get_payload_as()?)
                    }
                    Nl80211StaInfo::StaInfoRxPackets => {
                        res.rx_packets = Some(attr.get_payload_as()?)
                    }
                    Nl80211StaInfo::StaInfoTxPackets => {
                        res.tx_packets = Some(attr.get_payload_as()?)
                    }
                    Nl80211StaInfo::StaInfoTxRetries => {
                        res.tx_retries = Some(attr.get_payload_as()?)
                    }
                    Nl80211StaInfo::StaInfoTxFailed => res.tx_failed = Some(attr.get_payload_as()?),
                    Nl80211StaInfo::StaInfoRxBitrate => {
                        if let Some(rate) = attr
                            .get_attr_handle::<Nl80211RateInfo>()?
                            .get_attribute(Nl80211RateInfo::RateInfoBitrate32)
                        {
                            res.rx_bitrate = Some(rate.get_payload_as()?);
                        }
                    }
                    Nl80211StaInfo::StaInfoTxBitrate => {
                        let rate_info = attr.get_attr_handle::<Nl80211RateInfo>()?;
                        if let Some(rate) =
                            rate_info.get_attribute(Nl80211RateInfo::RateInfoBitrate32)
                        {
                            res.tx_bitrate = Some(rate.get_payload_as()?);
                        }

                        if let Some(ht_mcs) = rate_info.get_attribute(Nl80211RateInfo::RateInfoMcs)
                        {
                            res.ht_mcs = ht_mcs.get_payload_as().ok();
                        }
                        if let Some(vht_mcs) =
                            rate_info.get_attribute(Nl80211RateInfo::RateInfoVhtMcs)
                        {
                            res.vht_mcs = vht_mcs.get_payload_as().ok();
                        }
                        if let Some(he_mcs) =
                            rate_info.get_attribute(Nl80211RateInfo::RateInfoHeMcs)
                        {
                            res.he_mcs = he_mcs.get_payload_as().ok();
                        }
                        if let Some(eht_mcs) =
                            rate_info.get_attribute(Nl80211RateInfo::RateInfoEhtMcs)
                        {
                            res.eht_mcs = eht_mcs.get_payload_as().ok();
                        }
                    }
                    _ => (),
                }
            }
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests_station {
    use super::*;
    use crate::attr::Nl80211Attr::AttrMac;
    use crate::attr::Nl80211Attr::AttrStaInfo;
    use neli::attr::AttrHandle;
    use neli::genl::{AttrType, Nlattr};
    use neli::types::Buffer;

    fn new_attr(t: Nl80211Attr, d: Vec<u8>) -> Nlattr<Nl80211Attr, Buffer> {
        Nlattr {
            nla_len: (4 + d.len()) as _,
            nla_type: AttrType {
                nla_nested: false,
                nla_network_order: true,
                nla_type: t,
            },
            nla_payload: d.into(),
        }
    }

    #[test]
    fn test_parser() {
        let handler = vec![
            new_attr(AttrMac, vec![46, 46, 46, 46, 46, 46]),
            new_attr(
                AttrStaInfo,
                vec![
                    8, 0, 16, 0, 17, 27, 0, 0, 8, 0, 1, 0, 248, 2, 0, 0, 8, 0, 2, 0, 43, 98, 156,
                    29, 8, 0, 3, 0, 99, 123, 109, 1, 12, 0, 23, 0, 43, 98, 156, 29, 0, 0, 0, 0, 12,
                    0, 24, 0, 99, 123, 109, 1, 0, 0, 0, 0, 5, 0, 7, 0, 218, 0, 0, 0, 5, 0, 13, 0,
                    215, 0, 0, 0, 20, 0, 25, 0, 5, 0, 0, 0, 216, 0, 0, 0, 5, 0, 1, 0, 213, 0, 0, 0,
                    20, 0, 26, 0, 5, 0, 0, 0, 212, 0, 0, 0, 5, 0, 1, 0, 211, 0, 0, 0, 28, 0, 8, 0,
                    8, 0, 5, 0, 16, 4, 0, 0, 6, 0, 1, 0, 16, 4, 0, 0, 5, 0, 2, 0, 13, 0, 0, 0, 28,
                    0, 14, 0, 8, 0, 5, 0, 134, 1, 0, 0, 6, 0, 1, 0, 134, 1, 0, 0, 5, 0, 2, 0, 4, 0,
                    0, 0, 8, 0, 9, 0, 226, 128, 7, 0, 8, 0, 10, 0, 9, 170, 2, 0, 8, 0, 11, 0, 27,
                    130, 0, 0, 8, 0, 12, 0, 47, 0, 0, 0, 8, 0, 27, 0, 196, 160, 0, 0, 8, 0, 18, 0,
                    0, 0, 0, 0, 28, 0, 15, 0, 4, 0, 2, 0, 4, 0, 3, 0, 5, 0, 4, 0, 1, 0, 0, 0, 6, 0,
                    5, 0, 100, 0, 0, 0, 12, 0, 17, 0, 254, 0, 0, 0, 170, 0, 0, 0, 12, 0, 28, 0,
                    183, 3, 0, 0, 0, 0, 0, 0, 12, 0, 29, 0, 225, 254, 0, 0, 0, 0, 0, 0, 5, 0, 30,
                    0, 216, 0, 0, 0, 5, 0, 34, 0, 46, 0, 0, 0, 56, 8, 31, 0, 128, 0, 1, 0, 12, 0,
                    1, 0, 168, 103, 5, 0, 0, 0, 0, 0, 12, 0, 2, 0, 71, 169, 2, 0, 0, 0, 0, 0, 12,
                    0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 76, 0, 6,
                    0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 61, 39, 1, 0, 8,
                    0, 4, 0, 23, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0, 0, 8, 0, 8,
                    0, 0, 0, 0, 0, 8, 0, 9, 0, 38, 56, 109, 1, 8, 0, 10, 0, 71, 169, 2, 0, 128, 0,
                    2, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 76,
                    0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0, 0, 0,
                    0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0, 0, 8,
                    0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0, 128, 0,
                    3, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 76,
                    0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0, 0, 0,
                    0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0, 0, 8,
                    0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0, 128, 0,
                    4, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 76,
                    0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0, 0, 0,
                    0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0, 0, 8,
                    0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0, 128, 0,
                    5, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 76,
                    0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0, 0, 0,
                    0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0, 0, 8,
                    0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0, 128, 0,
                    6, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 76,
                    0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0, 0, 0,
                    0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0, 0, 8,
                    0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0, 128, 0,
                    7, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 180, 0, 0, 0, 0, 0, 0,
                    0, 12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    76, 0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 180,
                    0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0,
                    0, 8, 0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 115, 64, 0, 0, 8, 0, 10, 0, 180, 0, 0,
                    0, 128, 0, 8, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 2, 0, 0, 0,
                    0, 0, 0, 0, 12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 76, 0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0,
                    2, 0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0,
                    0, 0, 8, 0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 32, 1, 0, 0, 8, 0, 10, 0, 2, 0, 0, 0,
                    128, 0, 9, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 76, 0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0,
                    0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0,
                    0, 8, 0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0,
                    128, 0, 10, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 76, 0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0,
                    0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0,
                    0, 8, 0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0,
                    128, 0, 11, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 76, 0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0,
                    0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0,
                    0, 8, 0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0,
                    128, 0, 12, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 76, 0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0,
                    0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0,
                    0, 8, 0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0,
                    128, 0, 13, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 76, 0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0,
                    0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0,
                    0, 8, 0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0,
                    128, 0, 14, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 76, 0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0,
                    0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0,
                    0, 8, 0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0,
                    128, 0, 15, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 76, 0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0,
                    0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0,
                    0, 8, 0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0,
                    128, 0, 16, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 76, 0, 6, 0, 8, 0, 1, 0, 0, 0, 0, 0, 8, 0, 2, 0, 0, 0, 0, 0, 8, 0, 3, 0, 0,
                    0, 0, 0, 8, 0, 4, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 8, 0, 6, 0, 0, 0, 0,
                    0, 8, 0, 8, 0, 0, 0, 0, 0, 8, 0, 9, 0, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0, 0, 0, 52,
                    0, 17, 0, 12, 0, 1, 0, 109, 25, 0, 0, 0, 0, 0, 0, 12, 0, 2, 0, 4, 0, 0, 0, 0,
                    0, 0, 0, 12, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0,
                    0,
                ],
            ),
        ];

        let station: Station = AttrHandle::new(handler.into_iter().collect())
            .try_into()
            .unwrap();
        let expected_station = Station {
            average_signal: Some(i8::from_le_bytes([215])),
            beacon_loss: Some(u32::from_le_bytes([0, 0, 0, 0])),
            bssid: Some(vec![46, 46, 46, 46, 46, 46]),
            connected_time: Some(u32::from_le_bytes([17, 27, 0, 0])),
            rx_bitrate: Some(u32::from_le_bytes([134, 1, 0, 0])),
            rx_packets: Some(u32::from_le_bytes([226, 128, 7, 0])),
            signal: Some(i8::from_le_bytes([218])),
            tx_bitrate: Some(u32::from_le_bytes([16, 4, 0, 0])),
            tx_failed: Some(u32::from_le_bytes([47, 0, 0, 0])),
            tx_packets: Some(u32::from_le_bytes([9, 170, 2, 0])),
            tx_retries: Some(u32::from_le_bytes([27, 130, 0, 0])),
            ht_mcs: Some(13),
            vht_mcs: None,
            he_mcs: None,
            eht_mcs: None,
        };

        assert_eq!(station, expected_station)
    }
}
