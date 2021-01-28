use {
    crate::Config,
    semver::{AlphaNumeric, Version},
    semver_parser::lexer::{Error, Lexer, Token},
    serde::Deserialize,
    std::{collections::BTreeMap, fs, path::Path},
    url::Url,
};

pub const BASE_URL: &str = "https://api.github.com";

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub name: String,
    pub zipball_url: Url,
    pub tarball_url: Url,
    pub commit: Commit,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub url: Url,
}

pub fn parse_hyphened<'input>(input: &'input str) -> Option<(u64, Option<&'input str>)> {
    let mut parts = input.splitn(2, |c: char| c == '_' || c == '-');

    let version = parts.next()?.parse().ok()?;
    let build = parts.next();

    Some((version, build))
}

fn parse_part<'input>(
    input: &'input [Result<Token<'input>, Error>],
) -> (u64, &'input [Result<Token<'input>, Error>]) {
    let (part, rest) = match input {
        [Ok(Token::Numeric(numeric)), Ok(Token::Dot), rest @ ..]
        | [Ok(Token::Numeric(numeric)), rest @ ..] => (Some(*numeric), rest),
        rest => (None, rest),
    };

    (part.unwrap_or(0), rest)
}

fn parse_pre<'input>(
    input: &'input [Result<Token<'input>, Error>],
) -> (Option<&str>, &'input [Result<Token<'input>, Error>]) {
    match &input {
        [Ok(Token::Hyphen), Ok(Token::AlphaNumeric(pre)), rest @ ..]
        | [Err(Error::UnexpectedChar('/')), Ok(Token::AlphaNumeric(pre)), rest @ ..] => {
            (Some(pre), rest)
        }
        rest => (None, rest),
    }
}

fn skip_junk<'input>(
    input: &'input [Result<Token<'input>, Error>],
) -> &'input [Result<Token<'input>, Error>] {
    let offset = input.iter().position(|token| match token {
        Ok(Token::Numeric(_)) => true,
        _ => false,
    });

    if let Some(offset) = offset {
        &input[offset..]
    } else {
        input
    }
}

pub fn parse_version(input: &str) -> Version {
    let mut input: Vec<_> = Lexer::new(&input[..]).collect();

    let input = skip_junk(&input[..]);
    let (major, input) = parse_part(&input[..]);
    let (minor, input) = parse_part(&input[..]);
    let (patch, input) = parse_part(&input[..]);
    let (pre, input) = parse_pre(&input[..]);

    let mut version = Version::new(major, minor, patch);

    if let Some(pre) = pre {
        version.pre.push(AlphaNumeric(pre.to_owned()));
    }

    version
}

pub async fn fetch_github_tags(
    config: &Config,
    name: impl AsRef<str>,
    user: impl AsRef<str>,
    repo: impl AsRef<str>,
) -> anyhow::Result<BTreeMap<Version, Tag>> {
    let path = Path::new(name.as_ref()).join("tags.json");
    let url = BASE_URL.to_owned() + "/repos/" + user.as_ref() + "/" + repo.as_ref() + "/tags";

    config.fetch_cached(&path, url.as_str()).await?;

    let tags: Vec<Tag> = serde_json::from_slice(&fs::read(config.cache_with(&path))?[..])?;
    let tags = tags
        .into_iter()
        .map(|tag| (parse_version(tag.name.as_str()), tag))
        .collect();

    println!("tags: {:?}", &tags);

    Ok(tags)
}

/*#[derive(Debug, Deserialize)]
pub struct Ref {
    #[serde(rename = "ref")]
    pub reference: PathBuf,
    pub url: String,
    pub object: Object,
}

#[derive(Debug, Deserialize)]
pub struct Object {
    pub sha: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: Url,
}

pub async fn fetch_github_refs(
    client: &Client,
    name: impl AsRef<str>,
    user: impl AsRef<str>,
    repo: impl AsRef<str>,
) -> anyhow::Result<HashMap<PathBuf, Ref>> {
    let name = name.as_ref();
    let user = user.as_ref();
    let repo = repo.as_ref();
    let url = format!(
        "https://api.github.com/repos/{}/{}/git/refs/tags",
        &user, &repo
    );

    let bytes = client.get(&name, "refs.json", url.as_str()).await?;
    let refs: Vec<Ref> = serde_json::from_slice(&bytes)?;

    Ok(refs
        .into_iter()
        .map(|reference| (reference.reference.clone(), reference))
        .collect())
}*/
