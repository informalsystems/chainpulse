use std::fmt;

use ibc_proto::{
    google::protobuf::Any,
    ibc::{
        apps::transfer::v1::MsgTransfer,
        core::{
            channel::v1::{
                MsgAcknowledgement, MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit,
                MsgChannelOpenTry, MsgRecvPacket, MsgTimeout, Packet,
            },
            client::v1::{MsgCreateClient, MsgUpdateClient},
        },
    },
};

use prost::Message;

use crate::Result;

#[derive(Clone, Debug)]
pub enum Msg {
    // Client
    CreateClient(MsgCreateClient),
    UpdateClient(MsgUpdateClient),

    // Channel
    RecvPacket(MsgRecvPacket),
    Acknowledgement(MsgAcknowledgement),
    Timeout(MsgTimeout),

    /// Channel handshake
    ChanOpenInit(MsgChannelOpenInit),
    ChanOpenTry(MsgChannelOpenTry),
    ChanOpenAck(MsgChannelOpenAck),
    ChanOpenConfirm(MsgChannelOpenConfirm),

    // Transfer
    Transfer(MsgTransfer),

    // Other
    Other(Any),
}

impl Msg {
    pub fn is_ibc(&self) -> bool {
        if let Self::Other(other) = self {
            other.type_url.starts_with("/ibc")
        } else {
            true
        }
    }

    pub fn is_relevant(&self) -> bool {
        matches!(
            self,
            Self::RecvPacket(_) | Self::Acknowledgement(_) | Self::Timeout(_)
        )
    }

    pub fn packet(&self) -> Option<&Packet> {
        match self {
            Self::RecvPacket(msg) => msg.packet.as_ref(),
            Self::Acknowledgement(msg) => msg.packet.as_ref(),
            Self::Timeout(msg) => msg.packet.as_ref(),
            _ => None,
        }
    }

    pub fn signer(&self) -> Option<&str> {
        match self {
            Self::CreateClient(msg) => Some(&msg.signer),
            Self::UpdateClient(msg) => Some(&msg.signer),
            Self::RecvPacket(msg) => Some(&msg.signer),
            Self::Acknowledgement(msg) => Some(&msg.signer),
            Self::Timeout(msg) => Some(&msg.signer),
            Self::ChanOpenInit(msg) => Some(&msg.signer),
            Self::ChanOpenTry(msg) => Some(&msg.signer),
            Self::ChanOpenAck(msg) => Some(&msg.signer),
            Self::ChanOpenConfirm(msg) => Some(&msg.signer),
            _ => None,
        }
    }

    pub fn decode(msg: Any) -> Result<Self> {
        match msg.type_url.as_str() {
            "/ibc.core.client.v1.MsgCreateClient" => MsgCreateClient::decode(msg.value.as_slice())
                .map(Msg::CreateClient)
                .map_err(Into::into),

            "/ibc.core.client.v1.MsgUpdateClient" => MsgUpdateClient::decode(msg.value.as_slice())
                .map(Msg::UpdateClient)
                .map_err(Into::into),

            "/ibc.core.channel.v1.MsgTimeout" => MsgTimeout::decode(msg.value.as_slice())
                .map(Msg::Timeout)
                .map_err(Into::into),

            "/ibc.core.channel.v1.MsgRecvPacket" => MsgRecvPacket::decode(msg.value.as_slice())
                .map(Msg::RecvPacket)
                .map_err(Into::into),

            "/ibc.core.channel.v1.MsgAcknowledgement" => {
                MsgAcknowledgement::decode(msg.value.as_slice())
                    .map(Msg::Acknowledgement)
                    .map_err(Into::into)
            }

            "/ibc.core.channel.v1.MsgChannelOpenInit" => {
                MsgChannelOpenInit::decode(msg.value.as_slice())
                    .map(Msg::ChanOpenInit)
                    .map_err(Into::into)
            }

            "/ibc.core.channel.v1.MsgChannelOpenTry" => {
                MsgChannelOpenTry::decode(msg.value.as_slice())
                    .map(Msg::ChanOpenTry)
                    .map_err(Into::into)
            }

            "/ibc.core.channel.v1.MsgChannelOpenAck" => {
                MsgChannelOpenAck::decode(msg.value.as_slice())
                    .map(Msg::ChanOpenAck)
                    .map_err(Into::into)
            }

            "/ibc.core.channel.v1.MsgChannelOpenConfirm" => {
                MsgChannelOpenConfirm::decode(msg.value.as_slice())
                    .map(Msg::ChanOpenConfirm)
                    .map_err(Into::into)
            }

            "/ibc.applications.transfer.v1.MsgTransfer" => {
                MsgTransfer::decode(msg.value.as_slice())
                    .map(Msg::Transfer)
                    .map_err(Into::into)
            }

            _ => Ok(Msg::Other(msg)),
        }
    }
}

impl fmt::Display for Msg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Msg::CreateClient(_msg) => {
                write!(f, "CreateClient")
            }

            Msg::UpdateClient(msg) => {
                write!(f, "UpdateClient: {}", msg.client_id)
            }

            Msg::RecvPacket(msg) => {
                let packet = msg.packet.as_ref().unwrap();

                write!(
                    f,
                    "RecvPacket: {} -> {}",
                    packet.source_channel, packet.destination_channel
                )
            }

            Msg::Timeout(msg) => {
                let packet = msg.packet.as_ref().unwrap();

                write!(
                    f,
                    "Timeout: {} -> {}",
                    packet.source_channel, packet.destination_channel
                )
            }

            Msg::Acknowledgement(msg) => {
                let packet = msg.packet.as_ref().unwrap();

                write!(
                    f,
                    "Acknowledgement: {} -> {}",
                    packet.source_channel, packet.destination_channel
                )
            }

            Msg::ChanOpenInit(msg) => {
                write!(f, "ChanOpenInit: {}", msg.port_id)
            }

            Msg::ChanOpenTry(msg) => {
                write!(f, "ChanOpenTry: {}", msg.port_id)
            }

            Msg::ChanOpenAck(msg) => {
                write!(f, "ChanOpenAck: {}/{}", msg.channel_id, msg.port_id)
            }

            Msg::ChanOpenConfirm(msg) => {
                write!(f, "ChanOpenConfirm: {}/{}", msg.channel_id, msg.port_id)
            }

            Msg::Transfer(msg) => {
                write!(f, "Transfer: {}/{}", msg.source_channel, msg.source_port)
            }

            Msg::Other(msg) => {
                write!(f, "Unhandled msg: {}", msg.type_url)
            }
        }
    }
}
