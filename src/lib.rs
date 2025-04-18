use anyhow::Result;
use mockall::predicate::*;
use mockall::*;
use std::net::IpAddr;

struct Invalid;
struct Valid;
struct ServerIp<State = Invalid> {
    state: std::marker::PhantomData<State>,
    current_ip: Option<IpAddr>,
    previous_ip: Option<IpAddr>,
}
enum ValidationResult {
    Valid(ServerIp<Valid>),
    Invalid(ServerIp<Invalid>),
}

#[automock]
trait Refreshable {
    fn get_public_ip(&mut self) -> Option<IpAddr>;
    fn get_configured_ip(&mut self) -> Option<IpAddr>;
}

#[automock]
trait Validatable<T> {
    fn validate(self) -> T;
}

#[automock]
trait Reconfigurable {
    fn reconfigure_ip(&self);
}

#[automock]
trait Storable {
    fn store_ip(&self);
}

impl Refreshable for ServerIp<Invalid> {
    fn get_public_ip(&mut self) -> Option<IpAddr> {
        todo!()
    }

    fn get_configured_ip(&mut self) -> Option<IpAddr> {
        todo!()
    }
}

impl<T> Validatable<T> for ServerIp<Invalid> {
    fn validate(self) -> T{
        todo!()
    }
}

impl Reconfigurable for ServerIp<Invalid> {
    fn reconfigure_ip(&self) {
        todo!()
    }
}

impl Storable for ServerIp<Valid> {
    fn store_ip(&self) {
        todo!()
    }
}

impl ServerIp {
    fn new() -> Self {
        Self {
            state: Default::default(),
            current_ip: None,
            previous_ip: None,
        }
    }
}

fn configure_server_ip<A, B>(mut server_ip: A) -> Result<()>
where
    A: Refreshable + Validatable<B>,
{
    server_ip.get_public_ip();
    server_ip.get_configured_ip();
    match server_ip.validate() {
        ValidationResult::Valid(valid) => {
            valid.store_ip();
            Ok(())
        }
        ValidationResult::Invalid(mut invalid) => {
            invalid.reconfigure_ip();
            invalid.get_configured_ip();
            match invalid.validate() {
                ValidationResult::Valid(valid) => {
                    valid.store_ip();
                    Ok(())
                }
                ValidationResult::Invalid(_) => panic!("IP could not be reassigned"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mock! {
        SIp {}
        impl Refreshable for SIp {
            fn get_public_ip(&mut self) -> Option<IpAddr>;
            fn get_configured_ip(&mut self) -> Option<IpAddr>;
        }

        impl Validatable for SIp {
            fn validate(self) -> ValidationResult;
        }

        impl Reconfigurable for SIp {
            fn reconfigure_ip(&self);
        }

        impl Storable for SIp {
            fn store_ip(&self);
        }
    }

    #[test]
    fn compare_ips_handles_vaild_ip_addresses_that_match() {
        let mut mock = MockSIp::new();
        mock.expect_get_public_ip()
            .returning(|| "127.0.0.1".parse().ok());
        mock.expect_get_configured_ip()
            .returning(|| "127.0.0.1".parse().ok());
        mock.expect_validate().returning(|| ValidationResult::Valid(SIp<Valid>));
        configure_server_ip(mock).unwrap();
    }

    // #[test]
    // #[should_panic]
    // fn compare_ips_handles_invalid_public_ip_address() {
    //     let mut mock = MockReconfigurable::new();
    //     mock.expect_get_public_ip()
    //         .returning(|| Ok("".parse::<IpAddr>()?));
    //     mock.expect_get_last_ip()
    //         .returning(|| Ok("127.0.0.1".parse::<IpAddr>()?));
    //     configure_server_ip(mock).unwrap();
    // }

    // #[test]
    // #[should_panic]
    // fn compare_ips_handles_invalid_last_ip_address() {
    //     let mut mock = MockReconfigurable::new();
    //     mock.expect_get_last_ip()
    //         .returning(|| Ok("".parse::<IpAddr>()?));
    //     mock.expect_get_public_ip()
    //         .returning(|| Ok("127.0.0.1".parse::<IpAddr>()?));
    //     configure_server_ip(mock).unwrap();
    // }

    // #[test]
    // fn compare_ips_handles_valid_ip_addresses_that_do_not_match_but_reconfigure_correctly() {
    //     let mut mock = MockReconfigurable::new();
    //     mock.expect_get_public_ip()
    //         .returning(|| Ok("127.0.0.2".parse::<IpAddr>()?));
    //     mock.expect_get_last_ip()
    //         .returning(|| Ok("127.0.0.1".parse::<IpAddr>()?));
    //     mock.expect_reconfigure_public_ip()
    //         .with(eq("127.0.0.2".parse::<IpAddr>().unwrap()))
    //         .return_const(());
    //     mock.expect_readback_configured_ip()
    //         .returning(|| Ok("127.0.0.2".parse::<IpAddr>()?));
    //     configure_server_ip(mock).unwrap();
    // }

    // #[test]
    // #[should_panic]
    // fn compare_ips_handles_valid_ip_addresses_that_do_not_match_and_reconfigure_incorrectly() {
    //     let mut mock = MockReconfigurable::new();
    //     mock.expect_get_public_ip()
    //         .returning(|| Ok("127.0.0.2".parse::<IpAddr>()?));
    //     mock.expect_get_last_ip()
    //         .returning(|| Ok("127.0.0.1".parse::<IpAddr>()?));
    //     mock.expect_reconfigure_public_ip()
    //         .with(eq("127.0.0.2".parse::<IpAddr>().unwrap()))
    //         .return_const(());
    //     mock.expect_readback_configured_ip()
    //         .returning(|| Ok("127.0.0.1".parse::<IpAddr>()?));
    //     configure_server_ip(mock).unwrap();
    // }
}
