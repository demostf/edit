use std::mem::take;
use tf_demo_parser::demo::message::Message;
use tf_demo_parser::demo::packet::Packet;
use tf_demo_parser::ParserState;

pub trait PacketMutator {
    fn mutate_packet(&self, packet: &mut Packet, state: &ParserState);
}

pub trait MessageMutator {
    fn mutate_message(&self, message: &mut Message, state: &ParserState);
}

pub trait MessageFilter {
    fn filter(&self, message: &Message) -> bool;
}

pub struct PacketMessageMutator<T: MessageMutator> {
    pub mutator: T,
}

impl<T: MessageMutator> PacketMutator for PacketMessageMutator<T> {
    fn mutate_packet(&self, packet: &mut Packet, state: &ParserState) {
        match packet {
            Packet::Message(msg_packet) | Packet::Signon(msg_packet) => {
                msg_packet
                    .messages
                    .iter_mut()
                    .for_each(|msg| self.mutator.mutate_message(msg, state));
            },
            _ => {}
        }
    }
}

impl<F: Fn(&mut Packet)> PacketMutator for F {
    fn mutate_packet(&self, packet: &mut Packet, _state: &ParserState) {
        self(packet)
    }
}

impl<T: MessageMutator> From<T> for PacketMessageMutator<T> {
    fn from(mutator: T) -> Self {
        PacketMessageMutator { mutator }
    }
}

impl<F: Fn(&mut Message)> MessageMutator for F {
    fn mutate_message(&self, message: &mut Message, _state: &ParserState) {
        self(message)
    }
}

pub struct PacketMessageFilter<T: MessageFilter> {
    pub filter: T,
}

impl<T: MessageFilter> PacketMutator for PacketMessageFilter<T> {
    fn mutate_packet(&self, packet: &mut Packet, _state: &ParserState) {
        match packet {
            Packet::Message(msg_packet) | Packet::Signon(msg_packet) => {
                let messages = take(&mut msg_packet.messages);
                msg_packet.messages = messages
                    .into_iter()
                    .filter(|msg| self.filter.filter(msg))
                    .collect();
            }
            _ => {}
        }
    }
}

impl<T: MessageFilter> From<T> for PacketMessageFilter<T> {
    fn from(filter: T) -> Self {
        PacketMessageFilter { filter }
    }
}

impl<F: Fn(&Message) -> bool> MessageFilter for F {
    fn filter(&self, message: &Message) -> bool {
        self(message)
    }
}

#[derive(Default)]
pub struct MutatorList {
    mutators: Vec<Box<dyn PacketMutator>>,
}

impl MutatorList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_packet_mutator<M: PacketMutator + 'static>(&mut self, mutator: M) {
        self.mutators.push(Box::new(mutator))
    }

    pub fn push_message_mutator<M: MessageMutator + 'static>(&mut self, mutator: M) {
        self.mutators
            .push(Box::new(PacketMessageMutator::from(mutator)))
    }

    pub fn push_message_filter<M: MessageFilter + 'static>(&mut self, filter: M) {
        self.mutators
            .push(Box::new(PacketMessageFilter::from(filter)))
    }
}

impl PacketMutator for MutatorList {
    fn mutate_packet(&self, packet: &mut Packet, state: &ParserState) {
        for mutator in self.mutators.iter() {
            mutator.mutate_packet(packet, state);
        }
    }
}

