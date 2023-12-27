use std::{
    cell::{Ref, RefMut},
    fmt,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};

use crate::{
    avm2::{Activation, Error, Value},
    tag_utils::SwfMovie,
    Player, PlayerBuilder, context::UpdateContext,
};

use super::{ClassObject, Object, ObjectPtr, ScriptObjectData, TObject};

/// A class instance allocator that allocates Worker objects.
pub fn worker_allocator<'gc>(
    _class: ClassObject<'gc>,
    _activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    unreachable!()
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct WorkerObject<'gc>(pub GcCell<'gc, WorkerObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct WorkerObjectWeak<'gc>(pub GcWeakCell<'gc, WorkerObjectData<'gc>>);

impl fmt::Debug for WorkerObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Collect)]
#[collect(require_static)]
pub enum WorkerKind {
    Primordial,
    NonPrimordial {
        movie: SwfMovie,
        running: Arc<AtomicBool>,
        // FIXME - actually join on this on primordial worker shutdown
        join_handle: Option<JoinHandle<()>>,
    },
}

pub struct WorkerHandle {
    running: Arc<AtomicBool>,
}

impl<'gc> WorkerObject<'gc> {
    pub fn new_primordial(context: &mut UpdateContext<'_, 'gc>) -> Self {
        let class = context.avm2.classes().worker;
        let base = ScriptObjectData::new(class);

        WorkerObject(GcCell::new(
            context.gc_context,
            WorkerObjectData {
                base,
                kind: WorkerKind::Primordial,
                other_workers: Arc::new(Mutex::new(vec![])),
            },
        ))
    }

    pub fn new_non_primordial(
        &self,
        movie: SwfMovie,
        activation: &mut Activation<'_, 'gc>,
    ) -> Self {
        let class = activation.avm2().classes().worker;
        let base = ScriptObjectData::new(class);

        let other_workers = self.0.read().other_workers.clone();
        let running = Arc::new(AtomicBool::new(true));
        let new_worker = WorkerObject(GcCell::new(
            activation.context.gc_context,
            WorkerObjectData {
                base,
                kind: WorkerKind::NonPrimordial {
                    movie,
                    running: running.clone(),
                    join_handle: None,
                },
                other_workers: other_workers.clone(),
            },
        ));
        other_workers.lock().unwrap().push(WorkerHandle { running });
        new_worker
    }

    pub fn is_primordial(&self) -> bool {
        matches!(self.0.read().kind, WorkerKind::Primordial)
    }

    pub fn start(&self, activation: &mut Activation<'_, 'gc>) {
        let mut write = self.0.write(activation.context.gc_context);
        let WorkerKind::NonPrimordial {
            running,
            ref mut join_handle,
            movie,
        } = &mut write.kind
        else {
            panic!("Can't start primordial worker!")
        };
        // FIXME - use Arc
        let movie = movie.clone();
        let running = running.clone();
        if join_handle.is_some() {
            panic!("Worker already started!");
        }

        let handle = std::thread::spawn(move || {
            let player = PlayerBuilder::new().with_movie(movie).build();
            let mut player = player.lock().unwrap();
            // FIXME - sleep betweeen frames
            while running.load(Ordering::Relaxed) {
                player.tick(100.0);
                // This has side effects
                player.render();
            }
        });
        *join_handle = Some(handle);
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct WorkerObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    kind: WorkerKind,
    #[collect(require_static)]
    pub other_workers: Arc<Mutex<Vec<WorkerHandle>>>,
}

impl<'gc> TObject<'gc> for WorkerObject<'gc> {
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

    fn as_worker_object(&self) -> Option<WorkerObject<'gc>> {
        Some(*self)
    }
}
