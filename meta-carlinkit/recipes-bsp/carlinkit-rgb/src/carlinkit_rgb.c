// SPDX-License-Identifier: GPL-2.0-only

#include <linux/led-class-multicolor.h>
#include <linux/leds.h>
#include <linux/module.h>
#include <linux/of_device.h>
#include <linux/property.h>
#include <linux/mutex.h>
#include <linux/version.h>
#include <linux/delay.h>
#include <linux/gpio/driver.h>
#include <linux/platform_device.h>
#include <linux/pinctrl/consumer.h>

#ifdef CONFIG_ARM
#include <asm/arch_timer.h>
#endif

#define RGB_LED_NAME "rgb"

#define GPIO_ADDR 0x0209c000 /* GPIO1 */
#define GPIO_PIN 0x03 /* Pin 3 */

#define GPIO_ADDR_SIZE 0x1000
#define GPIO_GDIR       0x04
#define GPIO_DR         0x00
#define GPIO_BIT(n)    (1U << (n))

#define DEFAULT_T0H_NS 30L
#define DEFAULT_T0L_NS 1220L
#define DEFAULT_T1H_NS 1125UL
#define DEFAULT_T1L_NS 125UL
#define RES_NS 300000UL /* 300 us */

#define XRGB(r, g, b) ((r) << 16 | (g) << 8 | (b))

/* Calibration data */
static short __T0H_NS = 0;
static short __T1H_NS = 0;
static short __T0L_NS = 0;
static short __T1L_NS = 0;

/* Non calibrated data */
static u16 T0H_NS = DEFAULT_T0H_NS;
static u16 T0L_NS = DEFAULT_T0L_NS;
static u16 T1H_NS = DEFAULT_T1H_NS;
static u16 T1L_NS = DEFAULT_T1L_NS;

/* State start */
struct mc_subled subleds[3];
struct led_classdev_mc mcdev;
struct led_init_data init_data = {
	.fwnode = NULL, 
};

struct mutex ledmutex;
void __iomem *reg_base = 0;
/* State end*/

static inline u64 read_cntvct(void) {
	#ifdef CONFIG_ARM
		return __arch_counter_get_cntvct();
	#else
		return 0;
	#endif
}

static inline u32 read_cntfrq(void) {
	return 24000000; // arch_timer_get_cntfrq();
}

static inline void nopdelay(int nops) {
	while (nops-- > 0) {
		asm volatile("nop");
	}
}

static int calibrate_ndelay_arg(u32 target_ns) {
	unsigned long flags;
	local_irq_save(flags);
	preempt_disable();

	u64 freq = read_cntfrq(); // Hz

	u64 best_diff = ~0ULL;
	u32 best_i = 0;

	for (u32 i = 0; i <= 2000; i++) {
		u64 t1 = read_cntvct();
		nopdelay(i);
		u64 t2 = read_cntvct();

		u64 delta_ticks = t2 - t1;
		u64 delta_ns = div64_u64(delta_ticks * 1000000000ULL, freq);
		u64 diff = delta_ns > target_ns ? delta_ns - target_ns : target_ns - delta_ns;

		if (diff < best_diff && delta_ns > target_ns) {
			best_diff = diff;
			best_i = i;
		}
	}

	preempt_enable();
	local_irq_restore(flags);

	pr_info("ws2812b: nopdelay(%u) ~= %u ns (delta = %llu ns)\n", best_i, target_ns, best_diff);
	return best_i;
}

static void ws2812b_bitbang_crit(void __iomem *reg, u32 flip_val, u32 xrgb) {
	const short th_table[2] = { __T0H_NS, __T1H_NS };
	const short tl_table[2] = { __T0L_NS, __T1L_NS };

	u32 val = readl(reg);
	register u32 val1 = val | flip_val;
	register u32 val0 = val & ~flip_val;

	register u8 bit;

	#define push_bit(b, m) do { \
		bit = !!(b & m); \
		__raw_writel(val1, reg); /* 1 */ \
		nopdelay(th_table[bit]); \
		 __raw_writel(val0, reg); /* 0 */ \
		nopdelay(tl_table[bit]); \
	} while (0)

	// GREEN (bits 15..8)
	push_bit(xrgb, 1 << 15); push_bit(xrgb, 1 << 14); push_bit(xrgb, 1 << 13); push_bit(xrgb, 1 << 12);
	push_bit(xrgb, 1 << 11); push_bit(xrgb, 1 << 10); push_bit(xrgb, 1 << 9);  push_bit(xrgb, 1 << 8);

	// RED (bits 23..16)
	push_bit(xrgb, 1 << 23); push_bit(xrgb, 1 << 22); push_bit(xrgb, 1 << 21); push_bit(xrgb, 1 << 20);
	push_bit(xrgb, 1 << 19); push_bit(xrgb, 1 << 18); push_bit(xrgb, 1 << 17); push_bit(xrgb, 1 << 16);

	// BLUE (bits 7..0)
	push_bit(xrgb, 1 << 7);  push_bit(xrgb, 1 << 6);  push_bit(xrgb, 1 << 5);  push_bit(xrgb, 1 << 4);
	push_bit(xrgb, 1 << 3);  push_bit(xrgb, 1 << 2);  push_bit(xrgb, 1 << 1);  push_bit(xrgb, 1 << 0);
}

