use std::sync::{Arc, Mutex, PoisonError};

type PanelEmitter = Arc<dyn Fn(PersonalAssistantPanel) -> Result<(), String> + Send + Sync>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PersonalAssistantPanel {
    UserName,
    StickyMessage,
    DailyPlanner,
}

impl PersonalAssistantPanel {
    const fn index(self) -> usize {
        match self {
            Self::UserName => 0,
            Self::StickyMessage => 1,
            Self::DailyPlanner => 2,
        }
    }

    const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::UserName),
            1 => Some(Self::StickyMessage),
            2 => Some(Self::DailyPlanner),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
struct PendingPanelRequests {
    active: bool,
    generation: u64,
    pending: [bool; 3],
}

#[derive(Clone, Default)]
pub(crate) struct PersonalAssistantEventQueue {
    state: Arc<Mutex<PendingPanelRequests>>,
    emitter: Option<PanelEmitter>,
}

impl std::fmt::Debug for PersonalAssistantEventQueue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PersonalAssistantEventQueue")
            .field("state", &self.state)
            .field("emitter_configured", &self.emitter.is_some())
            .finish()
    }
}

impl PersonalAssistantEventQueue {
    pub(crate) fn with_emitter(emitter: PanelEmitter) -> Self {
        Self {
            state: Arc::new(Mutex::new(PendingPanelRequests::default())),
            emitter: Some(emitter),
        }
    }

    pub(crate) fn activate(&self) -> Result<(), PersonalAssistantEventError> {
        let generation = {
            let mut state = self.state.lock()?;
            state.generation = state.generation.wrapping_add(1);
            state.active = false;
            state.generation
        };

        self.flush(generation);
        Ok(())
    }

    pub(crate) fn deactivate(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.generation = state.generation.wrapping_add(1);
            state.active = false;
        }
    }

    pub(crate) fn request(&self, panel: PersonalAssistantPanel) {
        let generation = match self.state.lock() {
            Ok(mut state) => {
                state.pending[panel.index()] = true;

                if state.active {
                    state.active = false;
                    Some(state.generation)
                } else {
                    None
                }
            }
            Err(_) => {
                eprintln!("[personal-assistant] event_queue_unavailable");
                None
            }
        };

        if let Some(generation) = generation {
            self.flush(generation);
        }
    }

    fn flush(&self, generation: u64) {
        let Some(emitter) = self.emitter.as_ref() else {
            return;
        };

        loop {
            let panel = match self.state.lock() {
                Ok(mut state) if state.generation == generation => {
                    let Some(index) = state.pending.iter().position(|pending| *pending) else {
                        state.active = true;
                        return;
                    };
                    state.pending[index] = false;
                    PersonalAssistantPanel::from_index(index)
                }
                Ok(_) => return,
                Err(_) => {
                    eprintln!("[personal-assistant] event_queue_unavailable");
                    return;
                }
            };

            let Some(panel) = panel else {
                continue;
            };

            if let Err(error) = emitter(panel) {
                eprintln!("[personal-assistant] event_delivery_failed: {error}");

                if let Ok(mut state) = self.state.lock() {
                    if state.generation == generation {
                        state.pending[panel.index()] = true;
                        state.active = false;
                    }
                }
                return;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PersonalAssistantEventError {
    StateUnavailable,
}

impl<T> From<PoisonError<T>> for PersonalAssistantEventError {
    fn from(_error: PoisonError<T>) -> Self {
        Self::StateUnavailable
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{PersonalAssistantEventQueue, PersonalAssistantPanel};

    #[test]
    fn requests_wait_for_renderer_activation_and_survive_reload() {
        let delivered = Arc::new(Mutex::new(Vec::new()));
        let delivered_for_emitter = delivered.clone();
        let queue = PersonalAssistantEventQueue::with_emitter(Arc::new(move |panel| {
            delivered_for_emitter.lock().unwrap().push(panel);
            Ok(())
        }));

        queue.request(PersonalAssistantPanel::StickyMessage);
        assert!(delivered.lock().unwrap().is_empty());

        queue.activate().unwrap();
        assert_eq!(
            *delivered.lock().unwrap(),
            [PersonalAssistantPanel::StickyMessage]
        );

        queue.deactivate();
        queue.request(PersonalAssistantPanel::DailyPlanner);
        assert_eq!(delivered.lock().unwrap().len(), 1);

        queue.activate().unwrap();
        assert_eq!(
            *delivered.lock().unwrap(),
            [
                PersonalAssistantPanel::StickyMessage,
                PersonalAssistantPanel::DailyPlanner,
            ]
        );
    }

    #[test]
    fn repeated_requests_are_coalesced_before_delivery() {
        let delivered = Arc::new(Mutex::new(Vec::new()));
        let delivered_for_emitter = delivered.clone();
        let queue = PersonalAssistantEventQueue::with_emitter(Arc::new(move |panel| {
            delivered_for_emitter.lock().unwrap().push(panel);
            Ok(())
        }));

        queue.request(PersonalAssistantPanel::UserName);
        queue.request(PersonalAssistantPanel::UserName);
        queue.activate().unwrap();

        assert_eq!(
            *delivered.lock().unwrap(),
            [PersonalAssistantPanel::UserName]
        );
    }

    #[test]
    fn failed_delivery_remains_pending_for_the_next_activation() {
        let attempts = Arc::new(Mutex::new(0_u8));
        let attempts_for_emitter = attempts.clone();
        let queue = PersonalAssistantEventQueue::with_emitter(Arc::new(move |_panel| {
            let mut attempts = attempts_for_emitter.lock().unwrap();
            *attempts += 1;
            if *attempts == 1 {
                Err("renderer unavailable".to_string())
            } else {
                Ok(())
            }
        }));

        queue.request(PersonalAssistantPanel::DailyPlanner);
        queue.activate().unwrap();
        assert_eq!(*attempts.lock().unwrap(), 1);

        queue.activate().unwrap();
        assert_eq!(*attempts.lock().unwrap(), 2);
    }
}
