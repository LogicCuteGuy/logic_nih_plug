//! Registered Parameter Number (RPN) and Non-Registered Parameter Number
//! (NRPN) helpers.
//!
//! RPNs and NRPNs are how MIDI encodes parameters that aren't covered by
//! the standard CCs — pitch-bend sensitivity, master fine tuning, MPE
//! configuration, sound controller slots, vendor-specific parameters, etc.
//!
//! They're sent as **4 CC messages in a row**, in this order:
//!
//! 1. `CC 101` (Data Entry MSB) or `CC 99` for NRPN — the **MSB** of the
//!    parameter number.
//! 2. `CC 100` (Data Entry LSB) or `CC 98` — the **LSB** of the parameter
//!    number.
//! 3. `CC 6` (Data Entry) — the **MSB** of the value.
//! 4. `CC 38` (Data Entry LSB) — optionally the **LSB** of the value.
//!    (The 14-bit form is optional; many devices only support 7-bit
//!    values.)
//!
//! After sending the 4 CCs, hosts and devices conventionally send an RPN
//! *null* (`CC 101 = 127`, `CC 100 = 127`) to clear the RPN register so
//! that the next stray CC isn't accidentally interpreted as another
//! parameter.
//!
//! This module gives you:
//!
//! - [`MidiRpnKind`] — `RPN` vs `NRPN`.
//! - [`MidiRPN`] — a parameter number plus a value, with helpers to emit
//!   the four CC messages plus the null-reset CCs.

use crate::midi_message::MidiMessage;

/// Whether this is an RPN or an NRPN.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MidiRpnKind {
    /// Registered Parameter Number (`CC 101` / `CC 100`).
    RPN,
    /// Non-Registered Parameter Number (`CC 99` / `CC 98`).
    NRPN,
}

impl MidiRpnKind {
    /// The CC number for the parameter MSB.
    pub fn msb_cc(self) -> u8 {
        match self {
            MidiRpnKind::RPN => 101,
            MidiRpnKind::NRPN => 99,
        }
    }

    /// The CC number for the parameter LSB.
    pub fn lsb_cc(self) -> u8 {
        match self {
            MidiRpnKind::RPN => 100,
            MidiRpnKind::NRPN => 98,
        }
    }
}

/// A single RPN / NRPN message: the parameter number and the value to
/// transmit.
///
/// The parameter number is encoded as two 7-bit values (`MSB << 7 | LSB`,
/// so a 14-bit number in `0..=16383`). The value defaults to a 7-bit
/// (`0..128`) value, but the `*_14bit` constructors accept a 14-bit
/// value that produces both the MSB and LSB CCs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MidiRPN {
    /// RPN vs NRPN.
    pub kind: MidiRpnKind,
    /// MIDI channel (`0..16`).
    pub channel: u8,
    /// The 14-bit parameter number (`0..=16383`).
    pub parameter: u16,
    /// Whether the value is 7-bit or 14-bit. Encoded only at the call
    /// site; the struct itself stores the value as a 14-bit integer.
    pub value: u16,
    /// If `true`, use the 14-bit value form (also emit CC 38). If `false`,
    /// only emit CC 6.
    pub is_14bit: bool,
}

impl MidiRPN {
    /// Construct a 7-bit RPN or NRPN message.
    ///
    /// # Panics
    ///
    /// Panics if `channel >= 16`, `parameter > 16383`, or `value >= 128`.
    pub fn new_7bit(kind: MidiRpnKind, channel: u8, parameter: u16, value: u8) -> Self {
        assert!(channel < 16, "channel must be in 0..16");
        assert!(parameter <= 16383, "parameter must be 0..=16383");
        assert!(value < 128, "7-bit value must be in 0..128");
        Self {
            kind,
            channel,
            parameter,
            value: value as u16,
            is_14bit: false,
        }
    }

    /// Construct a 14-bit RPN or NRPN message.
    ///
    /// # Panics
    ///
    /// Panics if `channel >= 16`, `parameter > 16383`, or `value > 16383`.
    pub fn new_14bit(kind: MidiRpnKind, channel: u8, parameter: u16, value: u16) -> Self {
        assert!(channel < 16, "channel must be in 0..16");
        assert!(parameter <= 16383, "parameter must be 0..=16383");
        assert!(value <= 16383, "14-bit value must be 0..=16383");
        Self {
            kind,
            channel,
            parameter,
            value,
            is_14bit: true,
        }
    }

