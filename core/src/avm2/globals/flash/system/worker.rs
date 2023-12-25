use crate::avm2::{Activation, Error, Object, Value, TObject};
pub use crate::avm2::object::worker_allocator;

pub fn get_current<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.worker.into())
}

pub fn get_is_primordial<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_worker_object().unwrap().is_primordial().into())
}
