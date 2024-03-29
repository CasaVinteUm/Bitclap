# Bitclap

Generate the post md file for your bitdevs meetup using Github issues.

# How to use

1. To use the Github API you'll need to follow these [instructions](https://docs.github.com/en/rest/quickstart).
2. Create a .env file following the .env.sample
3. Make sure to have your issues in the following format:

```
<title>
<url>
```

# Usage

```
cargo run -- --org-name=lorenzolfm --repo-name=floripabitdevs --issue-number=12 --meetup-date=2024-01-01 --meetup-number=23 --meetup-link=google.com
```
