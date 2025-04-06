use warp::reject::Reject;

#[derive(Debug)]
pub struct NotFoundError;

impl Reject for NotFoundError {}

#[derive(Debug)]

pub struct UnauthorizedError;

impl Reject for UnauthorizedError {}

#[derive(Debug)]
pub struct InvalidInputError;

impl Reject for InvalidInputError {}

#[derive(Debug)]
pub struct CannotJoinMatchError;

impl Reject for CannotJoinMatchError {}

#[derive(Debug)]
pub struct IdGenerationError;
impl Reject for IdGenerationError {}

#[derive(Debug)]
pub struct CannotBroadcastError;

impl Reject for CannotBroadcastError {}

#[derive(Debug)]
pub struct NoAvailablePorts;

impl Reject for NoAvailablePorts {}
