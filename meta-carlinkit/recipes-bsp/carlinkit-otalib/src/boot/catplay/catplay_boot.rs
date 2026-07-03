use crate::{SystemUtil, boot::catplay::CatplayConfig, dmesg};

pub struct CatPlay(());

impl CatPlay {
    pub fn start(params: &CatplayConfig<'_>) -> Result<(), &'static str> {
        let _ = SystemUtil::unlink_if_exists("/etc/catplay/catplay.conf");
        let _ = SystemUtil::mkdir_if_missing("/etc/catplay");

        let cfg = params.format().map_err(|_| "failed to format catplay config")?;

        let _ = SystemUtil::write_file("/etc/catplay/catplay.conf", cfg.as_str());

        dmesg!("[dmesg] CatPlay forking now");
        let _ = SystemUtil::run_shell("/etc/init.d/catplay start &");

        Ok(())
    }
}
