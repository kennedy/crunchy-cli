use anyhow::Result;
use crunchyroll_rs::media::{Resolution, Stream, VariantData};
use crunchyroll_rs::Locale;

pub async fn variant_data_from_stream(
    stream: &Stream,
    resolution: &Resolution,
) -> Result<Option<(VariantData, VariantData)>> {
    // sometimes Crunchyroll marks episodes without real subtitles that they have subtitles and
    // reports that only hardsub episode are existing. the following lines are trying to prevent
    // potential errors which might get caused by this incorrect reporting
    // (https://github.com/crunchy-labs/crunchy-cli/issues/231)
    let mut hardsub_locales = stream.streaming_hardsub_locales();
    let hardsub_locale = if !hardsub_locales.contains(&Locale::Custom("".to_string()))
        && !hardsub_locales.contains(&Locale::Custom(":".to_string()))
    {
        // if only one hardsub locale exists, assume that this stream doesn't really contains hardsubs
        if hardsub_locales.len() == 1 {
            Some(hardsub_locales.remove(0))
        } else {
            // fallback to `None`. this should trigger an error message in `stream.dash_streaming_data`
            // that the requested stream is not available
            None
        }
    } else {
        None
    };

    let mut streaming_data = stream.dash_streaming_data(hardsub_locale).await?;
    streaming_data
        .0
        .sort_by(|a, b| a.bandwidth.cmp(&b.bandwidth).reverse());
    streaming_data
        .1
        .sort_by(|a, b| a.bandwidth.cmp(&b.bandwidth).reverse());

    let video_variant = match resolution.height {
        u64::MAX => Some(streaming_data.0.into_iter().next().unwrap()),
        u64::MIN => Some(streaming_data.0.into_iter().last().unwrap()),
        _ => streaming_data
            .0
            .into_iter()
            .find(|v| resolution.height == v.resolution.height),
    };
    Ok(video_variant.map(|v| (v, streaming_data.1.first().unwrap().clone())))
}
