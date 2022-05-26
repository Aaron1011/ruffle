//! Property data structures

use crate::avm2::ClassObject;
use crate::avm2::Activation;
use crate::avm2::Error;
use crate::avm2::names::Multiname;
use gc_arena::{Gc, Collect};

#[derive(Debug, Collect, Clone, Copy)]
#[collect(no_drop)]
pub enum Property<'gc> {
    Virtual { get: Option<u32>, set: Option<u32> },
    Method { disp_id: u32 },
    Slot { slot_id: u32, class: LazyClass<'gc> },
    ConstSlot { slot_id: u32, class: LazyClass<'gc> },
}

#[derive(Debug, Collect, Clone, Copy)]
#[collect(no_drop)]
pub enum LazyClass<'gc> {
    Class(ClassObject<'gc>),
    Name(Gc<'gc, Multiname<'gc>>),
}

impl<'gc> LazyClass<'gc> {
    pub fn lazy(activation: &mut Activation<'_, 'gc, '_>, name: Multiname<'gc>) -> Self {
        LazyClass::Name(Gc::allocate(activation.context.gc_context, name))
    }
    pub fn get(&mut self, activation: &mut Activation<'_, 'gc, '_>) -> Result<ClassObject<'gc>, Error> {
        match self {
            LazyClass::Class(class) => Ok(*class),
            LazyClass::Name(name) => {
                let class = activation.resolve_class(name)?;
                *self = LazyClass::Class(class);
                Ok(class)
            }
        }
    }
}

impl<'gc> Property<'gc> {
    pub fn new_method(disp_id: u32) -> Self {
        Property::Method { disp_id }
    }

    pub fn new_getter(disp_id: u32) -> Self {
        Property::Virtual {
            get: Some(disp_id),
            set: None,
        }
    }

    pub fn new_setter(disp_id: u32) -> Self {
        Property::Virtual {
            get: None,
            set: Some(disp_id),
        }
    }

    pub fn new_slot(slot_id: u32, class: LazyClass<'gc>) -> Self {
        Property::Slot { slot_id, class }
    }

    pub fn new_const_slot(slot_id: u32, class: LazyClass<'gc>) -> Self {
        Property::ConstSlot { slot_id, class }
    }
}
