pub use crate::avm2::object::worker_allocator;
use crate::avm2::{
    object::MessageChannelObject, parameters::ParametersExt, Activation, Error, Object, TObject,
    Value,
};

pub fn get_current<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok((*activation.context.worker).into())
}

pub fn get_is_primordial<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_worker_object().unwrap().is_primordial().into())
}

pub fn create_message_channel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let sender = this.as_worker_object().unwrap();
    let receiver = args
        .get_object(activation, 0, "receiver")?
        .as_worker_object()
        .unwrap();

    Ok(MessageChannelObject::new(sender, receiver, &mut activation.context).into())
}
