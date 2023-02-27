//! `flash.utils` namespace

use crate::avm2::object::TObject;
use crate::avm2::QName;
use crate::avm2::property::Property;
use crate::avm2::{Activation, Error, Object, Value};
use crate::avm2_stub_method;
use crate::string::AvmString;
use crate::string::WString;
use instant::Instant;
use std::fmt::Write;

pub mod byte_array;
pub mod dictionary;
pub mod proxy;
pub mod timer;

/// Implements `flash.utils.getTimer`
pub fn get_timer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok((Instant::now()
        .duration_since(activation.context.start_time)
        .as_millis() as u32)
        .into())
}

/// Implements `flash.utils.setInterval`
pub fn set_interval<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Err(Error::from("setInterval: not enough arguments"));
    }
    let (args, params) = args.split_at(2);
    let callback = crate::timer::TimerCallback::Avm2Callback {
        closure: args
            .get(0)
            .expect("setInterval: not enough arguments")
            .as_object()
            .ok_or("setInterval: argument 0 is not an object")?,
        params: params.to_vec(),
    };
    let interval = args
        .get(1)
        .expect("setInterval: not enough arguments")
        .coerce_to_number(activation)?;
    Ok(Value::Integer(activation.context.timers.add_timer(
        callback,
        interval as i32,
        false,
    )))
}

/// Implements `flash.utils.clearInterval`
pub fn clear_interval<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = args
        .get(0)
        .ok_or("clearInterval: not enough arguments")?
        .coerce_to_number(activation)?;
    activation.context.timers.remove(id as i32);
    Ok(Value::Undefined)
}

/// Implements `flash.utils.setTimeout`
pub fn set_timeout<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Err(Error::from("setTimeout: not enough arguments"));
    }
    let (args, params) = args.split_at(2);
    let callback = crate::timer::TimerCallback::Avm2Callback {
        closure: args
            .get(0)
            .expect("setTimeout: not enough arguments")
            .as_object()
            .ok_or("setTimeout: argument 0 is not an object")?,
        params: params.to_vec(),
    };
    let interval = args
        .get(1)
        .expect("setTimeout: not enough arguments")
        .coerce_to_number(activation)?;
    Ok(Value::Integer(activation.context.timers.add_timer(
        callback,
        interval as i32,
        true,
    )))
}

/// Implements `flash.utils.clearTimeout`
pub fn clear_timeout<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = args
        .get(0)
        .ok_or("clearTimeout: not enough arguments")?
        .coerce_to_number(activation)?;
    activation.context.timers.remove(id as i32);
    Ok(Value::Undefined)
}

/// Implements `flash.utils.escapeMultiByte`
pub fn escape_multi_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let s = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let utf8 = s.as_wstr().to_utf8_lossy();
    let mut result = WString::new();
    for byte in utf8.as_bytes() {
        if *byte == 0 {
            break;
        }
        if byte.is_ascii_alphanumeric() {
            result.push_byte(*byte);
        } else {
            let _ = write!(&mut result, "%{byte:02X}");
        }
    }
    Ok(AvmString::new(activation.context.gc_context, result).into())
}

fn handle_percent<I>(chars: &mut I) -> Option<u8>
where
    I: Iterator<Item = char>,
{
    let high = chars.next()?.to_digit(16)? as u8;
    let low = chars.next()?.to_digit(16)? as u8;
    Some(low | (high << 4))
}

/// Implements `flash.utils.unescapeMultiByte`
pub fn unescape_multi_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let s = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let bs = s.as_wstr();
    let mut buf = WString::new();
    let chars = bs.chars().map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER));

    let mut chars = chars.peekable();
    while let Some(c) = chars.next() {
        if c == '\0' {
            break;
        }
        if c == '%' {
            let mut bytes = Vec::new();
            while let Some(b) = handle_percent(&mut chars) {
                bytes.push(b);
                if !matches!(chars.peek(), Some('%')) {
                    break;
                }
                chars.next();
            }
            buf.push_str(&WString::from_utf8_bytes(bytes));

            continue;
        }

        buf.push_char(c);
    }
    let v = AvmString::new(activation.context.gc_context, buf);
    Ok(v.into())
}

/// Implements `flash.utils.getQualifiedClassName`
pub fn get_qualified_class_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // This is a native method, which enforces the argument count.
    let val = args[0];
    match val {
        Value::Null => return Ok("null".into()),
        Value::Undefined => return Ok("void".into()),
        _ => {}
    }
    let obj = val.coerce_to_object(activation)?;

    let class = match obj.as_class_object() {
        Some(class) => class,
        None => match obj.instance_of() {
            Some(cls) => cls,
            None => return Ok(Value::Null),
        },
    };

    Ok(class
        .inner_class_definition()
        .read()
        .name()
        .to_qualified_name(activation.context.gc_context)
        .into())
}

