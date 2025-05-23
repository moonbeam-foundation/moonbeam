#!/bin/bash

cargo license --json > licenses.json
LICENSES=(
    "(MIT OR Apache-2.0) AND Unicode-DFS-2016"
    "(MIT OR Apache-2.0) AND Unicode-3.0"
    "0BSD OR Apache-2.0 OR MIT"
    "Apache-2.0 AND MIT"
    "Apache-2.0 OR LGPL-2.1-or-later OR MIT"
    "Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT"
    "Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR CC0-1.0"
    "Apache-2.0 OR BSD-1-Clause OR MIT"
    "Apache-2.0 OR BSD-2-Clause OR MIT"
    "Apache-2.0 OR BSD-3-Clause OR MIT"
    "Apache-2.0 OR BSD-3-Clause"
    "Apache-2.0 OR BSL-1.0"
    "Apache-2.0 OR CC0-1.0 OR MIT-0"
    "Apache-2.0 OR CC0-1.0"
    "Apache-2.0 OR GPL-3.0"
    "Apache-2.0 OR ISC OR MIT"
    "Apache-2.0 OR MIT OR Zlib"
    "Apache-2.0 OR MIT"
    "Apache-2.0 WITH LLVM-exception"
    "Apache-2.0"
    "BSD-2-Clause"
    "BSD-3-Clause OR MIT"
    "BSD-3-Clause"
    "CC0-1.0"
    "GPL-3.0-only"
    "GPL-3.0-or-later WITH Classpath-exception-2.0"
    "ISC"
    "MIT OR Unlicense"
    "MIT"
    "MPL-2.0"
    "Zlib"
    "Unicode-3.0"
)
AUTHORS=(
    "PureStake"
    "Parity Technologies <admin@parity.io>"
    "Moonsong-Labs"
    "Moonsong Labs"
    "moonbeam-foundation"
)
NAMES=(
    "webpki"
    "webpki-roots"
    "webpki-root-certs"
    "rustls-webpki"
    "ring"
    "nimbus-consensus"
)
licenses_filter=$(printf ' .license != "%s" and' "${LICENSES[@]}")
authors_filter=$(printf ' .authors != "%s" and' "${AUTHORS[@]}")
names_filter=$(printf ' .name != "%s" and' "${NAMES[@]}")
# "true" at the end is necessary to close the "and" conditional
filter="${licenses_filter}${authors_filter}${names_filter} true"

echo -e "checking licenses with filter:\n$filter\n"
RESULT=$(jq "[.[] | select($filter)]" licenses.json)

if [[ "$RESULT" == "[]" ]]; then
    echo "OK !!"
    exit 0
else
    echo -en "$RESULT\n"
    echo "FAILURE !!"
    exit 1
fi
