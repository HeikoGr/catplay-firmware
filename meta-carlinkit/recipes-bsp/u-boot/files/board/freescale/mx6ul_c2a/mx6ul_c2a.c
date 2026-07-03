
#include <common.h>
#include <asm/io.h>
#include <asm/global_data.h>
#include <asm/arch/sys_proto.h>
#include <asm/arch/mx6ul_pins.h>
#include <asm/arch/iomux.h>
#include <asm/arch/clock.h>
#include <asm/arch/imx-regs.h>
#include <asm/arch/crm_regs.h>
#include <usb.h>

DECLARE_GLOBAL_DATA_PTR;

int dram_init(void)
{
	gd->ram_size = /* 128 * 1024 * 1024 */ imx_ddr_size();
	return 0;
}

struct mxc_ccm_reg *_imx_ccm = (struct mxc_ccm_reg *)CCM_BASE_ADDR;

static u32 _decode_pll(u32/*enum pll_clocks*/ pll, u32 infreq)
{
	u32 div, test_div, pll_num, pll_denom;
	u64 temp64;

	switch (pll) {
		case 0/*PLL_SYS*/:
			div = __raw_readl(&_imx_ccm->analog_pll_sys);
			div &= BM_ANADIG_PLL_SYS_DIV_SELECT;
			return (infreq * div) >> 1;
		default:
			return 0;
	}
}

void set_mcu_main_clk(u32 target_hz)
{
	u32 pll_freq, best_podf = 0;
	u32 min_diff = ~0U;

	pll_freq = _decode_pll(0/*PLL_SYS*/, MXC_HCLK);

	for (u32 podf = 0; podf <= 7; ++podf) {
		u32 actual = pll_freq / (podf + 1);
		u32 diff = (actual > target_hz) ? (actual - target_hz) : (target_hz - actual);

		if (diff < min_diff) {
			min_diff = diff;
			best_podf = podf;
		}
	}

	u32 val = __raw_readl(&_imx_ccm->cacrr);
	val &= ~MXC_CCM_CACRR_ARM_PODF_MASK;
	val |= best_podf << MXC_CCM_CACRR_ARM_PODF_OFFSET;
	__raw_writel(val, &_imx_ccm->cacrr);
}

int c2a_clock_boost(void) {
	// TODO: doesn't reach full 900Mhz (needs to adjust divider)
    uint64_t max_hz = get_cpu_speed_grade_hz();
    set_mcu_main_clk(max_hz);
	puts(">>> Applied CPU clock boost\n");
	return 0;
}

int board_init(void)
{
	gd->bd->bi_boot_params = PHYS_SDRAM + 0x100;
    // puts("board_init()\n");
    enable_qspi_clk(0);
	enable_usdhc_clk(1, 0); // WiFi card probing
    // puts("board_init() after qspi clk\n");
	return 0;
}

int board_usb_phy_mode(int port)
{
	if (port == 1)
		return USB_INIT_DEVICE;
	else
		return USB_INIT_HOST;
}

int board_late_init(void)
{
	// puts("board_late_init()\n");
	set_wdog_reset((struct wdog_regs *)WDOG1_BASE_ADDR);
//	do_mx6_showclocks();
	
	// puts("board_late_init() after set_wdog_reset\n");
	return 0;
}

int board_early_init_f(void)
{
	c2a_clock_boost();
	// puts("board_early_init_f()\n");
    return 0;
}

int checkboard(void)
{
	puts("Board: MX6UL[L] C2A\n");
	return 0;
}
