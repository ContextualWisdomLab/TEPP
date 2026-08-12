//! In-memory registry separating mentions from instances.

use crate::{EventError, EventInstance, EventInstanceId, EventMention, EventMentionId};
use std::collections::BTreeMap;

/// Registry that keeps mentions and instances in separate namespaces.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EventRegistry {
    mentions: BTreeMap<EventMentionId, EventMention>,
    instances: BTreeMap<EventInstanceId, EventInstance>,
}

impl EventRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a mention.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::DuplicateEventIdentity`] when the mention ID exists.
    pub fn insert_mention(&mut self, mention: EventMention) -> Result<(), EventError> {
        let id = mention.mention_id();
        if self.mentions.contains_key(&id) {
            return Err(EventError::DuplicateEventIdentity);
        }
        self.mentions.insert(id, mention);
        Ok(())
    }

    /// Insert a promoted instance, verifying supporting mentions exist.
    ///
    /// # Errors
    ///
    /// Returns identity or unknown-mention errors when the promotion is invalid.
    pub fn insert_instance(&mut self, instance: EventInstance) -> Result<(), EventError> {
        let id = instance.instance_id();
        if self.instances.contains_key(&id) {
            return Err(EventError::DuplicateEventIdentity);
        }
        for mention_id in instance.supporting_mentions() {
            if !self.mentions.contains_key(mention_id) {
                return Err(EventError::UnknownEventInstance);
            }
        }
        self.instances.insert(id, instance);
        Ok(())
    }

    /// Return a mention by id.
    #[must_use]
    pub fn mention(&self, id: EventMentionId) -> Option<&EventMention> {
        self.mentions.get(&id)
    }

    /// Return an instance by id.
    #[must_use]
    pub fn instance(&self, id: EventInstanceId) -> Option<&EventInstance> {
        self.instances.get(&id)
    }

    /// Count mentions.
    #[must_use]
    pub fn mention_count(&self) -> usize {
        self.mentions.len()
    }

    /// Count instances.
    #[must_use]
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }
}
