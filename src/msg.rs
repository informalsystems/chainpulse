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
use tracing::info;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

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
}

pub fn decode_msg(msg: Any) -> Result<Msg> {
    match msg.type_url.as_str() {
        "/ibc.core.client.v1.MsgCreateClient" => MsgCreateClient::decode(msg.value.as_slice())
            .map(Msg::CreateClient)
            .map_err(Into::into),

        "/ibc.core.client.v1.MsgUpdateClient" => MsgUpdateClient::decode(msg.value.as_slice())
            .map(Msg::UpdateClient)
            .map_err(Into::into),

        "/ibc.core.channel.v1.Timeout" => MsgTimeout::decode(msg.value.as_slice())
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

        "/ibc.core.channel.v1.MsgChannelOpenTry" => MsgChannelOpenTry::decode(msg.value.as_slice())
            .map(Msg::ChanOpenTry)
            .map_err(Into::into),

        "/ibc.core.channel.v1.MsgChannelOpenAck" => MsgChannelOpenAck::decode(msg.value.as_slice())
            .map(Msg::ChanOpenAck)
            .map_err(Into::into),

        "/ibc.core.channel.v1.MsgChannelOpenConfirm" => {
            MsgChannelOpenConfirm::decode(msg.value.as_slice())
                .map(Msg::ChanOpenConfirm)
                .map_err(Into::into)
        }

        "/ibc.applications.transfer.v1.MsgTransfer" => MsgTransfer::decode(msg.value.as_slice())
            .map(Msg::Transfer)
            .map_err(Into::into),

        _ => Ok(Msg::Other(msg)),
    }
}

pub fn print_msg(msg: &Msg) {
    match msg {
        Msg::CreateClient(_msg) => {
            info!("CreateClient");
        }

        Msg::UpdateClient(msg) => {
            info!("UpdateClient: {}", msg.client_id);
        }

        Msg::RecvPacket(msg) => {
            let packet = msg.packet.as_ref().unwrap();

            info!(
                "RecvPacket: {} -> {}",
                packet.source_channel, packet.destination_channel
            );
        }

        Msg::Timeout(msg) => {
            let packet = msg.packet.as_ref().unwrap();

            info!(
                "Timeout: {} -> {}",
                packet.source_channel, packet.destination_channel
            );
        }

        Msg::Acknowledgement(msg) => {
            let packet = msg.packet.as_ref().unwrap();

            info!(
                "Acknowledgement: {} -> {}",
                packet.source_channel, packet.destination_channel
            );
        }

        Msg::ChanOpenInit(msg) => {
            info!("ChanOpenInit: {}", msg.port_id);
        }

        Msg::ChanOpenTry(msg) => {
            info!("ChanOpenTry: {}", msg.port_id);
        }

        Msg::ChanOpenAck(msg) => {
            info!("ChanOpenAck: {}/{}", msg.channel_id, msg.port_id);
        }

        Msg::ChanOpenConfirm(msg) => {
            info!("ChanOpenConfirm: {}/{}", msg.channel_id, msg.port_id);
        }

        Msg::Transfer(msg) => {
            info!("Transfer: {}/{}", msg.source_channel, msg.source_port);
        }

        Msg::Other(msg) => {
            info!("Unhandled msg: {}", msg.type_url);
        }
    }
}