static void ws2812b_bitbang(void __iomem *reg_base, u32 xrgb) {
	unsigned long flags;

	u32 flip_val = GPIO_BIT(GPIO_PIN);
	
	local_irq_save(flags);
	preempt_disable();

	pr_info("ws2812b_bitbang: reg_base %p, data %08X!\n", reg_base, xrgb);
	ws2812b_bitbang_crit(reg_base + GPIO_DR, flip_val, xrgb);

	preempt_enable();
	local_irq_restore(flags);

	udelay(RES_NS / 1000);
}

static int ws2812b_set(struct led_classdev *cdev, enum led_brightness brightness) {
	struct led_classdev_mc *mc_cdev = lcdev_to_mccdev(cdev);
	if (!mc_cdev) {
		pr_err("ws2812b_set: mc_cdev is NULL!\n");
		return -ENODEV;
	}

	led_mc_calc_color_components(mc_cdev, brightness);

	mutex_lock(&ledmutex);

	u32 xrgb = XRGB(
		subleds[0].brightness,
		subleds[1].brightness,
		subleds[2].brightness
	);

	ws2812b_bitbang(reg_base, xrgb);

	mutex_unlock(&ledmutex);

	return 0;
}

static int ws2812b_probe(struct platform_device *pdev) {
	int ret;
	struct pinctrl *pinctrl;

	pr_info("ws2812b: loading Carlinkit C2A LED RGB driver\n");

	ret = devm_mutex_init(&pdev->dev, &ledmutex);
	if (ret)
		return ret;

	u32 color_idx[3] = {
		LED_COLOR_ID_RED,
		LED_COLOR_ID_GREEN,
		LED_COLOR_ID_BLUE,
	};

	for (int i = 0; i < 3; i++) {
		subleds[i].color_index = color_idx[i];
		subleds[i].intensity = 255;
	}

	mcdev.subled_info = subleds;
	mcdev.num_colors = 3;
	mcdev.led_cdev.max_brightness = 255;
	mcdev.led_cdev.brightness_set_blocking = ws2812b_set;

	mcdev.led_cdev.name = RGB_LED_NAME;

	__T0H_NS = calibrate_ndelay_arg(T0H_NS);
	__T0L_NS = calibrate_ndelay_arg(T0L_NS);
	__T1H_NS = calibrate_ndelay_arg(T1H_NS);
	__T1L_NS = calibrate_ndelay_arg(T1L_NS);

	if (__T0H_NS < 0 || __T0L_NS < 0 || __T1H_NS < 0 || __T1L_NS < 0) {
		return -EINVAL;
	}

	void __iomem *gpio1 = ioremap(GPIO_ADDR, GPIO_ADDR_SIZE);
	if (!gpio1)
		return -ENOMEM;

	pinctrl = devm_pinctrl_get_select_default(&pdev->dev);
	if (IS_ERR(pinctrl))
		dev_warn(&pdev->dev, "ws2812b: pinctrl default state not applied: %d\n", ret);

	// Mode: output
	writel(readl(gpio1 + GPIO_GDIR) | GPIO_BIT(GPIO_PIN), gpio1 + GPIO_GDIR);

	// Set to LOW
	writel(readl(gpio1 + GPIO_DR) & ~GPIO_BIT(GPIO_PIN), gpio1 + GPIO_DR);

	reg_base = gpio1;

	init_data.fwnode = dev_fwnode(&pdev->dev);
	ret = devm_led_classdev_multicolor_register_ext(
		&pdev->dev, &mcdev, &init_data);
	if (ret) {
		dev_err(&pdev->dev, "ws2812b: RGB LED registration failed\n");
		return ret;
	}

	return 0;
}

static const struct of_device_id ws2812b_dt_ids[] = {
	{ .compatible = "carlinkit,ws2812b" },
	{},
};
MODULE_DEVICE_TABLE(of, ws2812b_dt_ids);

static struct platform_driver ws2812b_driver = {
	.probe		= ws2812b_probe,
	.driver = {
		.name		= KBUILD_MODNAME,
		.of_match_table	= ws2812b_dt_ids,
	},
};

module_param_named(t0h, T0H_NS, ushort, 0644);
MODULE_PARM_DESC(t0h, "T0H");

module_param_named(t0l, T0L_NS, ushort, 0644);
MODULE_PARM_DESC(t0l, "T0L");

module_param_named(t1h, T1H_NS, ushort, 0644);
MODULE_PARM_DESC(t1h, "T1H");

module_param_named(t1l, T1L_NS, ushort, 0644);
MODULE_PARM_DESC(t1l, "T1L");

module_platform_driver(ws2812b_driver);

MODULE_AUTHOR("CatPlay");
MODULE_DESCRIPTION("WS2812B LED driver - high frequency GPIO bitbang for Carlinkit devices");
MODULE_LICENSE("GPL");
