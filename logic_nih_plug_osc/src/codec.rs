//! Internal conversions between our types and `rosc`'s wire-format types.
//!
//! The boundary is deliberately kept narrow — only [`crate::sender::OscSender`]
//! and [`crate::receiver::OscReceiver`] actually touch rosc directly.
//! Everything else in the crate is portable and never depends on rosc's
//! exact shape.
//!
//! Some of these functions are only used by the sender and some only by
//! the receiver, so they end up "unused" under single-feature builds. We
//! silence the lint at the module level because the functions are all
//! legitimate part of the rosc bridge — it's just that each one only has
//! one caller.
#![allow(dead_code)]

use rosc::{OscArray, OscBundle, OscColor, OscMidiMessage, OscPacket, OscTime, OscType};

use crate::argument::{OSCArgument, OSCColour, OSCMidiMessage, OSCTimeTag};
use crate::bundle::OSCPacket as OurPacket;
use crate::error::OscError;
use crate::message::OSCMessage;

pub(crate) fn encode_packet(packet: &OurPacket) -> Result<Vec<u8>, OscError> {
    let rosc_packet = packet_to_rosc(packet);
    rosc::encoder::encode(&rosc_packet).map_err(|e| OscError::Encode(e.to_string()))
}

pub(crate) fn decode_packet(buf: &[u8]) -> Result<OurPacket, OscError> {
    let (rest, packet) = rosc::decoder::decode_udp(buf)
        .map_err(|e| OscError::Decode(e.to_string()))?;
    if !rest.is_empty() {
        // UDP-delivered OSC packets are atomic; trailing bytes are a
        // protocol violation.
        return Err(OscError::Decode(format!(
            "{} trailing bytes after OSC packet",
            rest.len()
        )));
    }
    Ok(packet_from_rosc(packet))
}

pub(crate) fn packet_to_rosc(packet: &OurPacket) -> OscPacket {
    match packet {
        OurPacket::Message(m) => OscPacket::Message(message_to_rosc(m)),
        OurPacket::Bundle(b) => OscPacket::Bundle(bundle_to_rosc(b)),
    }
}

pub(crate) fn packet_from_rosc(packet: OscPacket) -> OurPacket {
    match packet {
        OscPacket::Message(m) => OurPacket::Message(message_from_rosc(m)),
        OscPacket::Bundle(b) => OurPacket::Bundle(bundle_from_rosc(b)),
    }
}

pub(crate) fn message_to_rosc(msg: &OSCMessage) -> rosc::OscMessage {
    rosc::OscMessage {
        addr: msg.address.clone(),
        args: msg.args.iter().map(argument_to_rosc).collect(),
    }
}

pub(crate) fn message_from_rosc(msg: rosc::OscMessage) -> OSCMessage {
    OSCMessage {
        address: msg.addr,
        args: msg.args.into_iter().map(argument_from_rosc).collect(),
    }
}

pub(crate) fn bundle_to_rosc(bundle: &crate::bundle::OSCBundle) -> OscBundle {
    OscBundle {
        timetag: time_tag_to_rosc(bundle.time_tag),
        content: bundle.packets.iter().map(packet_to_rosc).collect(),
    }
}

pub(crate) fn bundle_from_rosc(bundle: OscBundle) -> crate::bundle::OSCBundle {
    crate::bundle::OSCBundle {
        time_tag: time_tag_from_rosc(bundle.timetag),
        packets: bundle.content.into_iter().map(packet_from_rosc).collect(),
    }
}

pub(crate) fn argument_to_rosc(arg: &OSCArgument) -> OscType {
    match arg {
        OSCArgument::Int32(v) => OscType::Int(*v),
        OSCArgument::Int64(v) => OscType::Long(*v),
        OSCArgument::Float32(v) => OscType::Float(*v),
        OSCArgument::Float64(v) => OscType::Double(*v),
        OSCArgument::String(s) => OscType::String(s.clone()),
        OSCArgument::Blob(b) => OscType::Blob(b.clone()),
        OSCArgument::TimeTag(t) => OscType::Time(time_tag_to_rosc(*t)),
        OSCArgument::Char(c) => OscType::Char(*c),
        OSCArgument::Colour(c) => OscType::Color(colour_to_rosc(*c)),
        OSCArgument::MidiMessage(m) => OscType::Midi(midi_to_rosc(*m)),
        OSCArgument::Bool(b) => OscType::Bool(*b),
        OSCArgument::Nil => OscType::Nil,
        OSCArgument::Inf => OscType::Inf,
        OSCArgument::Array(items) => OscType::Array(OscArray {
            content: items.iter().map(argument_to_rosc).collect(),
        }),
    }
}