    /// The MSB of the parameter number (the top 7 bits of the 14-bit
    /// parameter number).
    pub fn parameter_msb(&self) -> u8 {
        ((self.parameter >> 7) & 0x7F) as u8
    }

    /// The LSB of the parameter number (the bottom 7 bits).
    pub fn parameter_lsb(&self) -> u8 {
        (self.parameter & 0x7F) as u8
    }

    /// The MSB of the value.
    pub fn value_msb(&self) -> u8 {
        ((self.value >> 7) & 0x7F) as u8
    }

    /// The LSB of the value.
    pub fn value_lsb(&self) -> u8 {
        (self.value & 0x7F) as u8
    }

    /// Build the four CC messages that constitute this RPN/NRPN.
    ///
    /// The order is: parameter MSB CC, parameter LSB CC, value MSB CC,
    /// value LSB CC (the last is only emitted for the 14-bit form).
    ///
    /// For the 7-bit form, the entire value lives in the value MSB CC
    /// (CC 6) and the value LSB CC (CC 38) is omitted.
    pub fn to_messages(&self) -> Vec<MidiMessage> {
        let mut out = Vec::with_capacity(if self.is_14bit { 4 } else { 3 });
        out.push(MidiMessage::controller(
            self.channel,
            self.kind.msb_cc(),
            self.parameter_msb(),
        ));
        out.push(MidiMessage::controller(
            self.channel,
            self.kind.lsb_cc(),
            self.parameter_lsb(),
        ));
        // For 7-bit values, the whole value goes in the MSB CC.
        let value_msb = if self.is_14bit {
            self.value_msb()
        } else {
            (self.value & 0x7F) as u8
        };
        out.push(MidiMessage::controller(self.channel, 6, value_msb));
        if self.is_14bit {
            out.push(MidiMessage::controller(self.channel, 38, self.value_lsb()));
        }
        out
    }

    /// Build the four CC messages for this RPN/NRPN **plus** the
    /// null-parameter CC pair (`CC 101 = 127`, `CC 100 = 127` for RPN;
    /// `CC 99 = 127`, `CC 98 = 127` for NRPN) used to reset the device's
    /// parameter register so that the next stray CC isn't misinterpreted.
    pub fn to_messages_with_null(&self) -> Vec<MidiMessage> {
        let mut out = self.to_messages();
        out.push(MidiMessage::controller(
            self.channel,
            self.kind.msb_cc(),
            127,
        ));
        out.push(MidiMessage::controller(
            self.channel,
            self.kind.lsb_cc(),
            127,
        ));
        out
    }
}

/// Standard, well-known RPN parameter numbers.
pub mod standard_rpn {
    /// Pitch-bend sensitivity (in semitones, MSB only — LSB adds
    /// fractional semitones).
    #[allow(dead_code)]
    pub const PITCH_BEND_SENSITIVITY: u16 = 0x0000;
    /// Channel fine tuning (`±100 cents`, MSB only).
    #[allow(dead_code)]
    pub const CHANNEL_FINE_TUNING: u16 = 0x0001;
    /// Channel coarse tuning (`±24 semitones`, MSB only).
    #[allow(dead_code)]
    pub const CHANNEL_COARSE_TUNING: u16 = 0x0002;
    /// Tuning program change (selects a tuning table; value is the
    /// program number).
    #[allow(dead_code)]
    pub const TUNING_PROGRAM: u16 = 0x0003;
    /// Tuning bank select (selects a tuning bank).
    #[allow(dead_code)]
    pub const TUNING_BANK: u16 = 0x0004;
    /// Modulation depth range (in 1/600 of a semitone, RPN 0x0005 — GM2).
    #[allow(dead_code)]
    pub const MOD_DEPTH_RANGE: u16 = 0x0005;
    /// MPE Configuration Message (RPN 0x0006) — sends the number of
    /// member channels in the MSB and the Manager Channel in the LSB.
    #[allow(dead_code)]
    pub const MPE_CONFIGURATION: u16 = 0x0006;
    /// MPE Pitch Bend Sensitivity (RPN 0x0007).
    #[allow(dead_code)]
    pub const MPE_PITCH_BEND_SENSITIVITY: u16 = 0x0007;
    /// Null parameter (used to clear the RPN register).
    #[allow(dead_code)]
    pub const RPN_NULL: u16 = 0x7F7F;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpn_kind_cc_numbers() {
        assert_eq!(MidiRpnKind::RPN.msb_cc(), 101);
        assert_eq!(MidiRpnKind::RPN.lsb_cc(), 100);
        assert_eq!(MidiRpnKind::NRPN.msb_cc(), 99);
        assert_eq!(MidiRpnKind::NRPN.lsb_cc(), 98);
    }

