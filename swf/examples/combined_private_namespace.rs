//! Modifies a SWF file to use the same private namespace for all fields named `FIELD_NAME`.
//! This is used to modify the SWF file `tests/tests/swfs/avm2/private_namespace/test.swf`

use std::fs::File;
use std::io::BufReader;

use swf::Tag;
use swf::avm2::types::{TraitKind, Multiname};
use swf::avm2::write::Writer;

const FIELD_NAME: &str = "myfield";

fn main() {
    let path = std::env::args().nth(1).expect("Missing path to SWF file");
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);
    let swf_buf = swf::decompress_swf(reader).unwrap();
    let mut swf = swf::parse_swf(&swf_buf).unwrap();

    let mut first_myfield_ns = None;

    for tag in swf.tags.iter_mut() {
        if let Tag::DoAbc(abc_tag) = tag {
            let mut reader = swf::avm2::read::Reader::new(abc_tag.data);
            let mut abc = reader.read().unwrap();
            for instance in &mut abc.instances {
                for abc_trait in &mut instance.traits {
                    if let TraitKind::Slot { .. } = abc_trait.kind {
                        let multiname = &mut abc.constant_pool.multinames[abc_trait.name.0 as usize - 1];
                        if let Multiname::QName { namespace, name } = multiname {
                            let string_name = &abc.constant_pool.strings[name.0 as usize - 1];
                            if string_name == FIELD_NAME {
                                if let Some(first_myfield_ns) = first_myfield_ns {
                                    eprintln!("Changing private namespace from {:?} to {:?} on trait {:?}", *namespace, first_myfield_ns, abc_trait);
                                    *namespace = first_myfield_ns;
                                } else {
                                    eprintln!("Found private namspace {:?} on trait {:?}", first_myfield_ns, abc_trait);
                                    first_myfield_ns = Some(*namespace);
                                }
                            }
                        }

                    }
                }
            }
            let mut out_data = Vec::new();
            let mut writer = Writer::new(&mut out_data);
            writer.write(abc).unwrap();
            abc_tag.data = Vec::leak(out_data);
        }
    }

    let file = std::fs::File::create(&path).unwrap();
    let writer = std::io::BufWriter::new(file);
    swf::write_swf(&swf.header.swf_header(), &swf.tags, writer).unwrap();

}
