# This product does not ship OpenSSL ptests. Disable the ptest tasks as well as
# the tests themselves; forcing no-tests while PTEST_ENABLED=1 leaves the
# install task looking for binaries that were deliberately not built.
PTEST_ENABLED = "0"

# OpenSSL 3.5 models these as positive tls1/tls1_1 features. Keeping them
# disabled makes the recipe pass no-tls1/no-tls1_1 to Configure.
