#!/bin/sh

### BEGIN INIT INFO
# Provides:          catplay
# Required-Start:    $syslog $local_fs messagebus
# Required-Stop:     
# Default-Start:     2 3 4 5
# Default-Stop:      0 1 6
# Short-Description: CatPlay service
# Description:       CarPlay headunit daemon
### END INIT INFO

. /etc/init.d/functions

catplay_bin=/usr/bin/catplay_c2a
pidfile=/var/run/catplay.pid
logfile=/var/log/catplay.log

case "$1" in
    start)
        echo -n "Starting catplay: "
        start-stop-daemon --start --quiet \
            --background \
            --make-pidfile --pidfile "$pidfile" \
            --startas /bin/sh -- -c "$catplay_bin >>'$logfile' 2>&1"
        echo "."
        ;;
    stop)
        echo -n "Stopping catplay: "
        start-stop-daemon --stop --quiet --pidfile "$pidfile"
        echo "."
        ;;
    restart)
        echo -n "Stopping catplay: "
        start-stop-daemon --stop --quiet --pidfile "$pidfile"
        echo "."
        echo -n "Starting catplay: "
        start-stop-daemon --start --quiet \
            --background \
            --make-pidfile --pidfile "$pidfile" \
            --startas /bin/sh -- -c "$catplay_bin >>'$logfile' 2>&1"
        echo "."
        ;;
    reload)
        echo -n "Reloading catplay: "
        # SIGHUP maybe
        echo "......................."
        ;;
    status)
        status_of_proc -p "$pidfile" "$catplay_bin" catplay && exit 0 || exit $?
        ;;
    *)
        echo "Usage: /etc/init.d/catplay {start|stop|restart|reload|status}" >&2
        exit 1
        ;;
esac

exit 0
