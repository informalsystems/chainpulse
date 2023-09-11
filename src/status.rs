use std::{cmp::Reverse, time::Duration};

use serde::Deserialize;
use tokio::time::sleep;
use tracing::info;

use crate::{config::Chains, metrics::Metrics, Result};

const STATUS_URL: &str = "https://api-osmosis.imperator.co/ibc/v1/raw";

pub async fn run(chains: Chains, metrics: Metrics) -> Result<()> {
    loop {
        let Ok(status) = fetch_status().await else {
            sleep(Duration::from_secs(120)).await;
            continue;
        };

        let mut stuck = Vec::new();

        for chain_id in chains.endpoints.keys() {
            stuck.extend(
                status
                    .by_chain(chain_id.as_str())
                    .filter(|channel| channel.status.size_queue > 0),
            );
        }

        stuck.sort_by_key(|channel| Reverse(channel.status.size_queue));

        info!("IBC packets are stuck on {} channels:", stuck.len());

        for channel in stuck {
            metrics.ibc_stuck_packets(
                channel.src_chain.as_str(),
                channel.dst_chain.as_str(),
                channel.src_channel.as_str(),
                channel.status.size_queue,
            );

            info!(
                "{} [{}] --> {}: {}",
                channel.src_chain,
                channel.src_channel,
                channel.dst_chain,
                channel.status.size_queue
            );
        }

        sleep(Duration::from_secs(60)).await;
    }
}

pub async fn fetch_status() -> Result<IbcStatus> {
    let resp = reqwest::get(STATUS_URL).await.unwrap();
    let body = resp.text().await.unwrap();
    let status = serde_json::from_str(&body)?;
    Ok(status)
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct IbcStatus(Vec<ChannelStatus>);

impl IbcStatus {
    pub fn channels(&self) -> impl Iterator<Item = &ChannelStatus> {
        self.0.iter()
    }

    pub fn by_chain<'a>(&'a self, chain: &'a str) -> impl Iterator<Item = &'a ChannelStatus> {
        self.0
            .iter()
            .filter(move |status| status.src_chain == chain || status.dst_chain == chain)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelStatus {
    pub src_chain: String,
    pub dst_chain: String,
    pub src_channel: String,
    pub status: Status,
}

impl ChannelStatus {
    fn from_desc(desc: &str, status: Status) -> Result<ChannelStatus> {
        let (src_chain, src_channel, dst_chain) = parse_desc(desc)?;

        Ok(Self {
            src_chain: src_chain.to_string(),
            dst_chain: dst_chain.to_string(),
            src_channel: src_channel.to_string(),
            status,
        })
    }
}

impl<'de> Deserialize<'de> for ChannelStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // deserialize a map of a single key to a `Status`
        let map = <std::collections::HashMap<String, Status>>::deserialize(deserializer)?;
        let (desc, status) = map.into_iter().next().ok_or_else(|| {
            serde::de::Error::custom("expected a map with a single key-value pair")
        })?;

        Self::from_desc(&desc, status).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Status {
    pub name: String,
    pub token_name: String,
    pub last_tx: String,
    pub counter: i64,
    pub size_queue: i64,
    pub is_trigger: bool,
}

fn parse_desc(desc: &str) -> Result<(&str, &str, &str)> {
    let (left, right) = desc.split_once(" --> ").ok_or("missing arrow")?;

    fn extract_chain_and_channel(s: &str) -> Result<(&str, Option<&str>)> {
        let mut parts = s.split_whitespace();
        if s.contains('(') {
            let _ = parts.next(); // skip the first part
        }
        let chain = parts
            .next()
            .ok_or("missing chain")?
            .trim_matches(|c| c == '(' || c == ')');

        let channel = parts
            .next()
            .map(|c| c.trim_matches(|c| c == '[' || c == ']'));

        Ok((chain, channel))
    }

    let (src_chain, Some(src_channel)) =
        extract_chain_and_channel(left).map_err(|_| "missing source chain or channel")?
    else {
        return Err("missing source channel".into());
    };

    let (dst_chain, _) =
        extract_chain_and_channel(right).map_err(|_| "missing destination chain")?;

    Ok((src_chain, src_channel, dst_chain))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_desc1() {
        let desc = "iov-mainnet-ibc [channel-2] --> osmosis-1";
        let (src_chain, src_channel, dst_chain) = parse_desc(desc).unwrap();
        assert_eq!(src_chain, "iov-mainnet-ibc");
        assert_eq!(src_channel, "channel-2");
        assert_eq!(dst_chain, "osmosis-1");
    }

    #[test]
    fn test_parse_desc2() {
        let desc = "osmosis-1 [channel-169] --> neta (juno-1)";
        let (src_chain, src_channel, dst_chain) = parse_desc(desc).unwrap();
        assert_eq!(src_chain, "osmosis-1");
        assert_eq!(src_channel, "channel-169");
        assert_eq!(dst_chain, "juno-1");
    }

    #[test]
    fn test_parse_desc3() {
        let desc = "foobar (osmosis-1) [channel-169] --> juno-1";
        let (src_chain, src_channel, dst_chain) = parse_desc(desc).unwrap();
        assert_eq!(src_chain, "osmosis-1");
        assert_eq!(src_channel, "channel-169");
        assert_eq!(dst_chain, "juno-1");
    }

    #[test]
    fn test_parse_desc4() {
        let desc = "foobar (osmosis-1) [channel-169] --> neta (juno-1)";
        let (src_chain, src_channel, dst_chain) = parse_desc(desc).unwrap();
        assert_eq!(src_chain, "osmosis-1");
        assert_eq!(src_channel, "channel-169");
        assert_eq!(dst_chain, "juno-1");
    }
}
