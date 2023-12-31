use std::time::{Duration, Instant};

pub fn http(method: String, url: String) -> bool {
    if url.is_empty() {
        log::warn!("readiness probe url is not presented");

        return false;
    }

    let timeout = Duration::from_secs(1);
    let now = Instant::now();
    let agent = ureq::builder().timeout_connect(timeout).build();
    let response = agent.request(method.to_uppercase().as_str(), &url).call();
    let took = now.elapsed().as_millis();

    match response {
        Err(err) => {
            log::warn!(
                "request {} {} FAILED, {}, took {} ms",
                method.to_uppercase(),
                url,
                err.to_string(),
                took
            );

            false
        }
        Ok(value) => {
            let status = value.status();

            if (200..=299).contains(&status) {
                log::debug!(
                    "request {} {} OK, status: {}, took {} ms",
                    method.to_uppercase(),
                    url,
                    status,
                    took
                );

                true
            } else {
                log::warn!(
                    "request {} {} OK, status: {}, took {} ms",
                    method.to_uppercase(),
                    url,
                    status,
                    took
                );

                false
            }
        }
    }
}
