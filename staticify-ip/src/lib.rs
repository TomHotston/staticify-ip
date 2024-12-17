use mockall::predicate::*;
use mockall::*;
use std::net::IpAddr;

#[automock]
pub trait IpGetter {
    fn get_public_ip(&self) -> Option<IpAddr>;
    fn get_last_ip(&self) -> Option<IpAddr>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn get_public_ip_handles_vaild_ip_addresses() {
        let mut mock = MockIpGetter::new();
        mock.expect_get_public_ip()
            .returning(|| "127.0.0.1".parse::<IpAddr>().ok());
        let localhost_v4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        assert_eq!(mock.get_public_ip().unwrap(), localhost_v4);
    }

    #[test]
    #[should_panic]
    fn get_public_ip_handles_invalid_ip_addresses() {
        let mut mock = MockIpGetter::new();
        mock.expect_get_public_ip()
            .returning(|| "".parse::<IpAddr>().ok());
        mock.get_public_ip().unwrap();
    }

    #[test]
    fn get_last_ip_handles_vaild_ip_addresses() {
        let mut mock = MockIpGetter::new();
        mock.expect_get_last_ip()
            .returning(|| "127.0.0.1".parse::<IpAddr>().ok());
        let localhost_v4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        assert_eq!(mock.get_last_ip().unwrap(), localhost_v4);
    }

    #[test]
    #[should_panic]
    fn get_last_ip_handles_invalid_ip_addresses() {
        let mut mock = MockIpGetter::new();
        mock.expect_get_last_ip()
            .returning(|| "".parse::<IpAddr>().ok());
        mock.get_last_ip().unwrap();
    }
}
