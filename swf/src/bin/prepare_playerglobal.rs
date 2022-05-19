//! An internal Ruffle utility to perform post-processing
//! of our compiled `library.swf` (from our `playerglobal`).

use std::env;
use std::fs::File;
use swf::read::{decompress_swf, parse_swf};
use swf::write::write_swf;
use swf::Tag;

fn main() {
    let filename = env::args().nth(1).unwrap();
    let in_data = std::fs::read(filename.clone()).unwrap();
    let swf_buf = decompress_swf(&in_data[..]).unwrap();
    let mut swf = parse_swf(&swf_buf).unwrap();
    let out_file = File::create(filename).unwrap();
    let encoding = swf::SwfStr::encoding_for_version(swf_buf.header.version());

    swf.tags = swf
        .tags
        .into_iter()
        .filter(|tag| {
            // If the tag isn't `Tag::DoAbc`, then discard it -
            // Ruffle initialization only needs bytecode
            // from 'playerglobal'
            let abc = if let Tag::DoAbc(abc) = tag {
                abc
            } else {
                return false;
            };

            // The `compc` tool generates a class ending with `_flash_display_Sprite`
            // There's no command-line flag to prevent it from being generated,
            // so we discard it here.
            !abc.name
                .to_str_lossy(encoding)
                .ends_with("_flash_display_Sprite")
        })
        .collect();
    write_swf(swf.header.swf_header(), &swf.tags, out_file).unwrap();
}
