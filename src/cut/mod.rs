mod entity;
mod string_tables;

use bitbuffer::{BitRead, BitWrite, BitWriteStream, LittleEndian};
use std::cmp::{max, min};
use std::collections::BTreeSet;
use std::convert::TryInto;
use std::iter::once;
use std::mem::take;
use tf_demo_parser::demo::header::Header;
use tf_demo_parser::demo::message::packetentities::{EntityId, PacketEntitiesMessage, UpdateType};
use tf_demo_parser::demo::message::usermessage::{UserMessage, UserMessageType};
use tf_demo_parser::demo::message::{Message, NetTickMessage};
use tf_demo_parser::demo::packet::message::{MessagePacket, MessagePacketMeta};
use tf_demo_parser::demo::packet::stop::StopPacket;
use tf_demo_parser::demo::packet::{Packet, PacketType};
use tf_demo_parser::demo::parser::{DemoHandler, Encode, NullHandler, RawPacketStream};
use tf_demo_parser::{Demo, DemoParser, MessageType, ParserState};
use tf_demo_parser::demo::data::{DemoTick, ServerTick};
use wasm_bindgen::prelude::*;
use web_sys::console;
use crate::cut::entity::ActiveEntities;
use crate::cut::string_tables::StringTablesUpdates;
use crate::mutate::MessageMutator;
use crate::{EditOptions, find_stv, MutatorList, PacketMutator};

const PRESERVE_PACKETS: &[PacketType] = &[
    PacketType::Signon,
    PacketType::DataTables,
    PacketType::StringTables,
    PacketType::SyncTick,
];

pub fn cut(input: &[u8], options: EditOptions) -> Vec<u8> {
    let mut out_buffer = Vec::with_capacity(input.len());
    {
        let mut out_stream = BitWriteStream::new(&mut out_buffer, LittleEndian);

        let demo = Demo::new(&input);
        let spectator_id = find_stv(&demo).unwrap_or_else(|| EntityId::from(1u32));
        let mut stream = demo.get_stream();
        let mut header = Header::read(&mut stream).unwrap();

        let mut mutators = options.as_mutator(spectator_id);
        let start_tick = options.cut.unwrap().from;
        let end_tick = options.cut.unwrap().to;

        let start_tick = min(DemoTick::from(header.ticks - 10), start_tick);
        let end_tick = min(DemoTick::from(header.ticks), end_tick);
        let duration_per_tick = header.ticks as f32 / header.duration;

        header.ticks = (end_tick - start_tick).into();
        header.duration = header.ticks as f32 * duration_per_tick;
        header.write(&mut out_stream).unwrap();

        let mut packets = RawPacketStream::new(stream.clone());
        let mut start_handler = DemoHandler::default();
        start_handler.handle_header(&header);

        let mut handler = DemoHandler::default();
        handler.handle_header(&header);

        let start_state = skip_start(&mut start_handler, &mut packets, start_tick);

        for packet in start_state.start_packets {
            packet
                .encode(&mut out_stream, &handler.state_handler)
                .unwrap();
            handler.handle_packet(packet).unwrap();
        }
        let delta_tick = start_state.last_delta;

        let start_entities = start_state.entities.entity_ids();

        let string_table_updates = start_state
            .table_updates
            .encode()
            .into_iter()
            .map(|msg| Message::UpdateStringTable(msg));
        let (baseline_updates, entity_update, removed_update) =
            start_state
                .entities
                .encode(&start_handler.state_handler, delta_tick - 2, start_tick, &start_handler.state_handler);
        let baseline_updates = baseline_updates.into_iter().map(Message::PacketEntities);
        let start_packets = string_table_updates
            .chain(baseline_updates)
            .map(|msg| msg_packet(vec![net_tick(delta_tick - 2), msg]))
            .chain(once(msg_packet(vec![
                net_tick(delta_tick - 1),
                Message::PacketEntities(entity_update),
            ])))
            .chain(once(Packet::Message(MessagePacket {
                messages: vec![
                    net_tick(delta_tick),
                    Message::PacketEntities(removed_update),
                ],
                ..MessagePacket::default()
            })));
        for packet in start_packets {
            packet
                .encode(&mut out_stream, &handler.state_handler)
                .unwrap();
            handler.handle_packet(packet).unwrap();
        }

        // create the net ticks needed for later deltas
        let fill_ticks = (delta_tick + 1).range_inclusive(start_state.server_tick)
            .into_iter()
            .map(|tick| net_tick(tick));
        let fill_packets = fill_ticks.map(|msg| {
            Packet::Message(MessagePacket {
                messages: vec![
                    msg,
                    Message::PacketEntities(PacketEntitiesMessage {
                        max_entries: start_state.entity_max,
                        delta: Some((delta_tick - 1).try_into().unwrap()),
                        ..PacketEntitiesMessage::default()
                    }),
                ],
                ..MessagePacket::default()
            })
        });
        for packet in fill_packets {
            packet
                .encode(&mut out_stream, &handler.state_handler)
                .unwrap();
        }

        mutators.push_message_mutator(DeleteFilter::new(start_entities, start_state.server_tick));
        mutators.push_packet_mutator(move |packet: &mut Packet| {
            packet.set_tick(packet.tick() - start_tick)
        });

        while let Some(mut packet) = packets.next(&handler.state_handler).unwrap() {
            let original_tick = packet.tick();

            mutators.mutate_packet(&mut packet, &handler.state_handler);

            if packet.packet_type() != PacketType::ConsoleCmd {
                packet
                    .encode(&mut out_stream, &handler.state_handler)
                    .unwrap();
            }
            handler.handle_packet(packet).unwrap();

            if original_tick >= end_tick {
                break;
            }
        }
        PacketType::Stop.write(&mut out_stream).unwrap();
        StopPacket {
            tick: (end_tick - start_tick).into(),
        }
        .encode(&mut out_stream, &handler.state_handler)
        .unwrap();
    }
    out_buffer
}

