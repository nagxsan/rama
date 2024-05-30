//! Set required headers on the response, if they are missing.
//!
//! For now this only sets `Server` and `Date` heades.

use crate::http::{
    header::{DATE, SERVER},
    headers::{Date, HeaderMapExt},
};
use crate::service::{Context, Layer, Service};
use crate::{
    error::BoxError,
    http::{
        header::{self, RAMA_ID_HEADER_VALUE},
        Request, Response,
    },
};
use std::{fmt, time::SystemTime};

/// Layer that applies [`RequiredResponseHeader`] which adds a request header.
///
/// See [`RequiredResponseHeader`] for more details.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct RequiredResponseHeaderLayer;

impl RequiredResponseHeaderLayer {
    /// Create a new [`RequiredResponseHeaderLayer`].
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for RequiredResponseHeaderLayer {
    type Service = RequiredResponseHeader<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequiredResponseHeader { inner }
    }
}

/// Middleware that sets a header on the request.
#[derive(Clone)]
pub struct RequiredResponseHeader<S> {
    inner: S,
}

impl<S> RequiredResponseHeader<S> {
    /// Create a new [`RequiredResponseHeader`].
    pub fn new(inner: S) -> Self {
        Self { inner }
    }

    define_inner_service_accessors!();
}

impl<S> fmt::Debug for RequiredResponseHeader<S>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequiredResponseHeader")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<ReqBody, ResBody, State, S> Service<State, Request<ReqBody>> for RequiredResponseHeader<S>
where
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
    State: Send + Sync + 'static,
    S: Service<State, Request<ReqBody>, Response = Response<ResBody>>,
    S::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;

    async fn serve(
        &self,
        ctx: Context<State>,
        mut req: Request<ReqBody>,
    ) -> Result<Self::Response, Self::Error> {
        if let header::Entry::Vacant(header) = req.headers_mut().entry(SERVER) {
            header.insert(RAMA_ID_HEADER_VALUE.clone());
        }

        if !req.headers().contains_key(DATE) {
            req.headers_mut()
                .typed_insert(Date::from(SystemTime::now()));
        }

        self.inner.serve(ctx, req).await.map_err(Into::into)
    }
}
