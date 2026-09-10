# FFmpeg's ARMv6 inline-assembly probe uses sadd16. Clang 22 accepts that
# probe with -mcpu=arm926ej-s, but correctly rejects the rev16 instruction
# selected later by libavutil/arm/bswap.h. The product CPU is ARMv5TE, so make
# the architectural boundary explicit while retaining ARMv5TE optimizations.
EXTRA_OECONF:append:armv5 = " --disable-armv6 --disable-armv6t2"
