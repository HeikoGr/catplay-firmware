#!/bin/sh

### BEGIN INIT INFO
# Provides:          c2a-overlay-init
# Required-Start:    $syslog $local_fs messagebus
# Required-Stop:     
# Default-Start:     2 3 4 5
# Default-Stop:      0 1 6
# Short-Description: C2A overlays
# Description:       C2A overlays
### END INIT INFO

. /etc/init.d/functions

case "$1" in
    start)
        echo "Mounting C2A overlays" > /dev/kmsg

        # hostname.sh
        read HOSTNAME < /etc/hostname
        hostname "$HOSTNAME"
        
        # devpts.sh
        #. /etc/default/devpts
        #mknod -m 666 /dev/ptmx c 5 2
		mkdir -p /dev/pts
		mount -t devpts devpts /dev/pts -ogid=5,mode=620

        # sysfs.sh
        mount -t proc proc /proc
        mount -t sysfs sysfs /sys
        #mount -t debugfs debugfs /sys/kernel/debug
        mount -t configfs configfs /sys/kernel/config
        #mount -n -t devtmpfs devtmpfs /dev

        # alignment.sh
        echo "3" > /proc/cpu/alignment

        # c2a-overlay-init.sh

        # This one will work in booting in initramfs mode but isn't strictly needed anymore
        mount -o remount,rw / &>/dev/null

        mount -t tmpfs tmpfs /run

        mkdir -p /run/upper /run/work /run/newroot
        mount -t overlay overlay \
            -o lowerdir=/,upperdir=/run/upper,workdir=/run/work \
            /run/newroot
        mkdir -p /run/newroot/old_root
        echo "Performing pivot_root" > /dev/kmsg
        pivot_root /run/newroot /run/newroot/old_root
        mount --move /old_root/dev /dev
        mount --move /old_root/sys /sys
        mount --move /old_root/proc /proc

        #mount --move /old_root/var/lib /var/lib & 
        #mount --move /old_root/var/volatile /var/volatile &
        wait
        
        mkdir -p /var/volatile/lib
        mkdir -p /var/volatile/log
        mkdir -p /var/volatile/tmp

        # dmesg.sh
        if [ -f /var/log/dmesg ]; then
            if LOGPATH=$(which logrotate); then
                $LOGPATH -f /etc/logrotate-dmesg.conf
            else
                mv -f /var/log/dmesg /var/log/dmesg.old
            fi
        fi
        dmesg -s 131072 > /var/log/dmesg

        echo "Mounted C2A overlays!" > /dev/kmsg

        ;;
    stop)
        ;;
    restart)
        ;;
    reload)
        ;;
    status)
        ;;
    *)
        echo "Usage: /etc/init.d/c2a-overlay-init {start|stop|restart|reload|status}" >&2
        exit 1
        ;;
esac
