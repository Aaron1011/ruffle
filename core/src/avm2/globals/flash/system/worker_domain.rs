use crate::{
    avm2::{parameters::ParametersExt, Activation, Error, Object, TObject, Value},
    tag_utils::SwfMovie,
};

pub fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, &[])?;

    Ok(Value::Undefined)
}

pub fn get_current<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok((*activation.context.worker_domain).into())
}

pub fn create_worker<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let swf = args
        .get_object(activation, 0, "swf")?
        .as_bytearray_object()
        .unwrap();
    let swf = swf.as_bytearray().unwrap();
    let bytes = swf.read_at(swf.len(), 0).unwrap();
    let give_app_privileges = args.get_bool(1);

    let movie = SwfMovie::from_data(bytes, String::new(), None).unwrap();

    let worker = activation
        .context
        .worker
        .as_worker_object()
        .unwrap()
        .new_non_primordial(movie, activation);
    Ok(worker.into())
}
