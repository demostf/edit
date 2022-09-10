use std::cell::Cell;
use tf_demo_parser::demo::message::Message;
use tf_demo_parser::demo::message::packetentities::{EntityId, PacketEntity, UpdateType};
use tf_demo_parser::demo::message::usermessage::{UserMessage};
use tf_demo_parser::demo::packet::Packet;
use tf_demo_parser::demo::sendprop::{SendPropIdentifier, SendPropValue};
use tf_demo_parser::ParserState;
use crate::mutate::{MessageMutator, MutatorList};

struct AddStvEntity {
    added: Cell<bool>,
    entity_index: EntityId,
}

impl AddStvEntity {
    pub fn new(entity_index: EntityId) -> AddStvEntity {
        AddStvEntity {
            added: Cell::new(false),
            entity_index,
        }
    }
}

const TEAM_PROP: SendPropIdentifier = SendPropIdentifier::new("DT_BaseEntity", "m_iTeamNum");

impl MessageMutator for AddStvEntity {
    fn mutate_message(&self, message: &mut Message, state: &ParserState) {
        if !self.added.get() {
            if let Message::PacketEntities(ent_message) = message {
                if ent_message.base_line == 0 {
                    let player_entity = ent_message.entities.iter().find(|ent| ent.entity_index >= 1 && ent.entity_index < 255).expect("Failed to find a player entity");
                    if player_entity.entity_index == self.entity_index {
                        panic!("already an stv entity?");
                    }
                    let server_class = player_entity.server_class;

                    let mut team_prop = player_entity.get_prop_by_identifier(&TEAM_PROP, state).unwrap().clone();
                    team_prop.value = SendPropValue::Integer(1);

                    ent_message.entities.push(PacketEntity {
                        server_class,
                        entity_index: self.entity_index,
                        props: vec![team_prop],
                        in_pvs: false,
                        update_type: UpdateType::Enter,
                        serial_number: 1234567,
                        delay: None,
                        delta: None,
                        baseline_index: 0
                    });
                    ent_message.entities.sort_by(|a, b| a.entity_index.cmp(&b.entity_index));
                    self.added.set(true);
                }
            }
        }
    }
}

pub fn unlock_pov(mutators: &mut MutatorList, spectator_id: EntityId) {
    mutators.push_message_mutator(move |message: &mut Message| {
        if let Message::ServerInfo(info) = message {
            info.player_slot = u32::from(spectator_id) as u8 - 1;
        }
    });
    mutators.push_message_filter(|message: &Message| {
        !matches!(message, Message::SetView(_))
    });
    mutators.push_message_filter(|message: &Message| {
        !matches!(message, Message::UserMessage(UserMessage::VGuiMenu(_)))
    });
    mutators.push_message_mutator(|message: &mut Message| {
        if let Message::ServerInfo(info) = message {
            info.stv = true;
        }
    });
    mutators.push_packet_mutator(|packet: &mut Packet| {
        if let Packet::Message(message_packet) = packet {
            message_packet.meta.view_angles = Default::default();
        };
    });
    mutators.push_message_mutator(AddStvEntity::new(spectator_id));
}
