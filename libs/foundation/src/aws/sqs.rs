use anyhow::{bail, Context};

pub async fn send_to_queue<T>(
    client: &aws_sdk_sqs::Client,
    queue_name: &str,
    message: T,
) -> Result<(), anyhow::Error>
where
    T: serde::Serialize,
{
    let queue_url_output = client.get_queue_url().queue_name(queue_name).send().await?;

    if let Some(queue_url) = queue_url_output.queue_url() {
        send_to_queue_by_url(client, queue_url, message).await?;
    } else {
        bail!("missing queue url for {}", queue_name);
    }

    Ok(())
}

pub async fn send_to_queue_by_url<T>(
    client: &aws_sdk_sqs::Client,
    queue_url: &str,
    message: T,
) -> Result<(), anyhow::Error>
where
    T: serde::Serialize,
{
    let resp = client
        .send_message()
        .queue_url(queue_url)
        .message_body(serde_json::to_string(&message).context("must serialize")?)
        .send()
        .await?;
    log::info!("added to queue: {:?}", resp);

    Ok(())
}
