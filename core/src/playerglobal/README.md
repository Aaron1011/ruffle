# Ruffle playerglobal

This directory contains implementations of builtin Flash classes,
(e.g. 'flash.geom.Point') written in Actionscript. In many cases, this
is a simpler alternative to defining Flash classes in Rust
(in 'core/src/avm2/globals/flash').

Flash's `playerglobal.swc` cannot be uesd as a drop-in replacement
(though it's similar to our `playerglobal`). In addition to potential
copyright issues around redistributing Flash's `playerglobal.swc`,
many of its classes rely on specific 'native' method being provided
by the Flash VM, which Ruffle does not implement.

## Usage

These classes are processed by the 'compile\_swc.sh' script
in the root of the repository. The script first uses 'compc'
from the Flex SDK to compile these ActionScript files into
a 'playerglobal.swc' file. It then extracts the 'library.swf'
from the SWC file (which is just a zip archive), placing it
into the repository as 'core/src/playerglobal/avm2/library.swf'.
This file is checked into the repository, allowing
Ruffle to be compiled without running this script
(when the files are unchanged).

When you add or modify any classes in 'core/src/playerglobal',
you must run 'compile\_swc.sh' to rebuild 'library.swf'

## Limitations

* Currently, this script only runs on Linux - however, a Github Actions job
on open PRs will run this script, and make the result available
as a built artifact for download.
* Only pure ActionScript classes are currently supported. Classes with
'native' methods are not yet supported.
* 'Special' classes which are loaded early during Ruffle initialization
(e.g. 'Object', 'Function', 'Class') cannot currently
be implemented in 'playerglobal', since Ruffle initializes them in a special
way. However, virtually all classes in the 'flash' package are initialized
in a 'normal' way, and are eligible for implementation in 'playerglobal'
