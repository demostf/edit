use tf_demo_parser::demo::message::Message;
use tf_demo_parser::demo::message::packetentities::{EntityId, PacketEntity};
use tf_demo_parser::demo::sendprop::{SendPropIdentifier, SendPropValue};
use tf_demo_parser::ParserState;
use crate::mutate::MessageMutator;
use crate::MutatorList;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Cond {
    Uber = 5,
    UberDecay = 8,
    Quickfix = 28,
    Kritz = 11,
    Jarate = 24,
    Bleed = 25,
}

pub struct CondMask {
    cond: i64,
    entity: Option<EntityId>,
}

#[allow(dead_code)]
impl CondMask {
    pub fn new(entity: Option<EntityId>) -> Self {
        CondMask {
            cond: i64::MAX,
            entity,
        }
    }

    pub fn remove_cond(&mut self, cond: Cond) {
        self.cond &= !(1 << cond as u8);
    }
}

const PROP_ID: SendPropIdentifier = SendPropIdentifier::new("DT_TFPlayerShared", "m_nPlayerCond");

impl CondMask {
    fn mutate_entity(&self, entity: &mut PacketEntity) {
        if Some(entity.entity_index) == self.entity || self.entity.is_none() {
            entity
                .props
                .iter_mut()
                .filter(|prop| prop.identifier == PROP_ID)
                .for_each(|prop| {
                    if let SendPropValue::Integer(value) = &mut prop.value {
                        *value &= self.cond;
                    }
                })
        }
    }
}

impl MessageMutator for CondMask {
    fn mutate_message(&self, message: &mut Message, _state: &ParserState) {
        if let Message::PacketEntities(entity_message) = message {
            entity_message
                .entities
                .iter_mut()
                .for_each(|ent| self.mutate_entity(ent))
        }
    }
}

pub fn strip_cond(mutators: &mut MutatorList, entity: Option<EntityId>, mask: u32) {
    mutators.push_message_mutator(CondMask {
        entity,
        cond: mask as i64,
    });
}