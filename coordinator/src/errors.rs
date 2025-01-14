use std::fmt::Debug;
use std::io;

use datafusion::arrow::error::ArrowError;
use datafusion::error::DataFusionError;
use flatbuffers::InvalidFlatbuffer;
use meta::error::MetaError;
use models::error_code::{ErrorCode, ErrorCoder};
use models::meta_data::{ReplicationSet, ReplicationSetId};
use models::schema::Precision;
use models::Timestamp;
use protos::PointsError;
use snafu::Snafu;

#[derive(Snafu, Debug, ErrorCoder)]
#[snafu(visibility(pub))]
#[error_code(mod_code = "05")]
pub enum CoordinatorError {
    TskvError {
        source: tskv::TskvError,
    },

    Meta {
        source: MetaError,
    },

    ArrowError {
        source: ArrowError,
    },

    DataFusionError {
        source: DataFusionError,
    },

    ReplicatError {
        source: replication::errors::ReplicationError,
    },

    #[snafu(display("Meta request error: {}", msg))]
    #[error_code(code = 1)]
    MetaRequest {
        msg: String,
    },

    #[snafu(display("Io error: {}", msg))]
    #[error_code(code = 2)]
    IOErrors {
        msg: String,
    },

    #[snafu(display("Invalid serde message: {}", err))]
    #[error_code(code = 3)]
    InvalidSerdeMsg {
        err: String,
    },

    #[snafu(display("Fails to serialize or deserialize: {source}"))]
    BincodeSerde {
        source: bincode::Error,
    },

    #[snafu(display("Fails to send to channel: {}", msg))]
    #[error_code(code = 4)]
    ChannelSend {
        msg: String,
    },

    #[snafu(display("Fails to recv from channel: {}", msg))]
    #[error_code(code = 5)]
    ChannelRecv {
        msg: String,
    },

    #[snafu(display("Write vnode error: {}", msg))]
    #[error_code(code = 6)]
    WriteVnode {
        msg: String,
    },

    #[snafu(display("Error from models: {}", source))]
    #[error_code(code = 7)]
    ModelsError {
        source: models::Error,
    },

    #[snafu(display("Not found tenant: {}", name))]
    #[error_code(code = 9)]
    TenantNotFound {
        name: String,
    },

    #[snafu(display("Invalid flatbuffers: {}", source))]
    #[error_code(code = 10)]
    InvalidFlatbuffer {
        source: flatbuffers::InvalidFlatbuffer,
    },

    #[snafu(display("Unknow coordinator command: {}", cmd))]
    #[error_code(code = 11)]
    UnKnownCoordCmd {
        cmd: u32,
    },

    #[snafu(display("Coordinator command parse failed"))]
    #[error_code(code = 12)]
    CoordCommandParseErr,

    #[snafu(display("Unexpect response message"))]
    #[error_code(code = 13)]
    UnExpectResponse,

    #[snafu(display("{}", msg))]
    #[error_code(code = 14)]
    CommonError {
        msg: String,
    },

    #[snafu(display("Vnode not found: {}", id))]
    #[error_code(code = 15)]
    VnodeNotFound {
        id: u32,
    },

    #[snafu(display("pre-execution failed: {}", error))]
    #[error_code(code = 16)]
    PreExecution {
        error: String,
    },

    #[snafu(display("kv instance not found: node_id:{}", node_id))]
    #[error_code(code = 18)]
    KvInstanceNotFound {
        node_id: u64,
    },

    #[snafu(display("grpc client request error: {}", msg))]
    #[error_code(code = 19)]
    GRPCRequest {
        msg: String,
    },

    #[snafu(display("flatbuffer point miss field : {}", msg))]
    #[error_code(code = 20)]
    Points {
        msg: String,
    },

    #[snafu(display("{}", source))]
    #[error_code(code = 21)]
    FBPoints {
        source: PointsError,
    },

