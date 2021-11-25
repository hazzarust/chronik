use std::collections::HashMap;

use bitcoinsuite_core::Sha256d;
use chronik_rocksdb::PayloadPrefix;
use tokio::sync::broadcast;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscribeMessage {
    AddedToMempool(Sha256d),
    RemovedFromMempool(Sha256d),
    Confirmed(Sha256d),
    Reorg(Sha256d),
}

const CHANNEL_CAPACITY: usize = 16;

#[derive(Debug, Clone, Default)]
pub struct Subscribers {
    subs: HashMap<(PayloadPrefix, Vec<u8>), broadcast::Sender<SubscribeMessage>>,
}

impl Subscribers {
    pub fn subscribe(
        &mut self,
        script: &(PayloadPrefix, Vec<u8>),
    ) -> broadcast::Receiver<SubscribeMessage> {
        match self.subs.get(script) {
            Some(sender) => sender.subscribe(),
            None => {
                let (sender, receiver) = broadcast::channel(CHANNEL_CAPACITY);
                self.subs.insert(script.clone(), sender);
                receiver
            }
        }
    }

    /// Clean unsubscribe
    pub fn unsubscribe(&mut self, script: &(PayloadPrefix, Vec<u8>)) {
        if let Some(sender) = self.subs.get(script) {
            if sender.receiver_count() == 0 {
                self.subs.remove(script);
            }
        }
    }

    pub(crate) fn broadcast(&mut self, script: &(PayloadPrefix, Vec<u8>), msg: SubscribeMessage) {
        if let Some(sender) = self.subs.get(script) {
            // Unclean unsubscribe
            if sender.send(msg).is_err() {
                self.subs.remove(script);
            }
        }
    }
}
