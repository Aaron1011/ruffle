#!/bin/bash

# This script compiles the Actionscript files in 'core/src/playerglobal/avm2/'
# to the file 'core/src/playerglobal/avm2/library.swf'. Files should be placed
# in a directory structure corresponding to the package name
# (e.g. 'core/src/playerglobal/avm2/flash/geom/Point.as' corresponds to
# the class 'flash.geom.Point')

# All '.as' files found in 'core/src/playerglobal/avm2/' are included in
# 'library.swf' - there is no need to register them explicitly

# The compiled 'core/src/playerglobal/avm2/library.swf' is checked into
# the Git repository, and is included into the compiled Ruffle
# artifacts using `include_bytes!

set -euxo pipefail

# We need to use a custom target directory, so that
# this script can be invoked from a build script. Otherwise,
# the `cargo` invocation at the bottom of this script will
# deadlock due to the outer `cargo build` process already
# acquiring a lock on the target directory
OUT=./target/ruffle_playerglobal

rm -rf $OUT
mkdir -p $OUT

pushd core/src/playerglobal/avm2/
# Map a path like 'flash/geom/Point.as' to 'flash.geom.Point'
CLASSES=$(find . -name "*.as"  | sed -e 's/\.\///' -e 's/\.as//' -e 's/\//\./g')
popd

# Running 'compc' produces a '.swc' file, which contains 'catalog.xml' and a 'library.swf'
# We only need 'library.swf', so we extract it
COMPC_ARGS="-metadata.date=0 -compute-digest=false -compiler.debug=false -source-path core/src/playerglobal/avm2/ -include-classes $CLASSES -output=$OUT/playerglobal.swc"

# Use a Docker container if FLEX_SDK is not set
set +u
if [[ -z "${FLEX_SDK}" ]]; then
	docker run --rm -v $(pwd):/app instrumentisto/flex-sdk compc $COMPC_ARGS
else
	$FLEX_SDK/compc $COMPC_ARGS
fi
set -u

unzip -d $OUT $OUT/playerglobal.swc
cp $OUT/library.swf core/src/playerglobal/avm2/


# Modify the generated 'library.swf' to make the build reproducible
# The 'compc' compile unconditionally inserts a 'ProductInfo' tag, which
# contains the current date. We strip out all tags other than `DoAbc` tags
# from our SWF file. This eliminates unnecessary tags (Ruffle initialization
# only needs bytecode class definitions from 'playerglobal'), and ensures
# that the compiled SWF only depends on the contents of the '.as' files
# used as inputs

cargo run --package=swf --bin prepare_playerglobal --target-dir $OUT -- core/src/playerglobal/avm2/library.swf