    #[snafu(display("ReplicationSet not found: {}", id))]
    #[error_code(code = 22)]
    ReplicationSetNotFound {
        id: u32,
    },

    #[snafu(display("Not enough valid replica of ReplicationSet({})", id))]
    #[error_code(code = 23)]
    NoValidReplica {
        id: u32,
    },

    #[snafu(display("Failed to convert '{from}' to '{to}' for timestamp: {ts}"))]
    #[error_code(code = 24)]
    NormalizeTimestamp {
        from: Precision,
        to: Precision,
        ts: Timestamp,
    },

    #[snafu(display("Writing expired timestamp {point_ts} which is less than {database_min_ts} to database '{database}'"))]
    #[error_code(code = 25)]
    PointTimestampExpired {
        database: String,
        database_min_ts: Timestamp,
        point_ts: Timestamp,
    },

    #[snafu(display("The Operation Can only Exec in Leader {:?}", replica))]
    #[error_code(code = 26)]
    LeaderIsWrong {
        replica: ReplicationSet,
    },

    #[snafu(display("Write to Raft Node Wrong ({})", msg))]
    #[error_code(code = 27)]
    RaftWriteError {
        msg: String,
    },

    #[snafu(display("Raft Group has Error ({})", msg))]
    #[error_code(code = 28)]
    RaftGroupError {
        msg: String,
    },

    #[snafu(display(
        "Forward to Leader (replcia id: {replica_id} leader vnode id: {leader_vnode_id})"
    ))]
    #[error_code(code = 29)]
    RaftForwardToLeader {
        replica_id: u32,
        leader_vnode_id: u32,
    },

    #[snafu(display("Raft Node not Found has ({})", id))]
    #[error_code(code = 30)]
    RaftNodeNotFound {
        id: ReplicationSetId,
    },

    #[error_code(code = 31)]
    #[snafu(display("Invalid configuration: {msg}"))]
    InvalidInitialConfig {
        msg: String,
    },
}

impl From<PointsError> for CoordinatorError {
    fn from(value: PointsError) -> Self {
        CoordinatorError::FBPoints { source: value }
    }
}

impl From<MetaError> for CoordinatorError {
    fn from(err: MetaError) -> Self {
        CoordinatorError::MetaRequest {
            msg: err.to_string(),
        }
    }
}

impl From<io::Error> for CoordinatorError {
    fn from(err: io::Error) -> Self {
        CoordinatorError::IOErrors {
            msg: err.to_string(),
        }
    }
}

impl From<tskv::TskvError> for CoordinatorError {
    fn from(err: tskv::TskvError) -> Self {
        match err {
            tskv::TskvError::Meta { source } => CoordinatorError::Meta { source },

            other => CoordinatorError::TskvError { source: other },
        }
    }
}

impl From<replication::errors::ReplicationError> for CoordinatorError {
    fn from(err: replication::errors::ReplicationError) -> Self {
        CoordinatorError::ReplicatError { source: err }
    }
}

impl From<ArrowError> for CoordinatorError {
    fn from(err: ArrowError) -> Self {
        match err {
            ArrowError::ExternalError(e) if e.downcast_ref::<CoordinatorError>().is_some() => {
                *e.downcast::<CoordinatorError>().unwrap()
            }
            ArrowError::ExternalError(e) if e.downcast_ref::<MetaError>().is_some() => {
                CoordinatorError::Meta {
                    source: *e.downcast::<MetaError>().unwrap(),
                }
            }
            ArrowError::ExternalError(e) if e.downcast_ref::<tskv::TskvError>().is_some() => {
                CoordinatorError::TskvError {
                    source: *e.downcast::<tskv::TskvError>().unwrap(),
                }
            }
            ArrowError::ExternalError(e) if e.downcast_ref::<ArrowError>().is_some() => {
                let arrow_error = *e.downcast::<ArrowError>().unwrap();
                arrow_error.into()
            }

            other => CoordinatorError::ArrowError { source: other },
        }
    }
}

