use crate::util::errors::OpenBitFunResult;
use openbitfun_services_core::coordination_persistence as storage;
pub(crate) use openbitfun_services_core::coordination_persistence::COORDINATION_SCHEMA_VERSION;
use rusqlite::Connection;

pub(crate) fn initialize_coordination_schema(connection: &Connection) -> OpenBitFunResult<()> {
    storage::initialize_coordination_schema(connection).map_err(Into::into)
}
pub(crate) fn coordination_table_has_column(
    connection: &Connection,
    table: &str,
    column: &str,
) -> OpenBitFunResult<bool> {
    storage::coordination_table_has_column(connection, table, column).map_err(Into::into)
}
pub(crate) fn validate_coordination_agent_id(agent_id: &str) -> OpenBitFunResult<()> {
    storage::validate_coordination_agent_id(agent_id).map_err(Into::into)
}
