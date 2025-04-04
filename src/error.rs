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
