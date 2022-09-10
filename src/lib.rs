mod mutate;
mod pov;
mod clean;
mod cond;

use wasm_bindgen::prelude::*;
use tf_demo_parser::{Demo, DemoParser};
use tf_demo_parser::demo::header::Header;
use tf_demo_parser::demo::parser::{RawPacketStream, DemoHandler, Encode};
use tf_demo_parser::demo::packet::PacketType;
use bitbuffer::{BitRead, BitWriteStream, LittleEndian};
use tf_demo_parser::demo::message::packetentities::EntityId;
use serde::{Serialize, Deserialize};
use bitbuffer::BitWrite;
use crate::clean::clean_demo;
use crate::cond::strip_cond;
use crate::mutate::{MutatorList, PacketMutator};
use crate::pov::unlock_pov;

extern crate web_sys;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EditOptions {
    pub unlock_pov: bool,
    pub remove_conditions: Vec<CondOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CondOptions {
    entity: u32,
    mask: u32,
}

#[wasm_bindgen]
pub fn edit(input: &[u8], options: JsValue) -> Vec<u8> {
    set_panic_hook();
    let options: EditOptions = options.into_serde().expect("invalid options");
    rust_edit(input, options)
}

pub fn rust_edit(input: &[u8], options: EditOptions) -> Vec<u8> {
    let mut out_buffer = Vec::with_capacity(input.len());
    {
        let mut out_stream = BitWriteStream::new(&mut out_buffer, LittleEndian);

        let demo = Demo::new(&input);
        let spectator_id = find_stv(&demo).expect("no stv bot found");
        dbg!(spectator_id);

        let mut stream = demo.get_stream();
        let header = Header::read(&mut stream).unwrap();
        header.write(&mut out_stream).unwrap();

        let mut packets = RawPacketStream::new(stream.clone());
        let mut handler = DemoHandler::default();
        handler.handle_header(&header);

        let mut mutators = MutatorList::new();
        clean_demo(&mut mutators);

        for cond_options in options.remove_conditions {
            let entity = if cond_options.entity > 0 {
                Some(EntityId::from(cond_options.entity))
            } else {
                None
            };
            strip_cond(&mut mutators, entity, cond_options.mask);
        }

        if options.unlock_pov {
            unlock_pov(&mut mutators, spectator_id);
        }


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