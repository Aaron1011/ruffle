use crossbeam_channel::{Receiver, Sender, TryRecvError};
use std::time::Duration;
use std::{
    cell::{Ref, RefMut},
    fmt::{self},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};

use crate::{
    avm2::{Activation, Error, Value},
    context::UpdateContext,
    tag_utils::SwfMovie,
    Player, PlayerBuilder,
};

use super::{ClassObject, Object, ObjectPtr, ScriptObjectData, TObject, WorkerObject};

/// A class instance allocator that allocates Worker objects.
pub fn messagechannel_allocator<'gc>(
    _class: ClassObject<'gc>,
    _activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    unreachable!()
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct MessageChannelObject<'gc>(pub GcCell<'gc, MessageChannelObjectData<'gc>>);

impl<'gc> MessageChannelObject<'gc> {
    pub fn new(
        sender_worker: WorkerObject<'gc>,
        receiver_worker: WorkerObject<'gc>,
        context: &mut UpdateContext<'_, 'gc>,
    ) -> Self {
        let class = context.avm2.classes().messagechannel;
        let base = ScriptObjectData::new(class);

        let (sender, receiver) = crossbeam_channel::unbounded();

        MessageChannelObject(GcCell::new(
            context.gc_context,
            MessageChannelObjectData {
                base,
                sender_worker,
                receiver_worker,
                sender,
                receiver,
            },
        ))
    }

    pub fn send(
        &self,
        message: MessageChannelMessage,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        // FIXME -
        let read = self.0.read();
        if *activation.context.worker != read.sender_worker.into() {
            return Err("Cannot send from a worker that does not own the channel".into());
        }
        read.sender.send(message).unwrap();
        Ok(())
    }

    pub fn receive(
        &self,
        block: bool,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<MessageChannelMessage>, Error<'gc>> {
        let read = self.0.read();
        if *activation.context.worker != read.receiver_worker.into() {
            return Err("Cannot receive from a worker that does not own the channel".into());
        }
        if block {
            if activation
                .context
                .worker
                .as_worker_object()
                .unwrap()
                .is_primordial()
            {
                // FIXME - throw ScriptTimeoutError
                Ok(Some(
                    read.receiver.recv_timeout(Duration::from_secs(15)).unwrap(),
                ))
            } else {
                Ok(Some(read.receiver.recv().unwrap()))
            }
        } else {
            match read.receiver.try_recv() {
                Ok(msg) => Ok(Some(msg)),
                Err(TryRecvError::Empty) => Ok(None),
                // TODO - handle closed
                Err(e) => panic!("Error receiving message: {:?}", e),
            }
        }
    }
}

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct MessageChannelObjectWeak<'gc>(pub GcWeakCell<'gc, MessageChannelObjectData<'gc>>);

impl fmt::Debug for MessageChannelObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessageChannelObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct MessageChannelObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    sender_worker: WorkerObject<'gc>,
    receiver_worker: WorkerObject<'gc>,

    #[collect(require_static)]
    sender: Sender<MessageChannelMessage>,
    #[collect(require_static)]
    receiver: Receiver<MessageChannelMessage>,
}

pub enum MessageChannelMessage {
    AMF3(Vec<u8>),
}
impl<'gc> TObject<'gc> for MessageChannelObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object((*self).into()))
    }

    fn as_message_channel_object(&self) -> Option<MessageChannelObject<'gc>> {
        Some(*self)
    }
}
