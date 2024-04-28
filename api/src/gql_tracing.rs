use async_graphql::{
    extensions::{
        Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextRequest,
        NextResolve, NextSubscribe, NextValidation, ResolveInfo,
    },
    parser::types::ExecutableDocument,
    Response, ServerError, ServerResult, ValidationResult, Value, Variables,
};
use futures_util::{stream::BoxStream, TryFutureExt};
use std::sync::Arc;
use tracing::{span, Level};
use tracing_futures::Instrument;

pub struct DoNotTrace;
pub struct Tracing;

impl ExtensionFactory for Tracing {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(TracingExtension)
    }
}

struct TracingExtension;

#[async_trait::async_trait]
impl Extension for TracingExtension {
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
        if ctx.data_opt::<DoNotTrace>().is_some() {
            return next.run(ctx).await;
        }

        next.run(ctx)
            .instrument(span!(
                target: "async_graphql::graphql",
                Level::INFO,
                "request",
            ))
            .await
    }

    fn subscribe<'s>(
        &self,
        ctx: &ExtensionContext<'_>,
        stream: BoxStream<'s, Response>,
        next: NextSubscribe<'_>,
    ) -> BoxStream<'s, Response> {
        Box::pin(next.run(ctx, stream).instrument(span!(
            target: "async_graphql::graphql",
            Level::INFO,
            "subscribe",
        )))
    }

    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        if ctx.data_opt::<DoNotTrace>().is_some() {
            return next.run(ctx, query, variables).await;
        }

        let span = span!(
            target: "async_graphql::graphql",
            Level::INFO,
            "parse",
            source = tracing::field::Empty
        );

        async move {
            let res = next.run(ctx, query, variables).await;
            if let Ok(doc) = &res {
                tracing::Span::current()
                    .record("source", ctx.stringify_execute_doc(doc, variables).as_str());
            }
            res
        }
        .instrument(span)
        .await
    }

    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        if ctx.data_opt::<DoNotTrace>().is_some() {
            return next.run(ctx).await;
        }

        let span = span!(
            target: "async_graphql::graphql",
            Level::INFO,
            "validation"
        );
        next.run(ctx).instrument(span).await
    }

    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        if ctx.data_opt::<DoNotTrace>().is_some() {
            return next.run(ctx, operation_name).await;
        }

        let span = span!(
            target: "async_graphql::graphql",
            Level::INFO,
            "execute"
        );
        next.run(ctx, operation_name).instrument(span).await
    }

    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        if ctx.data_opt::<DoNotTrace>().is_some() {
            return next.run(ctx, info).await;
        }

        let span = if !info.is_for_introspection {
            Some(span!(
                target: "async_graphql::graphql",
                Level::INFO,
                "field",
                path = %info.path_node,
                parent_type = %info.parent_type,
                return_type = %info.return_type,
            ))
        } else {
            None
        };

        let fut = next.run(ctx, info).inspect_err(|err| {
            tracing::info!(
                target: "async_graphql::graphql",
                error = %err.message,
                "error",
            );
        });
        match span {
            Some(span) => fut.instrument(span).await,
            None => fut.await,
        }
    }
}
