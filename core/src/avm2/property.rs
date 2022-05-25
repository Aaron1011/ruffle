//! Property data structures

use crate::avm2::ClassObject;
use gc_arena::Collect;

#[derive(Debug, Collect, Clone, Copy)]
#[collect(no_drop)]
pub enum Property<'gc> {
    Virtual { get: Option<u32>, set: Option<u32> },
    Method { disp_id: u32 },
    Slot { slot_id: u32, class: ClassObject<'gc> },
    ConstSlot { slot_id: u32, class: ClassObject<'gc> },
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

    pub fn new_slot(slot_id: u32, class: ClassObject<'gc>) -> Self {
        Property::Slot { slot_id, class }
    }

    pub fn new_const_slot(slot_id: u32, class: ClassObject<'gc>) -> Self {
        Property::ConstSlot { slot_id, class }
    }
}