    #[test]
    fn seven_bit_message_sequence() {
        let rpn = MidiRPN::new_7bit(MidiRpnKind::RPN, 0, 0x0000, 12);
        let msgs = rpn.to_messages();
        assert_eq!(msgs.len(), 3);
        // Parameter MSB = 0, LSB = 0; value MSB = 12.
        assert_eq!(msgs[0].to_bytes(), &[0xB0, 101, 0]);
        assert_eq!(msgs[1].to_bytes(), &[0xB0, 100, 0]);
        assert_eq!(msgs[2].to_bytes(), &[0xB0, 6, 12]);
    }

    #[test]
    fn fourteen_bit_message_sequence() {
        let rpn = MidiRPN::new_14bit(MidiRpnKind::NRPN, 3, 0x0123, 0x2567);
        let msgs = rpn.to_messages();
        assert_eq!(msgs.len(), 4);
        // Channel 3 → 0xB3.
        // Parameter 0x0123 = 291; MSB = 2, LSB = 0x23.
        assert_eq!(msgs[0].to_bytes(), &[0xB3, 99, 0x02]); // MSB of parameter
        assert_eq!(msgs[1].to_bytes(), &[0xB3, 98, 0x23]); // LSB of parameter
        // Value 0x2567 = 9575; MSB = 0x4A (9575 / 128), LSB = 0x67 (9575 % 128).
        assert_eq!(msgs[2].to_bytes(), &[0xB3, 6, 0x4A]);  // MSB of value
        assert_eq!(msgs[3].to_bytes(), &[0xB3, 38, 0x67]); // LSB of value
    }

    #[test]
    fn with_null_appends_reset_pair() {
        let rpn = MidiRPN::new_7bit(MidiRpnKind::RPN, 0, 0, 12);
        let msgs = rpn.to_messages_with_null();
        assert_eq!(msgs.len(), 5);
        // Last two messages are CC 101 = 127, CC 100 = 127.
        assert_eq!(msgs[3].to_bytes(), &[0xB0, 101, 127]);
        assert_eq!(msgs[4].to_bytes(), &[0xB0, 100, 127]);

        let nrpn = MidiRPN::new_7bit(MidiRpnKind::NRPN, 0, 0, 12);
        let msgs = nrpn.to_messages_with_null();
        assert_eq!(msgs[3].to_bytes(), &[0xB0, 99, 127]);
        assert_eq!(msgs[4].to_bytes(), &[0xB0, 98, 127]);
    }

    #[test]
    fn known_rpn_constants() {
        assert_eq!(standard_rpn::PITCH_BEND_SENSITIVITY, 0x0000);
        assert_eq!(standard_rpn::CHANNEL_FINE_TUNING, 0x0001);
        assert_eq!(standard_rpn::CHANNEL_COARSE_TUNING, 0x0002);
        assert_eq!(standard_rpn::MPE_CONFIGURATION, 0x0006);
        assert_eq!(standard_rpn::MPE_PITCH_BEND_SENSITIVITY, 0x0007);
        assert_eq!(standard_rpn::RPN_NULL, 0x7F7F);
    }

    #[test]
    #[should_panic(expected = "7-bit value must be in 0..128")]
    fn seven_bit_overflow_panics() {
        let _ = MidiRPN::new_7bit(MidiRpnKind::RPN, 0, 0, 128);
    }

    #[test]
    fn parameter_msb_lsb_split() {
        let rpn = MidiRPN::new_14bit(MidiRpnKind::RPN, 0, 0x1234, 0);
        assert_eq!(rpn.parameter_msb(), 0x24);
        assert_eq!(rpn.parameter_lsb(), 0x34);
    }
}
