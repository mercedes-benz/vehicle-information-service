// SPDX-License-Identifier: MIT

///
/// GET_VSS request
///
#[derive(Debug)]
struct GetMetadata {
    path: ActionPath,
    request_id: ReqID,
}