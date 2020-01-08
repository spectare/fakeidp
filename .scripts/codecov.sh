#!/bin/bash

REPORT=$(find target/debug -maxdepth 1 -name 'oidc-token-mock*' -a ! -name '*.d')

for file in $REPORT; do
    mkdir -p "target/cov/$(basename $file)"
    sudo kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/$(basename $file)" "$file"
done

wget -O - -q "https://codecov.io/bash" > .codecov
chmod +x .codecov
./.codecov -t $CODECOV_TOKEN
echo "Uploaded code coverage"
