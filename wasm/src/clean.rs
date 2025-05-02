use crate::missing_preserve::RemoveInvalidPreserveEntity;
use crate::mutate::MutatorList;
use tf_demo_parser::demo::message::usermessage::UserMessageType;
use tf_demo_parser::demo::message::Message;

/// General cleanup we always want to do
pub fn clean_demo(mutators: &mut MutatorList) {
    mutators.push_message_filter(|message: &Message| {
        if let Message::UserMessage(usr_message) = message {
            UserMessageType::CloseCaption != usr_message.message_type()
        } else {
            true
        }
    });
    mutators.push_message_mutator(RemoveInvalidPreserveEntity::new());
}