impl From<DataFusionError> for CoordinatorError {
    fn from(err: DataFusionError) -> Self {
        match err {
            DataFusionError::ArrowError(e) => e.into(),
            other => CoordinatorError::DataFusionError { source: other },
        }
    }
}

impl From<bincode::Error> for CoordinatorError {
    fn from(err: bincode::Error) -> Self {
        Self::BincodeSerde { source: err }
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for CoordinatorError {
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        CoordinatorError::ChannelSend {
            msg: err.to_string(),
        }
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for CoordinatorError {
    fn from(err: tokio::sync::oneshot::error::RecvError) -> Self {
        CoordinatorError::ChannelRecv {
            msg: err.to_string(),
        }
    }
}

impl From<tonic::Status> for CoordinatorError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::Internal => CoordinatorError::GRPCRequest {
                msg: format!("status code: {}, message; {}", status.code(), status),
            },

            _ => CoordinatorError::PreExecution {
                error: format!("{}", status),
            },
        }
    }
}

impl From<models::Error> for CoordinatorError {
    fn from(err: models::Error) -> Self {
        CoordinatorError::ModelsError { source: err }
    }
}

impl CoordinatorError {
    pub fn error_code(&self) -> &dyn ErrorCode {
        match self {
            CoordinatorError::Meta { source } => source.error_code(),
            CoordinatorError::TskvError { source } => source.error_code(),
            _ => self,
        }
    }
}

impl From<flatbuffers::InvalidFlatbuffer> for CoordinatorError {
    fn from(value: InvalidFlatbuffer) -> Self {
        Self::InvalidFlatbuffer { source: value }
    }
}

pub const FORWARD_TO_LEADER_CODE: i32 = -2;
pub const FAILED_RESPONSE_CODE: i32 = -1;
pub const SUCCESS_RESPONSE_CODE: i32 = 1;

pub fn encode_grpc_response(
    result: CoordinatorResult<Vec<u8>>,
) -> tonic::Response<protos::kv_service::BatchBytesResponse> {
    match result {
        Ok(data) => tonic::Response::new(protos::kv_service::BatchBytesResponse {
            data,
            code: SUCCESS_RESPONSE_CODE,
        }),

        Err(err) => {
            if let CoordinatorError::RaftForwardToLeader {
                replica_id,
                leader_vnode_id: new_leader,
            } = err
            {
                tonic::Response::new(protos::kv_service::BatchBytesResponse {
                    data: format!("{}-{}", replica_id, new_leader).into(),
                    code: FORWARD_TO_LEADER_CODE,
                })
            } else {
                tonic::Response::new(protos::kv_service::BatchBytesResponse {
                    data: err.to_string().into_bytes(),
                    code: FAILED_RESPONSE_CODE,
                })
            }
        }
    }
}

pub fn decode_grpc_response(
    status: protos::kv_service::BatchBytesResponse,
) -> CoordinatorResult<Vec<u8>> {
    if status.code == SUCCESS_RESPONSE_CODE {
        Ok(status.data)
    } else if status.code == FORWARD_TO_LEADER_CODE {
        let data = String::from_utf8_lossy(&status.data).to_string();
        let strs: Vec<&str> = data.split('-').collect();
        let replica_id = strs[0].parse::<u32>().unwrap_or_default();
        let leader_vnode_id = strs[1].parse::<u32>().unwrap_or_default();
        Err(CoordinatorError::RaftForwardToLeader {
            replica_id,
            leader_vnode_id,
        })
    } else {
        let mut len = 256;
        if status.data.len() < len {
            len = status.data.len();
        }
        let tmp = String::from_utf8_lossy(&status.data[..len]).to_string();
        Err(CoordinatorError::GRPCRequest { msg: tmp })
    }
}

pub type CoordinatorResult<T> = Result<T, CoordinatorError>;

#[test]
fn test_mod_code() {
    let e = CoordinatorError::UnExpectResponse;
    assert!(e.code().starts_with("05"));
}
