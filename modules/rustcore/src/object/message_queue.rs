//! `core/object/message_queue.*`: deferred calls (state + dispatch order).
//!
//! The engine stores deferred calls/sets/notifications in flat pages and flushes
//! them at the end of the frame. This port keeps the same message kinds and
//! push/flush semantics in a `VecDeque`:
//!
//! - `push_call` / `push_callable` (method or callable, arguments, optional
//!   error reporting) require at least one target;
//! - `push_set` (object id + property + value) and `push_notification`
//!   (object id + notification) reject an invalid target;
//! - `flush` pops messages in order and hands each to the caller's dispatcher,
//!   recording the peak queue size; messages pushed *during* a flush are queued
//!   for the next flush (like the engine's page boundary);
//! - `clear`, `has_messages`, `is_flushing`, `get_max_buffer_usage`.
//!
//! The actual `Object::call`/`set`/notification delivery and the statistics dump
//! are not ported: the dispatcher closure receives the message instead.

use std::collections::VecDeque;

use crate::object::ObjectId;
use crate::string::string_name::StringName;
use crate::variant::{Callable, Variant};

/// One deferred message.
#[derive(Clone, Debug)]
pub enum Message {
    /// `TYPE_CALL` with a method name.
    Call {
        object: ObjectId,
        method: StringName,
        args: Vec<Variant>,
        show_error: bool,
    },
    /// `TYPE_CALL` with a callable target.
    Callable {
        callable: Callable,
        args: Vec<Variant>,
        show_error: bool,
    },
    /// `TYPE_SET`.
    Set {
        object: ObjectId,
        property: StringName,
        value: Variant,
    },
    /// `TYPE_NOTIFICATION`.
    Notification { object: ObjectId, notification: i32 },
}

#[derive(Clone, Debug, Default)]
pub struct MessageQueueData {
    messages: VecDeque<Message>,
    flushing: bool,
    max_buffer_usage: usize,
}

impl MessageQueueData {
    pub fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, message: Message) -> usize {
        if self.flushing {
            // The engine appends into the still-open page; the port queues it for
            // the next flush instead.
            self.messages.push_back(message);
            return self.messages.len();
        }
        self.messages.push_back(message);
        self.max_buffer_usage = self.max_buffer_usage.max(self.messages.len());
        self.messages.len()
    }

    /// `push_callp`: the engine rejects a missing target.
    pub fn push_call(
        &mut self,
        object: ObjectId,
        method: &str,
        args: Vec<Variant>,
        show_error: bool,
    ) -> bool {
        if object == ObjectId::NONE {
            return false;
        }
        self.push(Message::Call {
            object,
            method: StringName::new(method),
            args,
            show_error,
        });
        true
    }

    /// `push_callablep`.
    pub fn push_callable(&mut self, callable: Callable, args: Vec<Variant>, show_error: bool) -> bool {
        if callable.object == ObjectId::NONE && callable.method.is_empty() {
            return false;
        }
        self.push(Message::Callable {
            callable,
            args,
            show_error,
        });
        true
    }

    /// `push_set`.
    pub fn push_set(&mut self, object: ObjectId, property: &str, value: Variant) -> bool {
        if object == ObjectId::NONE {
            return false;
        }
        self.push(Message::Set {
            object,
            property: StringName::new(property),
            value,
        });
        true
    }

    /// `push_notification`.
    pub fn push_notification(&mut self, object: ObjectId, notification: i32) -> bool {
        if object == ObjectId::NONE {
            return false;
        }
        self.push(Message::Notification {
            object,
            notification,
        });
        true
    }

    /// `flush`: dispatches the messages queued when the call started, in order.
    pub fn flush<F: FnMut(&Message)>(&mut self, mut dispatch: F) -> usize {
        if self.flushing {
            return 0;
        }
        self.flushing = true;
        let pending: Vec<Message> = self.messages.drain(..).collect();
        let count = pending.len();
        for message in &pending {
            dispatch(message);
        }
        self.flushing = false;
        count
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn has_messages(&self) -> bool {
        !self.messages.is_empty()
    }

    pub fn is_flushing(&self) -> bool {
        self.flushing
    }

    pub fn get_max_buffer_usage(&self) -> usize {
        self.max_buffer_usage
    }

    pub fn get_message_count(&self) -> usize {
        self.messages.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u64) -> ObjectId {
        ObjectId::from_u64(value)
    }

    #[test]
    fn push_and_flush_order() {
        let mut queue = MessageQueueData::new();
        assert!(!queue.has_messages());
        assert!(queue.push_call(id(1), "run", vec![Variant::Int(1)], false));
        assert!(queue.push_set(id(1), "position", Variant::Int(2)));
        assert!(queue.push_notification(id(2), 7));
        assert_eq!(queue.get_message_count(), 3);

        let mut seen = Vec::new();
        let count = queue.flush(|message| {
            seen.push(match message {
                Message::Call { method, args, .. } => {
                    format!("call {} {}", method.as_string(), args.len())
                }
                Message::Set { property, .. } => format!("set {}", property.as_string()),
                Message::Notification { notification, .. } => format!("note {notification}"),
                Message::Callable { .. } => "callable".to_string(),
            });
        });
        assert_eq!(count, 3);
        assert_eq!(
            seen,
            vec!["call run 1".to_string(), "set position".to_string(), "note 7".to_string()]
        );
        assert!(!queue.has_messages());
        assert_eq!(queue.get_max_buffer_usage(), 3);
    }

    #[test]
    fn invalid_targets_are_rejected() {
        let mut queue = MessageQueueData::new();
        assert!(!queue.push_call(ObjectId::NONE, "x", Vec::new(), false));
        assert!(!queue.push_set(ObjectId::NONE, "x", Variant::Nil));
        assert!(!queue.push_notification(ObjectId::NONE, 1));
        assert!(!queue.push_callable(
            Callable {
                object: ObjectId::NONE,
                method: StringName::new(""),
            },
            Vec::new(),
            false
        ));
        assert!(!queue.has_messages());
    }

    #[test]
    fn flush_state_and_clear() {
        let mut queue = MessageQueueData::new();
        queue.push_notification(id(3), 1);
        assert!(!queue.is_flushing());
        let mut count = 0;
        queue.flush(|_| count += 1);
        assert_eq!(count, 1);
        assert!(!queue.is_flushing());
        assert!(!queue.has_messages());

        queue.push_notification(id(3), 2);
        queue.clear();
        assert_eq!(queue.get_message_count(), 0);
        assert_eq!(queue.get_max_buffer_usage(), 1);
    }
}
