use anyhow::Result;
use mockall::predicate::*;
use mockall::*;
use std::net::IpAddr;

#[automock]
trait IpReconfigurer {
    fn get_public_ip(&self) -> Result<IpAddr>;
    fn get_last_ip(&self) -> Result<IpAddr>;
    fn reconfigure_public_ip(&self);
    fn readback_configured_ip(&self) -> Result<IpAddr>;
}

fn compare_ips<A>(ip_getter: A) -> Result<()>
where
    A: IpReconfigurer,
{
    let public_ip = ip_getter.get_public_ip()?;
    let last_ip = ip_getter.get_last_ip()?;
    if public_ip != last_ip {
        ip_getter.reconfigure_public_ip();
        if ip_getter.readback_configured_ip()? != public_ip {
            panic!("IP addresses do not match after reconfiguration");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_ips_handles_vaild_ip_addresses_that_match() {
        let mut mock = MockIpReconfigurer::new();
        mock.expect_get_public_ip()
            .returning(|| Ok("127.0.0.1".parse::<IpAddr>()?));
        mock.expect_get_last_ip()
            .returning(|| Ok("127.0.0.1".parse::<IpAddr>()?));
        compare_ips(mock).unwrap();
    }

    #[test]
    #[should_panic]
    fn compare_ips_handles_invalid_public_ip_address() {
        let mut mock = MockIpReconfigurer::new();
        mock.expect_get_public_ip()
            .returning(|| Ok("".parse::<IpAddr>()?));
        mock.expect_get_last_ip()
            .returning(|| Ok("127.0.0.1".parse::<IpAddr>()?));
        compare_ips(mock).unwrap();
    }

    #[test]
    #[should_panic]
    fn compare_ips_handles_invalid_last_ip_address() {
        let mut mock = MockIpReconfigurer::new();
        mock.expect_get_last_ip()
            .returning(|| Ok("".parse::<IpAddr>()?));
        mock.expect_get_public_ip()
            .returning(|| Ok("127.0.0.1".parse::<IpAddr>()?));
        compare_ips(mock).unwrap();
    }

    #[test]
    fn compare_ips_handles_valid_ip_addresses_that_do_not_match_but_reconfigure_correctly() {
        let mut mock = MockIpReconfigurer::new();
        mock.expect_get_public_ip()
            .returning(|| Ok("127.0.0.2".parse::<IpAddr>()?));
        mock.expect_get_last_ip()
            .returning(|| Ok("127.0.0.1".parse::<IpAddr>()?));
        mock.expect_reconfigure_public_ip().return_const(());
        mock.expect_readback_configured_ip()
            .returning(|| Ok("127.0.0.2".parse::<IpAddr>()?));
        compare_ips(mock).unwrap();
    }

    #[test]
    #[should_panic]
    fn compare_ips_handles_valid_ip_addresses_that_do_not_match_and_reconfigure_incorrectly() {
        let mut mock = MockIpReconfigurer::new();
        mock.expect_get_public_ip()
            .returning(|| Ok("127.0.0.2".parse::<IpAddr>()?));
        mock.expect_get_last_ip()
            .returning(|| Ok("127.0.0.1".parse::<IpAddr>()?));
        mock.expect_reconfigure_public_ip().return_const(());
        mock.expect_readback_configured_ip()
            .returning(|| Ok("127.0.0.1".parse::<IpAddr>()?));
        compare_ips(mock).unwrap();
    }
}
