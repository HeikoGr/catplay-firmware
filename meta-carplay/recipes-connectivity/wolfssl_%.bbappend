EXTRA_OECONF += "\
  --enable-opensslextra \
  --enable-opensslall \
  --enable-sessioncerts \
  --enable-altcertchains \
  --enable-certgen \
  --enable-certreq \
  --enable-certext \
  --enable-crl \
  --enable-aesgcm \
  --enable-keygen \
  --enable-dtls \
  --enable-dtls13 \
  --enable-sni \
  --enable-md5 \
  --enable-md4 \
  --enable-cmac \
  --enable-aeskeywrap \
  --enable-ecc \
  --enable-sp \
"

EXTRA_OECONF:remove = "--enable-sp --enable-ecc --enable-sni --enable-dtls --enable-dtls13 --enable-crl --enable-opensslall"
TARGET_CFLAGS += "-DWOLFSSL_DER_LOAD"