pub(crate) fn argument_from_rosc(arg: OscType) -> OSCArgument {
    match arg {
        OscType::Int(v) => OSCArgument::Int32(v),
        OscType::Float(v) => OSCArgument::Float32(v),
        OscType::String(s) => OSCArgument::String(s),
        OscType::Blob(b) => OSCArgument::Blob(b),
        OscType::Time(t) => OSCArgument::TimeTag(time_tag_from_rosc(t)),
        OscType::Long(v) => OSCArgument::Int64(v),
        OscType::Double(v) => OSCArgument::Float64(v),
        OscType::Char(c) => OSCArgument::Char(c),
        OscType::Color(c) => OSCArgument::Colour(colour_from_rosc(c)),
        OscType::Midi(m) => OSCArgument::MidiMessage(midi_from_rosc(m)),
        OscType::Bool(b) => OSCArgument::Bool(b),
        OscType::Nil => OSCArgument::Nil,
        OscType::Inf => OSCArgument::Inf,
        OscType::Array(a) => OSCArgument::Array(
            a.content.into_iter().map(argument_from_rosc).collect(),
        ),
    }
}

pub(crate) fn time_tag_to_rosc(tag: OSCTimeTag) -> OscTime {
    OscTime {
        seconds: tag.seconds_since_1900,
        fractional: tag.fractional,
    }
}

pub(crate) fn time_tag_from_rosc(tag: OscTime) -> OSCTimeTag {
    OSCTimeTag {
        seconds_since_1900: tag.seconds,
        fractional: tag.fractional,
    }
}

pub(crate) fn colour_to_rosc(c: OSCColour) -> OscColor {
    OscColor {
        red: c.red,
        green: c.green,
        blue: c.blue,
        alpha: c.alpha,
    }
}

pub(crate) fn colour_from_rosc(c: OscColor) -> OSCColour {
    OSCColour {
        red: c.red,
        green: c.green,
        blue: c.blue,
        alpha: c.alpha,
    }
}

pub(crate) fn midi_to_rosc(m: OSCMidiMessage) -> OscMidiMessage {
    OscMidiMessage {
        port: m.port,
        status: m.status,
        data1: m.data1,
        data2: m.data2,
    }
}

pub(crate) fn midi_from_rosc(m: OscMidiMessage) -> OSCMidiMessage {
    OSCMidiMessage {
        port: m.port,
        status: m.status,
        data1: m.data1,
        data2: m.data2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::{OSCBundle, OSCPacket};

    fn round_trip(packet: &OSCPacket) -> OSCPacket {
        let encoded = encode_packet(packet).expect("encode");
        decode_packet(&encoded).expect("decode")
    }

    #[test]
    fn message_round_trips() {
        let msg = OSCMessage::new(
            "/amp",
            &[
                OSCArgument::Int32(1),
                OSCArgument::Float32(0.5),
                OSCArgument::String("hi".into()),
            ],
        );
        let packet = OSCPacket::Message(msg);
        let decoded = round_trip(&packet);
        assert_eq!(decoded, packet);
        assert_eq!(decoded.as_message().unwrap().address, "/amp");
    }

    #[test]
    fn bundle_round_trips() {
        let msg = OSCMessage::new("/x", &[OSCArgument::Int64(2)]);
        let inner = OSCBundle::immediate(vec![OSCMessage::new(
            "/y",
            &[OSCArgument::Bool(true)],
        )]);
        let packet = OSCPacket::Bundle(OSCBundle::immediate(vec![
            OSCPacket::Message(msg),
            OSCPacket::Bundle(inner),
        ]));
        let decoded = round_trip(&packet);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn array_argument_round_trips() {
        let msg = OSCMessage::new(
            "/mixer/levels",
            &[OSCArgument::Array(vec![
                OSCArgument::Int32(1),
                OSCArgument::Int32(2),
                OSCArgument::Int32(3),
            ])],
        );
        let packet = OSCPacket::Message(msg);
        let decoded = round_trip(&packet);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn colour_and_midi_round_trip() {
        let msg = OSCMessage::new(
            "/ui/led",
            &[
                OSCArgument::Colour(OSCColour::rgba(255, 0, 128, 255)),
                OSCArgument::MidiMessage(OSCMidiMessage::new(0, 0x90, 60, 127)),
            ],
        );
        let packet = OSCPacket::Message(msg);
        let decoded = round_trip(&packet);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn nil_and_inf_round_trip() {
        let msg = OSCMessage::new(
            "/special",
            &[OSCArgument::Nil, OSCArgument::Inf],
        );
        let packet = OSCPacket::Message(msg);
        let decoded = round_trip(&packet);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn decode_rejects_trailing_bytes() {
        let msg = OSCMessage::new("/x", &[OSCArgument::Int32(1)]);
        let mut bytes = encode_packet(&OSCPacket::Message(msg)).unwrap();
        bytes.extend_from_slice(b"junk");
        let err = decode_packet(&bytes).unwrap_err();
        assert!(matches!(err, OscError::Decode(_)));
    }
}
