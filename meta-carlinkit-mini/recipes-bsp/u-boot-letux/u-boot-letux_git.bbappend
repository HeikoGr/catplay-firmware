FILESEXTRAPATHS:prepend := "${THISDIR}/files:"

SRC_URI:append = " file://ax1800m.patch"

LETUX_UBOOT_BOARDS = "ax1800_spi_nor ax1800_spi_nor_burner ax1800_spi_nor_stg"
