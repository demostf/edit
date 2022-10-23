mod mutate;
mod pov;
mod clean;
mod cond;
mod cut;
mod options;

use wasm_bindgen::prelude::*;
use tf_demo_parser::{Demo, DemoParser};
use tf_demo_parser::demo::header::Header;
use tf_demo_parser::demo::parser::{RawPacketStream, DemoHandler, Encode};
use tf_demo_parser::demo::packet::PacketType;
use bitbuffer::{BitRead, BitWriteStream, LittleEndian};
use tf_demo_parser::demo::message::packetentities::EntityId;

use bitbuffer::BitWrite;

use crate::clean::clean_demo;
use crate::cond::strip_cond;
use crate::cut::cut;
use crate::mutate::{MutatorList, PacketMutator};
pub use crate::options::{EditOptions, TickRange, CondOptions};
use crate::pov::unlock_pov;

extern crate web_sys;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn edit_js(input: &[u8], options: JsValue) -> Vec<u8> {
    set_panic_hook();
    let options: EditOptions = serde_wasm_bindgen::from_value(options).expect("invalid options");
    edit(input, options)
}

pub fn edit(input: &[u8], options: EditOptions) -> Vec<u8> {
    if options.cut.is_some() {
        cut(input, options)
    } else {
        no_cut(input, options)
    }
}

fn no_cut(input: &[u8], options: EditOptions) -> Vec<u8> {
    let mut out_buffer = Vec::with_capacity(input.len());
    {
        let mut out_stream = BitWriteStream::new(&mut out_buffer, LittleEndian);

        let demo = Demo::new(&input);
        let spectator_id = find_stv(&demo).unwrap_or_else(|| EntityId::from(1u32));

        let mut stream = demo.get_stream();
        let header = Header::read(&mut stream).unwrap();
        header.write(&mut out_stream).unwrap();

        let mut packets = RawPacketStream::new(stream.clone());
        let mut handler = DemoHandler::default();
        handler.handle_header(&header);

        let mutators = options.as_mutator(spectator_id);

        while let Some(mut packet) = packets.next(&handler.state_handler).unwrap() {
            mutators.mutate_packet(&mut packet, &handler.state_handler);

            if packet.packet_type() != PacketType::ConsoleCmd && packet.packet_type() != PacketType::UserCmd {
                packet
                    .encode(&mut out_stream, &handler.state_handler)
                    .unwrap();
            }
            handler.handle_packet(packet).unwrap();
        }
    }
    out_buffer
}

fn find_stv(demo: &Demo) -> Option<EntityId> {
    let parser = DemoParser::new(demo.get_stream());
    let (_, data) = parser.parse().expect("failed to parse demo");
    data.users.values().find(|user| user.steam_id == "BOT")
        .map(|user| user.entity_id)
}