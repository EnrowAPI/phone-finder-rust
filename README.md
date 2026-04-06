# Phone Finder - Rust Library

[![crates.io](https://img.shields.io/crates/v/phone-finder.svg)](https://crates.io/crates/phone-finder)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![GitHub stars](https://img.shields.io/github/stars/EnrowAPI/phone-finder-rust)](https://github.com/EnrowAPI/phone-finder-rust)
[![Last commit](https://img.shields.io/github/last-commit/EnrowAPI/phone-finder-rust)](https://github.com/EnrowAPI/phone-finder-rust/commits)

Find mobile phone numbers from LinkedIn profiles or a name and company. Integrate phone discovery into your sales outreach or enrichment pipeline.

Powered by [Enrow](https://enrow.io) -- only charged when a phone number is found.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
phone-finder = "1.0"
tokio = { version = "1", features = ["full"] }
```

## Simple Usage

### Search by LinkedIn URL (preferred)

```rust
use phone_finder::{find_phone, get_phone_result, FindPhoneParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params = FindPhoneParams {
        linkedin_url: Some("https://www.linkedin.com/in/timcook/".into()),
        fullname: None,
        company_domain: None,
        company_name: None,
        webhook: None,
    };

    let search = find_phone("your_api_key", &params).await?;
    let result = get_phone_result("your_api_key", &search.id).await?;

    println!("{}", result.number.unwrap_or_default());        // +14155551234
    println!("{}", result.country.unwrap_or_default());       // US
    println!("{}", result.qualification.unwrap_or_default()); // found

    Ok(())
}
```

### Search by name and company

```rust
let params = FindPhoneParams {
    linkedin_url: None,
    fullname: Some("Tim Cook".into()),
    company_domain: Some("apple.com".into()),
    company_name: None,
    webhook: None,
};

let search = find_phone("your_api_key", &params).await?;
```

`find_phone` returns a search ID. The search runs asynchronously -- call `get_phone_result` to retrieve the result once it's ready. You can also pass a `webhook` URL to get notified automatically.

## Search by company name

If you don't have the domain, you can search by company name instead.

```rust
let params = FindPhoneParams {
    linkedin_url: None,
    fullname: Some("Tim Cook".into()),
    company_domain: None,
    company_name: Some("Apple Inc.".into()),
    webhook: None,
};

let search = find_phone("your_api_key", &params).await?;
```

## Bulk search

```rust
use phone_finder::{find_phones, get_phone_results, FindPhonesParams, BulkSearch};

let params = FindPhonesParams {
    searches: vec![
        BulkSearch {
            linkedin_url: Some("https://www.linkedin.com/in/timcook/".into()),
            fullname: None,
            company_domain: None,
            company_name: None,
            custom: None,
        },
        BulkSearch {
            linkedin_url: None,
            fullname: Some("Satya Nadella".into()),
            company_domain: Some("microsoft.com".into()),
            company_name: None,
            custom: None,
        },
        BulkSearch {
            linkedin_url: None,
            fullname: Some("Jensen Huang".into()),
            company_domain: None,
            company_name: Some("NVIDIA".into()),
            custom: None,
        },
    ],
    webhook: None,
};

let batch = find_phones("your_api_key", &params).await?;
// batch.batch_id, batch.total, batch.status

let results = get_phone_results("your_api_key", &batch.batch_id).await?;
// results.results -- Vec<PhoneResult>
```

Up to 5,000 searches per batch. Pass a `webhook` URL to get notified when the batch completes.

## Error handling

```rust
match find_phone("bad_key", &params).await {
    Ok(result) => println!("Found: {:?}", result),
    Err(e) => {
        // e contains the API error description
        // Common errors:
        // - "Invalid or missing API key" (401)
        // - "Your credit balance is insufficient." (402)
        // - "Rate limit exceeded" (429)
        eprintln!("Error: {}", e);
    }
}
```

## Getting an API key

Register at [app.enrow.io](https://app.enrow.io) to get your API key. You get **50 free credits** with no credit card required.

50 credits per phone found (only charged when found). Paid plans start at **$17/mo** for 20 phones up to **$497/mo** for 2,000 phones. See [pricing](https://enrow.io/pricing).

## Documentation

- [Enrow API documentation](https://docs.enrow.io)
- [Full Enrow SDK](https://github.com/EnrowAPI/enrow-rust) -- includes email finder, email verifierand more

## License

MIT -- see [LICENSE](LICENSE) for details.