/// Implements `flash.utils.getQualifiedSuperclassName`
pub fn get_qualified_superclass_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let obj = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)?;

    let class = match obj.as_class_object() {
        Some(class) => class,
        None => match obj.instance_of() {
            Some(cls) => cls,
            None => return Ok(Value::Null),
        },
    };

    if let Some(super_class) = class.superclass_object() {
        Ok(super_class
            .inner_class_definition()
            .read()
            .name()
            .to_qualified_name(activation.context.gc_context)
            .into())
    } else {
        Ok(Value::Null)
    }
}

/// Implements native method `flash.utils.getDefinitionByName`
pub fn get_definition_by_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let appdomain = activation.caller_domain();
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let qname = QName::from_qualified_name(name, activation);
    appdomain.get_defined_value(activation, qname)
}

// Implements `flash.utils.describeType`
pub fn describe_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // This method is very incomplete, and should be fully implemented
    // once we have a better way of constructing XML from the Rust side
    avm2_stub_method!(activation, "flash.utils", "describeType");

    let mut xml_string = String::new();
    let qualified_name =
        get_qualified_class_name(activation, None, &[args[0]])?.coerce_to_string(activation)?;

    let value = args[0].as_object().unwrap();

    let class_obj = args[0].as_object().unwrap().instance_of().unwrap();
    let class = class_obj.inner_class_definition();
    let class = class.read();

    let superclass = class_obj.superclass_object().unwrap();

    let base = superclass.inner_class_definition().read().name().to_qualified_name(activation.context.gc_context);
    let is_dynamic = !class.is_sealed();
    let is_final = class.is_final();

    let mut super_obj = superclass;


    let (is_static, vtable) = if let Some(as_class) = value.as_class_object() {
        (true, as_class.vtable())
    } else {
        (false, class_obj.vtable())
    };

    write!(xml_string, "<type name=\"{qualified_name}\" base=\"{base}\" isDynamic=\"{is_dynamic}\" isFinal=\"{is_final}\" isStatic=\"{is_static}\">").unwrap();
    loop {
        let super_name = super_obj.inner_class_definition().read().name().to_qualified_name(activation.context.gc_context);
        write!(xml_string, "<extendsClass type=\"{super_name}\"/>").unwrap();
        if let Some(superclass) = super_obj.superclass_object() {
            super_obj = superclass;
        } else {
            break;
        }
    }

    for interface in class_obj.interfaces() {
        let interface_name = interface.read().name().to_qualified_name(activation.context.gc_context);
        write!(xml_string, "<implementsInterface type=\"{interface_name}\"/>").unwrap();
    }

    if let Some(vtable) = vtable {
        // FIXME - get the correct order here
        for (prop_name, _ns, prop) in vtable.resolved_traits().iter() {
            match prop {
                Property::ConstSlot { slot_id } | Property::Slot { slot_id } => {
                    let prop_class = vtable.slot_class(*slot_id, activation)?;
                    let prop_class_name = if let Some(prop_class) = prop_class {
                        prop_class
                            .inner_class_definition()
                            .read()
                            .name()
                            .to_qualified_name(activation.context.gc_context)
                    } else {
                        AvmString::new_utf8(activation.context.gc_context, "*")
                    };

                    eprintln!("Got prop: {:?} {:?}", prop_name, prop);

                    let elem_name = match prop {
                        Property::ConstSlot { .. } => "constant",
                        Property::Slot { .. } => "variable",
                        _ => unreachable!(),
                    };

                    write!(xml_string, "<{elem_name} name=\"{prop_name}\" type=\"{prop_class_name}\"/>").unwrap();
                }
                Property::Method { .. } => {
                    write!(xml_string, "<method name=\"{prop_name}\"/>").unwrap();
                }
                Property::Virtual { get, set } => {
                    let access = match (get, set) {
                        (Some(_), Some(_)) => "readwrite",
                        (Some(_), None) => "readonly",
                        (None, Some(_)) => "writeonly",
                        (None, None) => unreachable!(),
                    };
                    write!(xml_string, "<accessor name=\"{prop_name}\" access=\"{access}\"/>").unwrap();
                }
            }
        }
    }

    xml_string += "</type>";

    eprintln!("Built describeXML: {}", xml_string);

    let xml_avm_string = AvmString::new_utf8(activation.context.gc_context, xml_string);

    Ok(activation
        .avm2()
        .classes()
        .xml
        .construct(activation, &[xml_avm_string.into()])?
        .into())
}
