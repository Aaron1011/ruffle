//! `flash.geom.ColorTransform` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::value::Value;
use crate::avm2::{Error, Object};
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.geom.ColorTransform`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.geom.ColorTransform`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.geom"), "ColorTransform"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<ColorTransform instance initializer>", mc),
        Method::from_builtin(class_init, "<ColorTransform class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);

    const PUBLIC_INSTANCE_SLOTS: &[(&str, &str, &str)] = &[
        ("alphaMultiplier", "", "Number"),
        ("alphaOffset", "", "Number"),
        ("blueMultiplier", "", "Number"),
        ("blueOffset", "", "Number"),
        ("color", "", "uint"),
        ("greenMultiplier", "", "Number"),
        ("greenOffset", "", "Number"),
        ("redMultiplier", "", "Number"),
        ("redOffset", "", "Number"),
    ];
    write.define_public_slot_instance_traits(PUBLIC_INSTANCE_SLOTS);

    class
}
