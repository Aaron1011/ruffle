//! XML builtin and prototype

use crate::avm2::e4x::{E4XNode, E4XNodeKind};
pub use crate::avm2::object::xml_allocator;
use crate::avm2::object::{E4XOrXml, QNameObject, TObject, XmlListObject};
use crate::avm2::string::AvmString;
use crate::avm2::{Activation, Error, Multiname, Object, Value};
use crate::avm2_stub_method;

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap().as_xml_object().unwrap();
    let mut value = args[0];

    if let Some(xml) = value.as_object().and_then(|o| o.as_xml_object()) {
        // Convert it to a string and re-parse (deep clone)
        value = xml.node().xml_to_xml_string(activation)?.into();
    }

    match E4XNode::parse(value, activation) {
        Ok(nodes) => {
            let node = match nodes.as_slice() {
                // XML defaults to an empty text node when nothing was parsed
                [] => E4XNode::text(activation.context.gc_context, AvmString::default()),
                [node] => *node,
                _ => {
                    return Err(Error::RustError(
                        format!(
                            "XML constructor must be called with a single node: found {:?}",
                            nodes
                        )
                        .into(),
                    ))
                }
            };
            this.set_node(activation.context.gc_context, node);
        }
        Err(e) => {
            return Err(Error::RustError(
                format!("Failed to parse XML: {e:?}").into(),
            ))
        }
    }

    Ok(Value::Undefined)
}

pub fn name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let node = this.unwrap().as_xml_object().unwrap();
    if let Some(local_name) = node.local_name() {
        avm2_stub_method!(activation, "XML", "name", "namespaces");
        // FIXME - use namespace
        let namespace = activation.avm2().public_namespace;
        Ok(QNameObject::from_name(activation, Multiname::new(namespace, local_name))?.into())
    } else {
        Ok(Value::Null)
    }
}

pub fn local_name<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let node = this.unwrap().as_xml_object().unwrap();
    Ok(node.local_name().map_or(Value::Null, Value::String))
}

pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.unwrap().as_xml_object().unwrap();
    let node = xml.node();
    Ok(Value::String(node.xml_to_string(activation)?))
}

pub fn to_xml_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.unwrap().as_xml_object().unwrap();
    let node = xml.node();
    Ok(Value::String(node.xml_to_xml_string(activation)?))
}

fn name_to_multiname<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: &Value<'gc>,
) -> Result<Multiname<'gc>, Error<'gc>> {
    Ok(match name {
        Value::String(s) => Multiname::new(activation.avm2().public_namespace, *s),
        Value::Object(o) => {
            if let Some(qname) = o.as_qname_object() {
                qname.name().clone()
            } else {
                Multiname::new(
                    activation.avm2().public_namespace,
                    name.coerce_to_string(activation)?,
                )
            }
        }
        _ => Multiname::new(
            activation.avm2().public_namespace,
            name.coerce_to_string(activation)?,
        ),
    })
}

pub fn child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.unwrap().as_xml_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0])?;
    // FIXME: Support numerical indexes.
    let children = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children
            .iter()
            .filter(|node| node.matches_name(&multiname))
            .map(|node| E4XOrXml::E4X(*node))
            .collect()
    } else {
        Vec::new()
    };

    Ok(XmlListObject::new(activation, children, Some(xml.into())).into())
}

pub fn children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.unwrap().as_xml_object().unwrap();
    let children = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children.iter().map(|node| E4XOrXml::E4X(*node)).collect()
    } else {
        Vec::new()
    };

    Ok(XmlListObject::new(activation, children, Some(xml.into())).into())
}

pub fn elements<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.unwrap().as_xml_object().unwrap();
    let children = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children
            .iter()
            .filter(|node| matches!(&*node.kind(), E4XNodeKind::Element { .. }))
            .map(|node| E4XOrXml::E4X(*node))
            .collect()
    } else {
        Vec::new()
    };

    Ok(XmlListObject::new(activation, children, Some(xml.into())).into())
}

pub fn attributes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap();
    let xml = this.as_xml_object().unwrap();
    let attributes = if let E4XNodeKind::Element { attributes, .. } = &*xml.node().kind() {
        attributes.iter().map(|node| E4XOrXml::E4X(*node)).collect()
    } else {
        Vec::new()
    };

    Ok(XmlListObject::new(activation, attributes, Some(xml.into())).into())
}

pub fn attribute<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap();
    let xml = this.as_xml_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0])?;
    let attribute = if let E4XNodeKind::Element { attributes, .. } = &*xml.node().kind() {
        attributes
            .iter()
            .find(|node| node.matches_name(&multiname))
            .copied()
    } else {
        None
    };

    Ok(XmlListObject::new(
        activation,
        attribute.map_or(Vec::new(), |node| vec![E4XOrXml::E4X(node)]),
        Some(xml.into()),
    )
    .into())
}

pub fn append_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap();
    let xml = this.as_xml_object().unwrap();

    let child = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)?;

    let child = if let Some(child) = child.as_xml_object() {
        child
    } else {
        return Err(format!("XML.appendChild is not yet implemented for {child:?}").into())
    };

    let child = child.node();
    
    xml.node().append_child(activation.context.gc_context, *child)?;

    Ok(Value::Undefined)
}