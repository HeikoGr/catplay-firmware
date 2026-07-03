### BEGIN INIT INFO
# Provides:          carlinkit-init
# Required-Start:    $remote_fs $syslog
# Required-Stop:     $remote_fs $syslog
# Default-Start:     2 3 4 5
# Default-Stop:      0 1 6
# Short-Description: Start my custom service
# Description:       My custom service daemon
### END INIT INFO

case "$1" in
  start)
    echo "Starting carlinkit-mini-init"
    cd /usr/lib/carlinkit-mini-init && ./init.sh &> /var/log/carlinkit-init.log
    ;;
  stop)
    echo "Stopping carlinkit-mini-init"
    #killall mybinary
    ;;
  restart)
    $0 stop
    $0 start
    ;;
  *)
    echo "Usage: $0 {start|stop|restart}"
    exit 1
esac

exit 0
