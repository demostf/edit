use crate::mutate::MessageMutator;
use log::{info, warn};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use tf_demo_parser::demo::message::packetentities::{EntityId, PacketEntity, UpdateType};
use tf_demo_parser::demo::message::Message;
use tf_demo_parser::demo::packet::datatable::{ClassId, ServerClass};
use tf_demo_parser::ParserState;

#[derive(Default)]
pub struct RemoveInvalidPreserveEntity {
    known_entities: RefCell<BTreeSet<EntityId>>,
    deferred_delete: RefCell<Vec<EntityId>>,
}

impl RemoveInvalidPreserveEntity {
    pub fn new() -> RemoveInvalidPreserveEntity {
        RemoveInvalidPreserveEntity::default()
    }
}

impl MessageMutator for RemoveInvalidPreserveEntity {
    fn mutate_message(&self, message: &mut Message, _state: &ParserState) {
        if let Message::PacketEntities(ent_message) = message {
            let deferred_deletes = self.deferred_delete.take();
            for entity in ent_message.entities.iter() {
                match entity.update_type {
                    UpdateType::Enter => {
                        self.known_entities.borrow_mut().insert(entity.entity_index);
                    }
                    UpdateType::Preserve => {
                        if !self.known_entities.borrow().contains(&entity.entity_index) {
                            warn!("preserving missing entity {}", entity.entity_index);
                        }
                    }
                    UpdateType::Delete => {
                        self.known_entities.borrow_mut().remove(&entity.entity_index);
                    },
                    _ => {}
                };
            }
            ent_message.removed_entities.retain(|id| {
                if self.known_entities.borrow().contains(&id) {
                    // just not deleting makes the demo play, but with some ERROR entities
                    // having a Delete or Leave makes it crash further in the demo

                    // warn!("inserting delete for {}", id);
                    // only entity_index and update_type is used
                    // ent_message.entities.push(PacketEntity {
                    //     entity_index: *id,
                    //     server_class: 0.into(),
                    //     props: vec![],
                    //     in_pvs: false,
                    //     update_type: UpdateType::Leave,
                    //     serial_number: 0,
                    //     delay: None,
                    //     delta: None,
                    //     baseline_index: 0,
                    // });
                    // self.deferred_delete.borrow_mut().push(*id);
                    false
                } else {
                    true
                }
            });
            ent_message.entities.sort_by(|a, b| a.entity_index.cmp(&b.entity_index));
            ent_message.removed_entities.extend(deferred_deletes);
            ent_message.removed_entities.sort();
        }
    }
}
