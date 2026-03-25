use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use crate::event_bus::EventBus;
use crate::events::TaskId;

#[derive(Debug)]
pub struct TaskManager {
    pub(crate) bus: Arc<EventBus>,
    /// Handle to the tokio runtime captured at construction time (inside an async context).
    /// Used to spawn tasks from non-async contexts (e.g. the iced update function on the
    /// macOS main thread, which runs outside the tokio runtime).
    rt: Handle,
    next_id: AtomicU64,
    handles: Mutex<HashMap<TaskId, JoinHandle<()>>>,
}

impl TaskManager {
    /// Must be called from within a tokio runtime (e.g. inside a `Task::perform` closure)
    /// so that `Handle::current()` succeeds.
    pub fn new(bus: Arc<EventBus>) -> Arc<Self> {
        Arc::new(Self {
            bus,
            rt: Handle::current(),
            next_id: AtomicU64::new(1),
            handles: Mutex::new(HashMap::new()),
        })
    }

    pub fn _cancel(&self, task_id: TaskId) {
        if let Some(handle) = self.handles.lock().unwrap().remove(&task_id) {
            handle.abort();
        }
    }

    /// Allocates a [`TaskId`], spawns `make_future(task_id)` on the tokio runtime, and
    /// registers the handle for cancellation. The closure receives the id so the future
    /// can embed it in emitted events.
    pub fn spawn<F, Fut>(&self, make_future: F) -> TaskId
    where
        F: FnOnce(TaskId) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let task_id = TaskId::new(self.next_id.fetch_add(1, Ordering::Relaxed));
        let handle = self.rt.spawn(make_future(task_id));
        self.handles.lock().unwrap().insert(task_id, handle);
        task_id
    }
}
