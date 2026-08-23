use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use crate::Injectable;

#[derive(Clone)]
pub struct EventEmitter {
    channels: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn emit<E: Clone + Send + Sync + 'static>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        let channels = self.channels.read().unwrap();
        if let Some(sender_any) = channels.get(&type_id) {
            if let Some(sender) = sender_any.downcast_ref::<broadcast::Sender<E>>() {
                let _ = sender.send(event);
            }
        }
    }

    pub fn subscribe<E: Clone + Send + Sync + 'static>(&self) -> broadcast::Receiver<E> {
        let type_id = TypeId::of::<E>();
        
        {
            let channels = self.channels.read().unwrap();
            if let Some(sender_any) = channels.get(&type_id) {
                if let Some(sender) = sender_any.downcast_ref::<broadcast::Sender<E>>() {
                    return sender.subscribe();
                }
            }
        }

        let mut channels = self.channels.write().unwrap();
        let sender = if let Some(sender_any) = channels.get(&type_id) {
            sender_any.downcast_ref::<broadcast::Sender<E>>().unwrap().clone()
        } else {
            let (sender, _) = broadcast::channel::<E>(1024);
            channels.insert(type_id, Box::new(sender.clone()));
            sender
        };

        sender.subscribe()
    }
}

impl<C> Injectable<C> for EventEmitter {
    fn create(_: &C) -> Arc<Self> {
        Arc::new(Self::new())
    }
}

pub trait EventListener<C>: Send + Sync + 'static {
    fn register_listeners(self: Arc<Self>, container: &C);
}