struct StartState<'a> {
    entities: ActiveEntities,
    table_updates: StringTablesUpdates,
    start_packets: Vec<Packet<'a>>,
    server_tick: ServerTick,
    entity_max: u16,
    last_delta: ServerTick,
}

fn skip_start<'a>(
    handler: &mut DemoHandler<'a, NullHandler>,
    packets: &mut RawPacketStream<'a>,
    start_tick: DemoTick,
) -> StartState<'a> {
    let mut entities = ActiveEntities::default();
    let mut table_updates = StringTablesUpdates::default();
    let mut start_packets = Vec::with_capacity(6);
    let mut server_tick = ServerTick::default();
    let mut entity_max = 0;
    let mut last_delta = ServerTick::default();

    while let Some(packet) = packets.next(&handler.state_handler).unwrap() {
        if PRESERVE_PACKETS.contains(&packet.packet_type()) {
            start_packets.push(packet.clone());
            handler.handle_packet(packet).unwrap();
        } else if packet.packet_type() != PacketType::ConsoleCmd {
            if let Packet::Message(message_packet) = &packet {
                for msg in &message_packet.messages {
                    table_updates.handle_message(&msg);
                    match msg {
                        Message::PacketEntities(msg) => {
                            if let Some(delta) = msg.delta {
                                last_delta = delta;
                            }
                            entity_max = msg.max_entries;
                            entities.handle_message(msg, &handler.state_handler, packet.tick());
                        }
                        Message::NetTick(NetTickMessage { tick, .. }) => {
                            server_tick = *tick;
                        }
                        _ => {}
                    }
                }
            }
            let tick = packet.tick();
            handler.handle_packet(packet).unwrap();

            if tick >= start_tick {
                break;
            }
        }
    }

    StartState {
        entities,
        table_updates,
        start_packets,
        server_tick,
        entity_max,
        last_delta,
    }
}

struct DeleteFilter {
    current_entities: BTreeSet<EntityId>,
    till_delta: ServerTick,
}

impl DeleteFilter {
    pub fn new(current_entities: BTreeSet<EntityId>, till_delta: ServerTick) -> Self {
        DeleteFilter {
            current_entities,
            till_delta,
        }
    }
}

impl MessageMutator for DeleteFilter {
    fn mutate_message(&self, message: &mut Message, _state: &ParserState) {
        if let Message::PacketEntities(message) = message {
            if let Some(delta) = message.delta {
                if delta < self.till_delta {
                    let packet_entities = take(&mut message.entities);
                    message.entities = packet_entities
                        .into_iter()
                        .filter(|ent| match ent.update_type {
                            UpdateType::Delete | UpdateType::Leave => {
                                self.current_entities.contains(&ent.entity_index)
                            }
                            _ => true,
                        })
                        .collect();
                }
            }
        }
    }
}

fn msg_packet(messages: Vec<Message>) -> Packet {
    Packet::Message(MessagePacket {
        messages,
        ..MessagePacket::default()
    })
}

fn net_tick(tick: ServerTick) -> Message<'static> {
    Message::NetTick(NetTickMessage {
        tick,
        frame_time: 1881,
        std_dev: 263,
    })
}
