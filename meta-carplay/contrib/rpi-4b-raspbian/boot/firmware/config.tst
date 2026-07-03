###
arm_freq=600
core_freq=100
gpu_freq=100

###
force_turbo=0
over_voltage=-4

###
hdmi_blanking=2
disable_splash=1
disable_overscan=1
hdmi_force_hotplug=0
hdmi_ignore_edid=0xa5000080
hdmi_ignore_hotplug=1

###
dtparam=act_led_trigger=none
dtparam=act_led_activelow=on

###
dtoverlay=disable-wifi
dtoverlay=disable-bt

###
dtparam=audio=off

###
dtoverlay=disable-pcie

###
dtoverlay=dwc2,dr_mode=peripheral
modules-load=dwc2,g_ether
###
gpu_mem=16

###
# avoid_warnings=1
dtoverlay=disable-vc4

#modules-load=dwc2,g_ether
