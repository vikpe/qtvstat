//! UDP related operations
use std::time::Duration;

use anyhow::{anyhow as e, Result};
use quake_qtvinfo::Qtvinfo;

/// Get info from given QTV address
pub fn info(address: &str, timeout: Option<Duration>) -> Result<Qtvinfo> {
    let status87_command = [vec![255; 4], b"status 87".to_vec()].concat();
    let options = tinyudp::ReadOptions {
        timeout,
        ..Default::default()
    };

    let Ok(response) = tinyudp::send_and_read(address, &status87_command, &options) else {
        return Err(e!("qtvstat::info: unable to get status 87 from {address}"));
    };

    parse_status87_response(&response)
}

fn parse_status87_response(response: &[u8]) -> Result<Qtvinfo> {
    const HEADER: &[u8; 5] = &[255, 255, 255, 255, b'n'];

    if !response.starts_with(HEADER) {
        return Err(e!("header is missing"));
    }

    let info_str = String::from_utf8_lossy(&response[HEADER.len()..]);
    Ok(Qtvinfo::from(info_str.as_ref()))
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use quake_qtvinfo::Qtvinfo;

    use super::*;

    #[test]
    fn test_parse_status87_response() -> Result<()> {
        // invalid response
        assert_eq!(
            parse_status87_response(br#"\invalid\response"#)
                .unwrap_err()
                .to_string(),
            "header is missing"
        );

        // valid response
        {
            let response = [
                vec![255; 4],
                br#"n\*version\QTV 1.14\maxclients\100\hostname\QUAKE.SE KTX Qtv"#.to_vec(),
            ]
            .concat();

            assert_eq!(
                parse_status87_response(&response)?,
                Qtvinfo {
                    hostname: Some("QUAKE.SE KTX Qtv".to_string()),
                    maxclients: Some(100),
                    version: Some("QTV 1.14".to_string()),
                }
            );
        }

        Ok(())
    }
}
