use crate::{clean_demo, strip_cond, unlock_pov, MutatorList};
use serde::{Deserialize, Serialize};
use tf_demo_parser::demo::data::DemoTick;
use tf_demo_parser::demo::message::packetentities::EntityId;
use tf_demo_parser::demo::message::Message;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EditOptions {
    pub unlock_pov: bool,
    #[serde(default)]
    pub remove_conditions: Vec<CondOptions>,
    #[serde(default)]
    pub cut: Option<TickRange>,
}

impl EditOptions {
    pub fn as_mutator(&self, spectator_id: EntityId) -> MutatorList {
        let mut mutators = MutatorList::new();

        mutators.push_message_mutator(|message: &mut Message| {
            if let Message::ServerInfo(info) = message {
                info.server_name = format!("{} - Edited", info.server_name);
            }
        });

        clean_demo(&mut mutators);

        for cond_options in self.remove_conditions.iter() {
            let entity = if cond_options.entity > 0 {
                Some(EntityId::from(cond_options.entity))
            } else {
                None
            };
            strip_cond(&mut mutators, entity, cond_options.mask);
        }

        if self.unlock_pov {
            unlock_pov(&mut mutators, spectator_id);
        }

        mutators
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct CondOptions {
    entity: EntityId,
    mask: u32,
}

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
pub struct TickRange {
    pub from: DemoTick,
    pub to: DemoTick,
}
