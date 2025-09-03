use async_graphql::{
    Response, ServerError, ServerResult, ValidationResult, Value, Variables,
    extensions::{
        Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextRequest,
        NextResolve, NextValidation, ResolveInfo,
    },
    parser::types::ExecutableDocument,
};
use futures_util::TryFutureExt;
use std::sync::Arc;
use tracing_futures::Instrument;

#[allow(unused)]
pub struct Tracing;

impl ExtensionFactory for Tracing {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(TracingExtension)
    }
}

#[allow(unused)]
struct TracingExtension;

#[async_trait::async_trait]
impl Extension for TracingExtension {
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
        next.run(ctx)
            .instrument(tracing::span!(
                target: "async_graphql::graphql",
                tracing::Level::INFO,
                "request",
            ))
            .await
    }

    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let span = tracing::span!(
            target: "async_graphql::graphql",
            tracing::Level::INFO,
            "parse_query",
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
        let span = tracing::span!(
            target: "async_graphql::graphql",
            tracing::Level::INFO,
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
        let span = tracing::span!(
            target: "async_graphql::graphql",
            tracing::Level::INFO,
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
        next.run(ctx, info)
            .inspect_err(|err| {
                tracing::error!(
                    target: "async_graphql::graphql",
                    error = %err.message,
                    "error",
                );
            })
            .await
    }
}
